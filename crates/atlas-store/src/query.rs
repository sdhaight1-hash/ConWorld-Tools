//! Phase 3 typed reads over a single authorized historical snapshot.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    AuthorRevision, Error, EventRole, ObjectData, ObjectRef, Scope, Store, Version, ViewContext,
    WorldId,
};

pub const QUERY_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuerySpec {
    pub schema_version: u16,
    pub filter: QueryFilter,
    pub limit: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryFilter {
    LabelContains(String),
    ExactLabel(String),
    /// Literal token phrase over versioned object labels, indexed with FTS5.
    FullTextPhrase(String),
    /// Literal token phrase in versioned, source-backed claim statements.
    ClaimFullTextPhrase(String),
    Kind(ObjectKindFilter),
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
    Definition,
    Relationship,
    EventInvolvement,
    ActorAction,
    HistoricalEvent,
    Source,
    Claim,
}

/// Versioned, allowlisted wire/persistence shape for composite predicates.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum PredicateInput {
    Contains { text: String },
    Exact { text: String },
    FullText { text: String },
    ClaimFullText { text: String },
    ObjectKind { kind: ObjectKindFilter },
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
            Self::FullText { text } => QueryFilter::FullTextPhrase(text),
            Self::ClaimFullText { text } => QueryFilter::ClaimFullTextPhrase(text),
            Self::ObjectKind { kind } => QueryFilter::Kind(kind),
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
            QueryFilter::FullTextPhrase(text) => Self::FullText { text: text.clone() },
            QueryFilter::ClaimFullTextPhrase(text) => Self::ClaimFullText { text: text.clone() },
            QueryFilter::Kind(kind) => Self::ObjectKind { kind: *kind },
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub spec: QuerySpec,
    pub recorded_at: AuthorRevision,
}

#[derive(Clone, Debug)]
pub struct QueryResult {
    pub rows: Vec<Version>,
    pub revision: AuthorRevision,
    pub truncated: bool,
}

