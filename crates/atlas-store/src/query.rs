//! Phase 3 typed reads over a single authorized historical snapshot.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::{
    AuthorRevision, Error, EventRole, ObjectData, ObjectRef, Scope, Store, Version, ViewContext,
    WorldId,
};

use crate::semantic::{AreaTest, ConceptProfile};

pub const QUERY_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct QuerySpec {
    pub schema_version: u16,
    pub filter: QueryFilter,
    pub limit: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum QueryFilter {
    LabelContains(String),
    ExactLabel(String),
    /// Exact active author alias, returning all matching canonical people.
    ExactAlias(String),
    /// Literal token phrase over versioned object labels, indexed with FTS5.
    FullTextPhrase(String),
    /// Literal token phrase in versioned, source-backed claim statements.
    ClaimFullTextPhrase(String),
    Kind(ObjectKindFilter),
    ClassifiedAs(ConceptProfile),
    ClassifiedBy(ObjectRef),
    SiteArea(AreaTest),
    HasSource(ObjectRef),
    And(Vec<QueryFilter>),
    Or(Vec<QueryFilter>),
    Not(Box<QueryFilter>),
    Backlinks(ObjectRef),
    EventRole {
        event: ObjectRef,
        role: EventRole,
    },
    /// Return visible historical events involving a specific person,
    /// optionally in one explicitly recorded role.
    ParticipantEvents {
        participant: ObjectRef,
        role: Option<EventRole>,
    },
    /// Author-only strict containment in one named hierarchy.
    WithinPlace {
        root: ObjectRef,
        kind: crate::network::NetworkKind,
        lens: String,
        include_self: bool,
        max_depth: u8,
    },
    RelationshipPath {
        start: ObjectRef,
        kind: String,
        direction: Direction,
        max_depth: u8,
        max_visited: usize,
    },
}

/// Allowlisted record kinds for author-only historical predicates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKindFilter {
    Person,
    Concept,
    Place,
    Organization,
    Household,
    FamilyHouse,
    Faction,
    Conflict,
    Office,
    Title,
    Asset,
    Population,
    ConflictIssue,
    ConflictPhase,
    SuccessionRule,
    Definition,
    Relationship,
    EventInvolvement,
    ActorAction,
    HistoricalEvent,
    Source,
    Claim,
}

impl From<&ObjectData> for ObjectKindFilter {
    fn from(data: &ObjectData) -> Self {
        match data {
            ObjectData::Person => Self::Person,
            ObjectData::Concept => Self::Concept,
            ObjectData::Place => Self::Place,
            ObjectData::Organization => Self::Organization,
            ObjectData::Household => Self::Household,
            ObjectData::FamilyHouse => Self::FamilyHouse,
            ObjectData::Faction => Self::Faction,
            ObjectData::Conflict => Self::Conflict,
            ObjectData::Office => Self::Office,
            ObjectData::Title => Self::Title,
            ObjectData::Asset => Self::Asset,
            ObjectData::Population => Self::Population,
            ObjectData::ConflictIssue => Self::ConflictIssue,
            ObjectData::ConflictPhase => Self::ConflictPhase,
            ObjectData::SuccessionRule => Self::SuccessionRule,
            ObjectData::Definition { .. } => Self::Definition,
            ObjectData::Relationship { .. } => Self::Relationship,
            ObjectData::EventInvolvement { .. } => Self::EventInvolvement,
            ObjectData::ActorAction { .. } => Self::ActorAction,
            ObjectData::HistoricalEvent { .. } => Self::HistoricalEvent,
            ObjectData::Source { .. } => Self::Source,
            ObjectData::Claim { .. } => Self::Claim,
        }
    }
}

/// Versioned, allowlisted wire/persistence shape for composite predicates.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum PredicateInput {
    Contains { text: String },
    Exact { text: String },
    ExactAlias { text: String },
    FullText { text: String },
    ClaimFullText { text: String },
    ObjectKind { kind: ObjectKindFilter },
    ClassifiedAs { profile: ConceptProfile },
    ClassifiedBy { descriptor_id: String },
    SiteArea { test: AreaTest },
    HasSource { source_id: String },
    And { filters: Vec<PredicateInput> },
    Or { filters: Vec<PredicateInput> },
    Not { filter: Box<PredicateInput> },
}

