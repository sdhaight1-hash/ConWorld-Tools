//! Phase 3 typed reads over a single authorized historical snapshot.
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::graph::walk;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        if matches!(
            &spec.filter,
            QueryFilter::Kind(_)
                | QueryFilter::HasSource(_)
                | QueryFilter::And(_)
                | QueryFilter::Or(_)
                | QueryFilter::Not(_)
        ) {
            let mut nodes = 0;
            validate_predicate(&spec.filter, view, 0, &mut nodes)?;
            let objects = self.objects_at(view, 2_000)?;
            let visible: HashSet<_> = objects.iter().map(|item| item.object).collect();
            let mut rows: Vec<_> = objects
                .into_iter()
                .filter(|item| predicate_matches(&spec.filter, item, &visible))
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
            _ => {}
        }
        let objects = self.objects_at(view, 2_000)?;
        let visible: HashSet<_> = objects.iter().map(|version| version.object).collect();
        let mut truncated = false;
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
            QueryFilter::EventRole { event, role } => {
                same_world(*event, view)?;
                if let EventRole::Custom(definition) = role {
                    same_world(*definition, view)?;
                }
                objects.into_iter().filter(|item| matches!(&item.record.data,
                    ObjectData::EventInvolvement { event: candidate, participant, role: candidate_role }
                        if candidate == event && candidate_role == role
                            && visible.contains(event) && visible.contains(participant)
                )).collect()
            }
            QueryFilter::ParticipantEvents { participant, role } => {
                same_world(*participant, view)?;
                if let Some(EventRole::Custom(definition)) = role {
                    same_world(*definition, view)?;
                }
                let person_visible = objects.iter().any(|item| {
                    item.object == *participant && matches!(&item.record.data, ObjectData::Person)
                });
                if !person_visible {
                    Vec::new()
                } else {
                    let events: HashSet<_> = objects
                        .iter()
                        .filter_map(|item| match &item.record.data {
                            ObjectData::EventInvolvement {
                                event,
                                participant: member,
                                role: recorded,
                            } if member == participant
                                && role.is_none_or(|wanted| wanted == *recorded)
                                && visible.contains(event) =>
                            {
                                Some(*event)
                            }
                            _ => None,
                        })
                        .collect();
                    objects
                        .into_iter()
                        .filter(|item| {
                            events.contains(&item.object)
                                && matches!(&item.record.data, ObjectData::HistoricalEvent { .. })
                        })
                        .collect()
                }
            }
            QueryFilter::RelationshipPath {
                start,
                kind,
                direction,
                max_depth,
                max_visited,
            } => {
                same_world(*start, view)?;
                let kind = search_label(kind)?;
                if !(1..=16).contains(max_depth) || !(1..=2_000).contains(max_visited) {
                    return Err(Error::InvalidInput("relationship traversal budget"));
                }
                let mut edges = HashMap::<ObjectRef, Vec<ObjectRef>>::new();
                for item in &objects {
                    if let ObjectData::Relationship {
                        subject,
                        target,
                        kind: label,
                        ..
                    } = &item.record.data
                        && label.to_lowercase() == kind
                        && visible.contains(subject)
                        && visible.contains(target)
                    {
                        if *direction != Direction::Incoming {
                            edges.entry(*subject).or_default().push(*target);
                        }
                        if *direction != Direction::Outgoing {
                            edges.entry(*target).or_default().push(*subject);
                        }
                    }
                }
                let (visited, limited) = if visible.contains(start) {
                    walk(*start, &mut edges, *max_depth, *max_visited)
                } else {
                    (HashSet::new(), false)
                };
                truncated = limited;
                objects
                    .into_iter()
                    .filter(|item| item.object != *start && visited.contains(&item.object))
                    .collect()
            }
        };
        rows.sort_by(|a, b| {
            a.record
                .label
                .cmp(&b.record.label)
                .then(a.object.id.0.cmp(&b.object.id.0))
        });
        truncated |= rows.len() > spec.limit;
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
fn validate_predicate(
    filter: &QueryFilter,
    view: ViewContext,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), Error> {
    *nodes += 1;
    if depth > MAX_PREDICATE_DEPTH || *nodes > MAX_PREDICATE_NODES {
        return Err(Error::QueryBudgetExceeded);
    }
    match filter {
        QueryFilter::LabelContains(label) | QueryFilter::ExactLabel(label) => {
            search_label(label)?;
        }
        QueryFilter::Kind(_) => {}
        QueryFilter::HasSource(source) => same_world(*source, view)?,
        QueryFilter::And(children) | QueryFilter::Or(children) => {
            if children.len() > MAX_BOOLEAN_CHILDREN {
                return Err(Error::QueryBudgetExceeded);
            }
            if children.len() < 2 {
                return Err(Error::InvalidInput("boolean predicate children"));
            }
            for child in children {
                validate_predicate(child, view, depth + 1, nodes)?;
            }
        }
        QueryFilter::Not(inner) => validate_predicate(inner, view, depth + 1, nodes)?,
        _ => return Err(Error::InvalidInput("unsupported nested query predicate")),
    }
    Ok(())
}

fn predicate_matches(
    filter: &QueryFilter,
    version: &Version,
    visible: &HashSet<ObjectRef>,
) -> bool {
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
        QueryFilter::HasSource(source) => {
            visible.contains(source) && version.record.source == Some(*source)
        }
        QueryFilter::And(children) => children
            .iter()
            .all(|child| predicate_matches(child, version, visible)),
        QueryFilter::Or(children) => children
            .iter()
            .any(|child| predicate_matches(child, version, visible)),
        QueryFilter::Not(inner) => !predicate_matches(inner, version, visible),
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

fn search_label(raw: &str) -> Result<String, Error> {
    let text = raw.trim();
    if text.is_empty() || text.len() > 800 {
        return Err(Error::InvalidInput("query label"));
    }
    Ok(text.to_lowercase())
}