impl Store {
    pub fn execute_query(&self, spec: &QuerySpec, view: ViewContext) -> Result<QueryResult, Error> {
        if view.scope != crate::Scope::Author {
            return Err(Error::CapabilityDenied);
        }
        if spec.schema_version != QUERY_SCHEMA_VERSION || !(1..=500).contains(&spec.limit) {
            return Err(Error::InvalidInput("query schema or result limit"));
        }
        self.validate_query_view(view)?;
        if matches!(
            &spec.filter,
            QueryFilter::Kind(_)
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
            let mut rows: Vec<_> = objects
                .into_iter()
                .filter(|item| predicate_matches(&spec.filter, item, &plan))
                .collect();
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
            QueryFilter::RelationshipPath { .. } => {
                return self.indexed_relationship_path_at(view, &spec.filter, spec.limit);
            }
            _ => {}
        }
        let objects = self.objects_at(view, 2_000)?;
        let visible: HashSet<_> = objects.iter().map(|version| version.object).collect();
        let mut rows: Vec<_> = match &spec.filter {
            QueryFilter::FullTextPhrase(_) | QueryFilter::ClaimFullTextPhrase(_) => {
                unreachable!("handled by the indexed FTS5 query path")
            }
            QueryFilter::Kind(_)
            | QueryFilter::HasSource(_)
            | QueryFilter::And(_)
            | QueryFilter::Or(_)
            | QueryFilter::Not(_) => unreachable!("handled by the snapshot predicate path"),
            QueryFilter::LabelContains(text) | QueryFilter::ExactLabel(text) => {
                let needle = search_label(text)?;
                let exact = matches!(spec.filter, QueryFilter::ExactLabel(_));
                objects
                    .into_iter()
                    .filter(|item| {
                        let label = item.record.label.to_lowercase();
                        if exact {
                            label == needle
                        } else {
                            label.contains(&needle)
                        }
                    })
                    .collect()
            }
            QueryFilter::Backlinks(target) => {
                same_world(*target, view)?;
                let approved = if visible.contains(target) {
                    self.approved_mention_sources_at(view, *target)?
                } else {
                    HashSet::new()
                };
                objects
                    .into_iter()
                    .filter(|item| {
                        visible.contains(target)
                            && (item.record.source == Some(*target)
                                || refers_to(&item.record.data, *target)
                                || approved.contains(&item.object))
                    })
                    .collect()
            }
            QueryFilter::EventRole { .. } | QueryFilter::ParticipantEvents { .. } => {
                unreachable!("handled by indexed event participation path")
            }
            QueryFilter::RelationshipPath { .. } => {
                unreachable!("handled by indexed relationship path")
            }
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
        | QueryFilter::FullTextPhrase(label)
        | QueryFilter::ClaimFullTextPhrase(label) => {
            search_label(label)?;
        }
        QueryFilter::Kind(_) => {}
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

/// Indexed leaves produce exact historical identity sets. A positive indexed
/// conjunct bounds an AND; an OR is bounded only if every arm is bounded.
/// NOT never supplies a candidate universe. All leaves are collected before
/// evaluating predicates, so short-circuiting cannot conceal failing reads.
#[derive(Default)]
struct PredicatePlan<'a> {
    indexed: HashMap<(bool, &'a str), HashSet<ObjectRef>>,
    rows: HashMap<ObjectRef, Version>,
    sources: HashSet<ObjectRef>,
    matched: usize,
}

impl<'a> PredicatePlan<'a> {
    fn collect(
        &mut self,
        store: &Store,
        filter: &'a QueryFilter,
        view: ViewContext,
    ) -> Result<Option<HashSet<ObjectRef>>, Error> {
        const INDEX_BUDGET: usize = 10_000;
        match filter {
            QueryFilter::FullTextPhrase(text) | QueryFilter::ClaimFullTextPhrase(text) => {
                let claims_only = matches!(filter, QueryFilter::ClaimFullTextPhrase(_));
                let key = (claims_only, text.as_str());
                if let Some(ids) = self.indexed.get(&key) {
                    return Ok(Some(ids.clone()));
                }
                let result = if claims_only {
                    store.indexed_claim_phrase_at(text, view, INDEX_BUDGET)?
                } else {
                    store.indexed_phrase_at(text, view, INDEX_BUDGET)?
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
        QueryFilter::Kind(kind) => matches!(
            (&version.record.data, kind),
            (ObjectData::Person, ObjectKindFilter::Person)
                | (ObjectData::Definition { .. }, ObjectKindFilter::Definition)
                | (
                    ObjectData::Relationship { .. },
                    ObjectKindFilter::Relationship
                )
                | (
                    ObjectData::EventInvolvement { .. },
                    ObjectKindFilter::EventInvolvement
                )
                | (
                    ObjectData::ActorAction { .. },
                    ObjectKindFilter::ActorAction
                )
                | (
                    ObjectData::HistoricalEvent { .. },
                    ObjectKindFilter::HistoricalEvent
                )
                | (ObjectData::Source { .. }, ObjectKindFilter::Source)
                | (ObjectData::Claim { .. }, ObjectKindFilter::Claim)
        ),
        QueryFilter::FullTextPhrase(text) | QueryFilter::ClaimFullTextPhrase(text) => {
            let claims_only = matches!(filter, QueryFilter::ClaimFullTextPhrase(_));
            plan.indexed
                .get(&(claims_only, text.as_str()))
                .is_some_and(|ids| ids.contains(&version.object))
        }
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

fn refers_to(data: &ObjectData, target: ObjectRef) -> bool {
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

fn same_world(object: ObjectRef, view: ViewContext) -> Result<(), Error> {
    if object.world == view.world {
        Ok(())
    } else {
        Err(Error::WrongWorld)
    }
}

pub(crate) fn search_label(raw: &str) -> Result<String, Error> {
    let text = raw.trim();
    if text.is_empty() || text.len() > 800 {
        return Err(Error::InvalidInput("query label"));
    }
    Ok(text.to_lowercase())
}