impl PredicateInput {
    /// Decode a typed predicate with the same depth/node rules as the
    /// authoritative planner. Only its exact saved world supplies references.
    pub fn into_filter(
        self,
        world: WorldId,
        depth: usize,
        nodes: &mut usize,
    ) -> Result<QueryFilter, Error> {
        *nodes += 1;
        if depth > MAX_PREDICATE_DEPTH || *nodes > MAX_PREDICATE_NODES {
            return Err(Error::QueryBudgetExceeded);
        }
        Ok(match self {
            Self::Contains { text } => QueryFilter::LabelContains(text),
            Self::Exact { text } => QueryFilter::ExactLabel(text),
            Self::ExactAlias { text } => QueryFilter::ExactAlias(text),
            Self::FullText { text } => QueryFilter::FullTextPhrase(text),
            Self::ClaimFullText { text } => QueryFilter::ClaimFullTextPhrase(text),
            Self::ObjectKind { kind } => QueryFilter::Kind(kind),
            Self::ClassifiedAs { profile } => QueryFilter::ClassifiedAs(profile),
            Self::SiteArea { test } => {
                test.validate()?;
                QueryFilter::SiteArea(test)
            }
            Self::ClassifiedBy { descriptor_id } => QueryFilter::ClassifiedBy(ObjectRef {
                world,
                id: crate::ObjectId(
                    Uuid::parse_str(&descriptor_id)
                        .map_err(|_| Error::InvalidInput("predicate descriptor ID"))?,
                ),
            }),
            Self::HasSource { source_id } => QueryFilter::HasSource(ObjectRef {
                world,
                id: crate::ObjectId(
                    Uuid::parse_str(&source_id)
                        .map_err(|_| Error::InvalidInput("predicate source ID"))?,
                ),
            }),
            Self::And { filters } | Self::Or { filters }
                if filters.len() > MAX_BOOLEAN_CHILDREN =>
            {
                return Err(Error::QueryBudgetExceeded);
            }
            Self::And { filters } | Self::Or { filters } if filters.len() < 2 => {
                return Err(Error::InvalidInput("boolean predicate children"));
            }
            Self::And { filters } => QueryFilter::And(
                filters
                    .into_iter()
                    .map(|item| item.into_filter(world, depth + 1, nodes))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Self::Or { filters } => QueryFilter::Or(
                filters
                    .into_iter()
                    .map(|item| item.into_filter(world, depth + 1, nodes))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Self::Not { filter } => {
                QueryFilter::Not(Box::new(filter.into_filter(world, depth + 1, nodes)?))
            }
        })
    }

    /// Encode only the allowed typed predicates; no raw SQL or arbitrary
    /// graph execution can be introduced through saved metadata.
    pub fn from_filter(filter: &QueryFilter) -> Result<Self, Error> {
        Ok(match filter {
            QueryFilter::LabelContains(text) => Self::Contains { text: text.clone() },
            QueryFilter::ExactLabel(text) => Self::Exact { text: text.clone() },
            QueryFilter::ExactAlias(text) => Self::ExactAlias { text: text.clone() },
            QueryFilter::FullTextPhrase(text) => Self::FullText { text: text.clone() },
            QueryFilter::ClaimFullTextPhrase(text) => Self::ClaimFullText { text: text.clone() },
            QueryFilter::Kind(kind) => Self::ObjectKind { kind: *kind },
            QueryFilter::ClassifiedAs(profile) => Self::ClassifiedAs { profile: *profile },
            QueryFilter::SiteArea(test) => Self::SiteArea { test: *test },
            QueryFilter::ClassifiedBy(descriptor) => Self::ClassifiedBy {
                descriptor_id: descriptor.id.0.to_string(),
            },
            QueryFilter::HasSource(source) => Self::HasSource {
                source_id: source.id.0.to_string(),
            },
            QueryFilter::And(filters) => Self::And {
                filters: filters
                    .iter()
                    .map(Self::from_filter)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            QueryFilter::Or(filters) => Self::Or {
                filters: filters
                    .iter()
                    .map(Self::from_filter)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            QueryFilter::Not(filter) => Self::Not {
                filter: Box::new(Self::from_filter(filter)?),
            },
            _ => return Err(Error::InvalidInput("unsupported saved predicate")),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    Outgoing,
    Incoming,
    Both,
}

/// Author-owned, versioned definition. Running it always reevaluates the
/// QuerySpec with the caller's current ViewContext; no results are persisted.
/// An explicit, author-scoped write with optimistic concurrency and an
/// idempotency identity. Query results are never saved in this command.
pub struct SaveNamedLabelQuery<'a> {
    pub world_id: WorldId,
    pub expected: AuthorRevision,
    pub query_id: Option<Uuid>,
    pub name: &'a str,
    pub spec: &'a QuerySpec,
    pub command_id: Uuid,
    pub scope: Scope,
}

#[derive(Clone, Debug)]
pub struct SavedLabelQuery {
    pub id: Uuid,
    pub name: String,
    pub world: WorldId,
    pub spec: QuerySpec,
    pub recorded_at: AuthorRevision,
    pub parameters: Vec<QueryParameter>,
}

/// Only leaf values may vary. Operators, shape, scope, and context are frozen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParameterValue {
    Text {
        value: String,
    },
    ObjectKind {
        value: ObjectKindFilter,
    },
    ConceptProfile {
        value: ConceptProfile,
    },
    Descriptor {
        world_id: String,
        descriptor_id: String,
    },
    Source {
        world_id: String,
        source_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryParameter {
    pub name: String,
    pub default: ParameterValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterBinding {
    pub name: String,
    pub value: ParameterValue,
}

// The deterministic pre-order leaf walk is part of the v1 parameter contract.
// Defaults are taken from the actual typed saved AST, never cached query rows.
fn visit_parameter_leaves(
    filter: &mut QueryFilter,
    visit: &mut impl FnMut(&str, &mut ParameterValue) -> Result<(), Error>,
) -> Result<(), Error> {
    let (prefix, mut value) = match filter {
        QueryFilter::LabelContains(text) => (
            "contains",
            ParameterValue::Text {
                value: text.clone(),
            },
        ),
        QueryFilter::ExactLabel(text) => (
            "exact",
            ParameterValue::Text {
                value: text.clone(),
            },
        ),
        QueryFilter::ExactAlias(text) => (
            "alias",
            ParameterValue::Text {
                value: text.clone(),
            },
        ),
        QueryFilter::FullTextPhrase(text) => (
            "label_phrase",
            ParameterValue::Text {
                value: text.clone(),
            },
        ),
        QueryFilter::ClaimFullTextPhrase(text) => (
            "claim_phrase",
            ParameterValue::Text {
                value: text.clone(),
            },
        ),
        QueryFilter::Kind(kind) => ("object_kind", ParameterValue::ObjectKind { value: *kind }),
        QueryFilter::ClassifiedAs(profile) => (
            "concept_profile",
            ParameterValue::ConceptProfile { value: *profile },
        ),
        QueryFilter::SiteArea(_) => return Ok(()),
        QueryFilter::ClassifiedBy(descriptor) => (
            "descriptor",
            ParameterValue::Descriptor {
                world_id: descriptor.world.0.to_string(),
                descriptor_id: descriptor.id.0.to_string(),
            },
        ),
        QueryFilter::HasSource(source) => (
            "source",
            ParameterValue::Source {
                world_id: source.world.0.to_string(),
                source_id: source.id.0.to_string(),
            },
        ),
        QueryFilter::And(children) | QueryFilter::Or(children) => {
            for child in children {
                visit_parameter_leaves(child, visit)?;
            }
            return Ok(());
        }
        QueryFilter::Not(inner) => return visit_parameter_leaves(inner, visit),
        _ => return Err(Error::InvalidInput("unsupported parameterized predicate")),
    };
    visit(prefix, &mut value)?;
    match (filter, value) {
        (
            QueryFilter::LabelContains(text)
            | QueryFilter::ExactLabel(text)
            | QueryFilter::ExactAlias(text)
            | QueryFilter::FullTextPhrase(text)
            | QueryFilter::ClaimFullTextPhrase(text),
            ParameterValue::Text { value },
        ) => {
            *text = value;
        }
        (QueryFilter::Kind(kind), ParameterValue::ObjectKind { value }) => *kind = value,
        (QueryFilter::ClassifiedAs(profile), ParameterValue::ConceptProfile { value }) => {
            *profile = value;
        }
        (
            QueryFilter::ClassifiedBy(descriptor),
            ParameterValue::Descriptor {
                world_id,
                descriptor_id,
            },
        ) => {
            if Uuid::parse_str(&world_id).ok() != Some(descriptor.world.0) {
                return Err(Error::WrongWorld);
            }
            descriptor.id = crate::ObjectId(
                Uuid::parse_str(&descriptor_id)
                    .map_err(|_| Error::InvalidInput("parameter descriptor ID"))?,
            );
        }
        (
            QueryFilter::HasSource(source),
            ParameterValue::Source {
                world_id,
                source_id,
            },
        ) => {
            if Uuid::parse_str(&world_id).ok() != Some(source.world.0) {
                return Err(Error::WrongWorld);
            }
            source.id = crate::ObjectId(
                Uuid::parse_str(&source_id)
                    .map_err(|_| Error::InvalidInput("parameter source ID"))?,
            );
        }
        _ => return Err(Error::InvalidInput("query parameter type")),
    }
    Ok(())
}

pub(crate) fn query_parameters(
    spec: &QuerySpec,
    world: WorldId,
) -> Result<Vec<QueryParameter>, Error> {
    let mut nodes = 0;
    validate_predicate(&spec.filter, world, 0, &mut nodes)?;
    let mut parameters = Vec::new();
    visit_parameter_leaves(&mut spec.filter.clone(), &mut |prefix, default| {
        parameters.push(QueryParameter {
            name: format!("{prefix}_{}", parameters.len() + 1),
            default: default.clone(),
        });
        Ok(())
    })?;
    Ok(parameters)
}

impl SavedLabelQuery {
    /// Materialize a normal typed query with overrides, using defaults for
    /// omitted parameters. No caller-supplied operator or context is accepted.
    pub fn bind(
        &self,
        view: ViewContext,
        bindings: &[ParameterBinding],
    ) -> Result<QuerySpec, Error> {
        if view.scope != Scope::Author {
            return Err(Error::CapabilityDenied);
        }
        if view.world != self.world {
            return Err(Error::WrongWorld);
        }
        if view.revision < self.recorded_at {
            return Err(Error::InvalidRevision);
        }
        validate_query_request(&self.spec, view)?;
        if self.parameters.is_empty() {
            return if bindings.is_empty() {
                Ok(self.spec.clone())
            } else {
                Err(Error::InvalidInput("query has no parameters"))
            };
        }
        if self.parameters != query_parameters(&self.spec, self.world)? {
            return Err(Error::InvalidInput("saved parameter schema"));
        }
        if bindings.len() > self.parameters.len() {
            return Err(Error::InvalidInput("query parameter count"));
        }
        let mut remaining = HashMap::new();
        for binding in bindings {
            if remaining
                .insert(binding.name.as_str(), &binding.value)
                .is_some()
            {
                return Err(Error::InvalidInput("duplicate query parameter"));
            }
        }
        let mut spec = self.spec.clone();
        let mut index = 0;
        visit_parameter_leaves(&mut spec.filter, &mut |_, value| {
            let parameter = &self.parameters[index];
            index += 1;
            if let Some(replacement) = remaining.remove(parameter.name.as_str()) {
                *value = replacement.clone();
            }
            Ok(())
        })?;
        if !remaining.is_empty() {
            return Err(Error::InvalidInput("unknown query parameter"));
        }
        let mut nodes = 0;
        validate_predicate(&spec.filter, self.world, 0, &mut nodes)?;
        Ok(spec)
    }
}

#[derive(Clone, Debug)]
pub struct QueryResult {
    pub rows: Vec<Version>,
    pub revision: AuthorRevision,
    pub truncated: bool,
}

/// Opaque in-process continuation tied to the entire original read context,
/// typed query and page size. The host must not accept client-authored fields.
#[derive(Clone, Debug)]
pub struct QueryCursor {
    spec: QuerySpec,
    view: ViewContext,
    last: ObjectRef,
    label: String,
}

#[derive(Clone, Debug)]
pub struct QueryPage {
    pub rows: Vec<Version>,
    pub revision: AuthorRevision,
    pub next_cursor: Option<QueryCursor>,
}

/// One cooperative read cancellation/deadline, shared with the requesting host.
/// It cannot cancel writes or mutate the historical query context.
#[derive(Clone, Debug)]
pub struct QueryControl {
    cancelled: Arc<AtomicBool>,
    deadline: Instant,
}

impl QueryControl {
    pub fn new(timeout: Duration) -> Result<Self, Error> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .ok_or(Error::InvalidInput("query deadline"))?;
        Ok(Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            deadline,
        })
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn check(&self) -> Result<(), Error> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(Error::QueryCancelled);
        }
        if Instant::now() >= self.deadline {
            return Err(Error::QueryDeadlineExceeded);
        }
        Ok(())
    }
}

fn validate_query_request(spec: &QuerySpec, view: ViewContext) -> Result<(), Error> {
    if view.scope != Scope::Author {
        return Err(Error::CapabilityDenied);
    }
    if spec.schema_version != QUERY_SCHEMA_VERSION || !(1..=500).contains(&spec.limit) {
        return Err(Error::InvalidInput("query schema or result limit"));
    }
    Ok(())
}

impl Store {
    pub fn execute_query(&self, spec: &QuerySpec, view: ViewContext) -> Result<QueryResult, Error> {
        validate_query_request(spec, view)?;
        self.execute_bounded_query(spec, view)
    }

    /// Execute a read with a cooperative deadline and cancellation token.
    pub fn execute_query_controlled(
        &self,
        spec: &QuerySpec,
        view: ViewContext,
        control: &QueryControl,
    ) -> Result<QueryResult, Error> {
        validate_query_request(spec, view)?;
        self.with_query_control(control, |store| store.execute_query(spec, view))
    }

    /// Execute one complete bounded page with the same cancellation rules.
    pub fn execute_query_page_controlled(
        &self,
        spec: &QuerySpec,
        view: ViewContext,
        after: Option<&QueryCursor>,
        control: &QueryControl,
    ) -> Result<QueryPage, Error> {
        validate_query_request(spec, view)?;
        self.with_query_control(control, |store| store.execute_query_page(spec, view, after))
    }

    /// Page only complete, bounded author queries. Reevaluate the immutable
    /// recorded view on each call; do not cache raw rows or trust offsets.
    /// Standalone classification profile/descriptor predicates seek an indexed label/UUID window.
    /// Other predicates retain the existing bounded materialization planner.
    pub fn execute_query_page(
        &self,
        spec: &QuerySpec,
        view: ViewContext,
        after: Option<&QueryCursor>,
    ) -> Result<QueryPage, Error> {
        validate_query_request(spec, view)?;
        if matches!(spec.filter, QueryFilter::RelationshipPath { .. }) {
            return Err(Error::InvalidInput("pagination of bounded traversal"));
        }
        if let Some(cursor) = after
            && (cursor.view != view || cursor.spec != *spec)
        {
            return Err(Error::InvalidInput("query cursor context"));
        }
        if matches!(
            &spec.filter,
            QueryFilter::ClassifiedAs(_) | QueryFilter::ClassifiedBy(_)
        ) {
            let boundary = after.map(|cursor| (cursor.label.as_str(), cursor.last));
            let (rows, has_more) =
                self.classification_page(view, &spec.filter, boundary, spec.limit)?;
            let next_cursor = if has_more {
                rows.last().map(|last| QueryCursor {
                    spec: spec.clone(),
                    view,
                    last: last.object,
                    label: last.record.label.clone(),
                })
            } else {
                None
            };
            return Ok(QueryPage {
                rows,
                revision: view.revision,
                next_cursor,
            });
        }
        const PAGE_CANDIDATE_BUDGET: usize = 10_000;
        let result = self.execute_bounded_query(
            &QuerySpec {
                limit: PAGE_CANDIDATE_BUDGET,
                ..spec.clone()
            },
            view,
        )?;
        if result.truncated {
            return Err(Error::QueryBudgetExceeded);
        }
        let mut rows = result.rows;
        if let Some(cursor) = after {
            let position = rows
                .iter()
                .position(|row| row.object == cursor.last && row.record.label == cursor.label)
                .ok_or(Error::InvalidInput("query cursor boundary"))?;
            rows.drain(..=position);
        }
        let has_more = rows.len() > spec.limit;
        rows.truncate(spec.limit);
        let next_cursor = if has_more {
            rows.last().map(|last| QueryCursor {
                spec: spec.clone(),
                view,
                last: last.object,
                label: last.record.label.clone(),
            })
        } else {
            None
        };
        Ok(QueryPage {
            rows,
            revision: result.revision,
            next_cursor,
        })
    }

    fn execute_bounded_query(
        &self,
        spec: &QuerySpec,
        view: ViewContext,
    ) -> Result<QueryResult, Error> {
        self.validate_query_view(view)?;
        if matches!(
            &spec.filter,
            QueryFilter::Kind(_)
                | QueryFilter::ClassifiedAs(_)
                | QueryFilter::ClassifiedBy(_)
                | QueryFilter::SiteArea(_)
                | QueryFilter::HasSource(_)
                | QueryFilter::And(_)
                | QueryFilter::Or(_)
                | QueryFilter::Not(_)
        ) {
            let mut nodes = 0;
            validate_predicate(&spec.filter, view.world, 0, &mut nodes)?;
            let mut plan = PredicatePlan::default();
            let candidates = plan.collect(self, &spec.filter, view)?;
            let objects = match candidates {
                Some(ids) => ids
                    .into_iter()
                    .filter_map(|id| plan.rows.remove(&id))
                    .collect(),
                None => self.objects_at(view, 2_000)?,
            };
            let mut rows = Vec::new();
            for item in objects {
                self.check_query_control()?;
                if predicate_matches(&spec.filter, &item, &plan) {
                    rows.push(item);
                }
            }
            rows.sort_by(|a, b| {
                a.record
                    .label
                    .cmp(&b.record.label)
                    .then(a.object.id.0.cmp(&b.object.id.0))
            });
            let truncated = rows.len() > spec.limit;
            rows.truncate(spec.limit);
            return Ok(QueryResult {
                rows,
                truncated,
                revision: view.revision,
            });
        }
        match &spec.filter {
            QueryFilter::ExactAlias(label) => {
                return self.indexed_alias_at(label, view, spec.limit);
            }
            QueryFilter::FullTextPhrase(phrase) => {
                return self.indexed_phrase_at(phrase, view, spec.limit);
            }
            QueryFilter::ClaimFullTextPhrase(phrase) => {
                return self.indexed_claim_phrase_at(phrase, view, spec.limit);
            }
            QueryFilter::EventRole { event, role } => {
                return self.indexed_involvements_at(view, *event, Some(role), false, spec.limit);
            }
            QueryFilter::ParticipantEvents { participant, role } => {
                return self.indexed_involvements_at(
                    view,
                    *participant,
                    role.as_ref(),
                    true,
                    spec.limit,
                );
            }
            QueryFilter::WithinPlace { .. } => {
                return self.within_place(view, &spec.filter, spec.limit);
            }
            QueryFilter::RelationshipPath { .. } => {
                return self.indexed_relationship_path_at(view, &spec.filter, spec.limit);
            }
            QueryFilter::Backlinks(target) => {
                return self.indexed_backlinks_at(view, *target, spec.limit);
            }
            _ => {}
        }
        let objects = self.objects_at(view, 2_000)?;
        let mut rows: Vec<_> = match &spec.filter {
            QueryFilter::ExactAlias(_)
            | QueryFilter::FullTextPhrase(_)
            | QueryFilter::ClaimFullTextPhrase(_) => {
                unreachable!("handled by the indexed FTS5 query path")
            }
            QueryFilter::Kind(_)
            | QueryFilter::ClassifiedAs(_)
            | QueryFilter::ClassifiedBy(_)
            | QueryFilter::SiteArea(_)
            | QueryFilter::HasSource(_)
            | QueryFilter::And(_)
            | QueryFilter::Or(_)
            | QueryFilter::Not(_) => unreachable!("handled by the snapshot predicate path"),
            QueryFilter::LabelContains(text) | QueryFilter::ExactLabel(text) => {
                let needle = search_label(text)?;
                let exact = matches!(spec.filter, QueryFilter::ExactLabel(_));
                let mut matches = Vec::new();
                for item in objects {
                    self.check_query_control()?;
                    let label = item.record.label.to_lowercase();
                    if (exact && label == needle) || (!exact && label.contains(&needle)) {
                        matches.push(item);
                    }
                }
                matches
            }
            QueryFilter::Backlinks(_) => unreachable!("handled by indexed backlinks"),
            QueryFilter::EventRole { .. } | QueryFilter::ParticipantEvents { .. } => {
                unreachable!("handled by indexed event participation path")
            }
            QueryFilter::RelationshipPath { .. } => {
                unreachable!("handled by indexed relationship path")
            }
            QueryFilter::WithinPlace { .. } => unreachable!("handled by spatial traversal"),
        };
        rows.sort_by(|a, b| {
            a.record
                .label
                .cmp(&b.record.label)
                .then(a.object.id.0.cmp(&b.object.id.0))
        });
        let truncated = rows.len() > spec.limit;
        rows.truncate(spec.limit);
        Ok(QueryResult {
            rows,
            truncated,
            revision: view.revision,
        })
    }
}

const MAX_PREDICATE_NODES: usize = 32;
const MAX_PREDICATE_DEPTH: usize = 4;
const MAX_BOOLEAN_CHILDREN: usize = 8;

/// Validate all children before short-circuiting, including those under NOT.
pub(crate) fn validate_predicate(
    filter: &QueryFilter,
    world: WorldId,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), Error> {
    *nodes += 1;
    if depth > MAX_PREDICATE_DEPTH || *nodes > MAX_PREDICATE_NODES {
        return Err(Error::QueryBudgetExceeded);
    }
    match filter {
        QueryFilter::LabelContains(label)
        | QueryFilter::ExactLabel(label)
        | QueryFilter::ExactAlias(label)
        | QueryFilter::FullTextPhrase(label)
        | QueryFilter::ClaimFullTextPhrase(label) => {
            search_label(label)?;
        }
        QueryFilter::Kind(_) | QueryFilter::ClassifiedAs(_) => {}
        QueryFilter::SiteArea(test) => test.validate()?,
        QueryFilter::ClassifiedBy(descriptor) => {
            if descriptor.world != world {
                return Err(Error::WrongWorld);
            }
        }
        QueryFilter::HasSource(source) => {
            if source.world != world {
                return Err(Error::WrongWorld);
            }
        }
        QueryFilter::And(children) | QueryFilter::Or(children) => {
            if children.len() > MAX_BOOLEAN_CHILDREN {
                return Err(Error::QueryBudgetExceeded);
            }
            if children.len() < 2 {
                return Err(Error::InvalidInput("boolean predicate children"));
            }
            for child in children {
                validate_predicate(child, world, depth + 1, nodes)?;
            }
        }
        QueryFilter::Not(inner) => validate_predicate(inner, world, depth + 1, nodes)?,
        _ => return Err(Error::InvalidInput("unsupported nested query predicate")),
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum IndexedTextKind {
    Label,
    Claim,
    Alias,
}

fn indexed_text_kind(filter: &QueryFilter) -> IndexedTextKind {
    match filter {
        QueryFilter::FullTextPhrase(_) => IndexedTextKind::Label,
        QueryFilter::ClaimFullTextPhrase(_) => IndexedTextKind::Claim,
        QueryFilter::ExactAlias(_) => IndexedTextKind::Alias,
        _ => unreachable!("not an indexed text predicate"),
    }
}

/// Indexed leaves produce exact historical identity sets. A positive indexed
/// conjunct bounds an AND; an OR is bounded only if every arm is bounded.
/// NOT never supplies a candidate universe. All leaves are collected before
/// evaluating predicates, so short-circuiting cannot conceal failing reads.
#[derive(Default)]
struct PredicatePlan<'a> {
    indexed: HashMap<(IndexedTextKind, &'a str), HashSet<ObjectRef>>,
    rows: HashMap<ObjectRef, Version>,
    sources: HashSet<ObjectRef>,
    profiles: HashMap<ConceptProfile, HashSet<ObjectRef>>,
    descriptors: HashMap<ObjectRef, HashSet<ObjectRef>>,
    area: HashMap<AreaTest, HashSet<ObjectRef>>,
    matched: usize,
}

impl<'a> PredicatePlan<'a> {
    fn collect(
        &mut self,
        store: &Store,
        filter: &'a QueryFilter,
        view: ViewContext,
    ) -> Result<Option<HashSet<ObjectRef>>, Error> {
        store.check_query_control()?;
        const INDEX_BUDGET: usize = 10_000;
        match filter {
            QueryFilter::FullTextPhrase(text)
            | QueryFilter::ClaimFullTextPhrase(text)
            | QueryFilter::ExactAlias(text) => {
                let key = (indexed_text_kind(filter), text.as_str());
                if let Some(ids) = self.indexed.get(&key) {
                    return Ok(Some(ids.clone()));
                }
                let result = match key.0 {
                    IndexedTextKind::Label => store.indexed_phrase_at(text, view, INDEX_BUDGET)?,
                    IndexedTextKind::Claim => {
                        store.indexed_claim_phrase_at(text, view, INDEX_BUDGET)?
                    }
                    IndexedTextKind::Alias => store.indexed_alias_at(text, view, INDEX_BUDGET)?,
                };
                self.matched += result.rows.len();
                if result.truncated || self.matched > INDEX_BUDGET {
                    return Err(Error::QueryBudgetExceeded);
                }
                let ids: HashSet<_> = result.rows.iter().map(|row| row.object).collect();
                for row in result.rows {
                    self.rows.insert(row.object, row);
                }
                self.indexed.insert(key, ids.clone());
                Ok(Some(ids))
            }
            QueryFilter::SiteArea(test) => {
                if let Some(ids) = self.area.get(test) {
                    return Ok(Some(ids.clone()));
                }
                let versions = store.site_area_matches(view, *test)?;
                self.matched += versions.len();
                if self.matched > INDEX_BUDGET {
                    return Err(Error::QueryBudgetExceeded);
                }
                let ids = versions
                    .iter()
                    .map(|version| version.object)
                    .collect::<HashSet<_>>();
                for version in versions {
                    self.rows.insert(version.object, version);
                }
                self.area.insert(*test, ids.clone());
                Ok(Some(ids))
            }
            QueryFilter::ClassifiedBy(descriptor) => {
                if let Some(ids) = self.descriptors.get(descriptor) {
                    return Ok(Some(ids.clone()));
                }
                let assignments = store.classifications_by_descriptor(view, *descriptor)?;
                self.matched += assignments.len();
                if self.matched > INDEX_BUDGET {
                    return Err(Error::QueryBudgetExceeded);
                }
                let mut ids = HashSet::new();
                for assignment in assignments {
                    store.check_query_control()?;
                    let object = assignment.input.subject;
                    if let Some(row) = store.get_object(object, view)? {
                        ids.insert(object);
                        self.rows.insert(object, row);
                    }
                }
                self.descriptors.insert(*descriptor, ids.clone());
                Ok(Some(ids))
            }
            QueryFilter::ClassifiedAs(profile) => {
                if let Some(ids) = self.profiles.get(profile) {
                    return Ok(Some(ids.clone()));
                }
                let assignments = store.classifications(view, None, Some(*profile))?;
                self.matched += assignments.len();
                if self.matched > INDEX_BUDGET {
                    return Err(Error::QueryBudgetExceeded);
                }
                let mut ids = HashSet::new();
                for assignment in assignments {
                    store.check_query_control()?;
                    let object = assignment.input.subject;
                    if let Some(row) = store.get_object(object, view)? {
                        ids.insert(object);
                        self.rows.insert(object, row);
                    }
                }
                self.profiles.insert(*profile, ids.clone());
                Ok(Some(ids))
            }
            QueryFilter::HasSource(source) => {
                if store.get_object(*source, view)?.is_some() {
                    self.sources.insert(*source);
                }
                Ok(None)
            }
            QueryFilter::And(children) => {
                let mut result: Option<HashSet<ObjectRef>> = None;
                for child in children {
                    if let Some(ids) = self.collect(store, child, view)? {
                        if let Some(existing) = &mut result {
                            existing.retain(|id| ids.contains(id));
                        } else {
                            result = Some(ids);
                        }
                    }
                }
                Ok(result)
            }
            QueryFilter::Or(children) => {
                let mut result = Some(HashSet::new());
                for child in children {
                    result = match (result, self.collect(store, child, view)?) {
                        (Some(mut existing), Some(ids)) => {
                            existing.extend(ids);
                            Some(existing)
                        }
                        _ => None,
                    };
                }
                Ok(result)
            }
            QueryFilter::Not(child) => {
                let _ = self.collect(store, child, view)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

fn predicate_matches(filter: &QueryFilter, version: &Version, plan: &PredicatePlan<'_>) -> bool {
    match filter {
        QueryFilter::LabelContains(text) => version
            .record
            .label
            .to_lowercase()
            .contains(&text.trim().to_lowercase()),
        QueryFilter::ExactLabel(text) => {
            version.record.label.to_lowercase() == text.trim().to_lowercase()
        }
        QueryFilter::Kind(kind) => ObjectKindFilter::from(&version.record.data) == *kind,
        QueryFilter::SiteArea(test) => plan
            .area
            .get(test)
            .is_some_and(|ids| ids.contains(&version.object)),
        QueryFilter::ClassifiedBy(descriptor) => plan
            .descriptors
            .get(descriptor)
            .is_some_and(|ids| ids.contains(&version.object)),
        QueryFilter::ClassifiedAs(profile) => plan
            .profiles
            .get(profile)
            .is_some_and(|ids| ids.contains(&version.object)),
        QueryFilter::FullTextPhrase(text)
        | QueryFilter::ClaimFullTextPhrase(text)
        | QueryFilter::ExactAlias(text) => plan
            .indexed
            .get(&(indexed_text_kind(filter), text.as_str()))
            .is_some_and(|ids| ids.contains(&version.object)),
        QueryFilter::HasSource(source) => {
            plan.sources.contains(source) && version.record.source == Some(*source)
        }
        QueryFilter::And(children) => children
            .iter()
            .all(|child| predicate_matches(child, version, plan)),
        QueryFilter::Or(children) => children
            .iter()
            .any(|child| predicate_matches(child, version, plan)),
        QueryFilter::Not(inner) => !predicate_matches(inner, version, plan),
        _ => unreachable!("the predicate validator rejects unsupported leaf filters"),
    }
}

pub(crate) fn refers_to(data: &ObjectData, target: ObjectRef) -> bool {
    match data {
        ObjectData::Relationship {
            subject,
            target: other,
            ..
        } => *subject == target || *other == target,
        ObjectData::EventInvolvement {
            event,
            participant,
            role,
        } => *event == target || *participant == target || *role == EventRole::Custom(target),
        ObjectData::ActorAction {
            event,
            actor,
            target: other,
            ..
        } => *event == target || *actor == target || *other == Some(target),
        ObjectData::Claim { subject, .. } => *subject == target,
        _ => false,
    }
}

pub(crate) fn search_label(raw: &str) -> Result<String, Error> {
    let text = raw.trim();
    if text.is_empty() || text.len() > 800 {
        return Err(Error::InvalidInput("query label"));
    }
    Ok(text.to_lowercase())
}
