//! Phase 3 typed reads over a single authorized historical snapshot.
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::graph::walk;
use crate::{AuthorRevision, Error, EventRole, ObjectData, ObjectRef, Store, Version, ViewContext};

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
    Backlinks(ObjectRef),
    EventRole {
        event: ObjectRef,
        role: EventRole,
    },
    RelationshipPath {
        start: ObjectRef,
        kind: String,
        direction: Direction,
        max_depth: u8,
        max_visited: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Outgoing,
    Incoming,
    Both,
}

/// Author-owned, versioned definition. Running it always reevaluates the
/// QuerySpec with the caller's current ViewContext; no results are persisted.
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
        if let QueryFilter::FullTextPhrase(phrase) = &spec.filter {
            return self.indexed_phrase_at(phrase, view, spec.limit);
        }
        let objects = self.objects_at(view, 2_000)?;
        let visible: HashSet<_> = objects.iter().map(|version| version.object).collect();
        let mut truncated = false;
        let mut rows: Vec<_> = match &spec.filter {
            QueryFilter::FullTextPhrase(_) => unreachable!("handled by the FTS5 query path"),
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
                objects
                    .into_iter()
                    .filter(|item| {
                        visible.contains(target)
                            && (item.record.source == Some(*target)
                                || refers_to(&item.record.data, *target))
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
