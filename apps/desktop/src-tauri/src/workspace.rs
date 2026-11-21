//! Minimal persistent author workspace for Phase Two.
//! Canonical records live only in atlas-store; the Northbridge fixture is separate.
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use atlas_store::{
    ActionOutcome, Change, Command, EventDate, EventRole, ObjectData, ObjectId, ObjectRef, Record,
    Scope, Store, ValidInterval, ViewContext, World, WorldDay, WorldId,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

pub struct AuthorWorkspace {
    store: Mutex<Store>,
}

#[derive(Serialize)]
pub struct WorldSummary {
    id: String,
    name: String,
    revision: i64,
}

#[derive(Serialize)]
pub struct PersonSummary {
    id: String,
    label: String,
}

#[derive(Serialize)]
pub struct RelationshipSummary {
    id: String,
    subject_id: String,
    target_id: String,
    kind: String,
    trust: Option<u8>,
    tags: Vec<String>,
    starts_at: Option<i64>,
    cause: Option<CauseSummary>,
}

#[derive(Serialize)]
pub struct CauseSummary {
    day: i64,
    event: String,
    actor: Option<String>,
    action: Option<String>,
    intent: Option<String>,
    outcome: Option<&'static str>,
    source: String,
}

#[derive(Serialize)]
pub struct WorkspaceSnapshot {
    world: WorldSummary,
    day: i64,
    people: Vec<PersonSummary>,
    relationships: Vec<RelationshipSummary>,
}

#[derive(Deserialize)]
pub struct RelationshipInput {
    world_id: String,
    subject_id: String,
    target_id: String,
    kind: String,
    trust: Option<u8>,
    tags: String,
    day: i64,
}

#[derive(Deserialize)]
pub struct EventInput {
    world_id: String,
    relationship_id: String,
    actor_id: String,
    day: i64,
    event_label: String,
    action: String,
    intent: String,
    outcome: String,
    source_citation: String,
    trust: Option<u8>,
    tags: String,
}

fn id(raw: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw).map_err(|_| "Invalid world or object identifier".into())
}

fn nonempty(value: &str, field: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 200 {
        return Err(format!("{field} must contain between 1 and 200 characters"));
    }
    Ok(value.into())
}

fn tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn reference(world: WorldId, raw: &str) -> Result<ObjectRef, String> {
    Ok(ObjectRef {
        world,
        id: ObjectId(id(raw)?),
    })
}

fn view(world: &World, day: i64) -> ViewContext {
    ViewContext {
        world: world.id,
        branch: world.main,
        revision: world.revision,
        day: WorldDay(day),
        scope: Scope::Author,
    }
}

fn summary(world: World) -> WorldSummary {
    WorldSummary {
        id: world.id.0.to_string(),
        name: world.name,
        revision: world.revision.0,
    }
}

fn commit(store: &mut Store, world: &World, changes: Vec<Change>) -> Result<(), String> {
    store
        .apply(&Command {
            world: world.id,
            branch: world.main,
            expected: world.revision,
            id: Uuid::new_v4(),
            author: "Local author".into(),
            rationale: "Phase Two authoring edit".into(),
            scope: Scope::Author,
            changes,
        })
        .map(|_| ())
        .map_err(|error| error.to_string())
}

impl AuthorWorkspace {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, atlas_store::Error> {
        Ok(Self {
            store: Mutex::new(Store::open(path)?),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Store>, String> {
        self.store
            .lock()
            .map_err(|_| "Author workspace storage lock was poisoned".into())
    }

    fn worlds(&self) -> Result<Vec<WorldSummary>, String> {
        self.lock()?
            .worlds(Scope::Author)
            .map(|worlds| worlds.into_iter().map(summary).collect())
            .map_err(|error| error.to_string())
    }

    fn create_world(&self, name: &str) -> Result<WorldSummary, String> {
        let name = nonempty(name, "World name")?;
        self.lock()?
            .create_world(&name, Scope::Author)
            .map(summary)
            .map_err(|error| error.to_string())
    }

    fn snapshot(&self, world_id: &str, day: i64) -> Result<WorkspaceSnapshot, String> {
        let store = self.lock()?;
        let world = store
            .world(WorldId(id(world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let versions = store
            .objects_at(view(&world, day), 2_000)
            .map_err(|error| error.to_string())?;
        let mut people = Vec::new();
        let mut relationships = Vec::new();
        for version in versions {
            let object_id = version.object.id.0.to_string();
            match version.record.data {
                ObjectData::Person => people.push(PersonSummary {
                    id: object_id,
                    label: version.record.label,
                }),
                ObjectData::Relationship {
                    subject,
                    target,
                    kind,
                    trust,
                    tags,
                } => relationships.push(RelationshipSummary {
                    cause: Self::explain(
                        &store,
                        &world,
                        version.object,
                        version.record.source,
                        day,
                    )?,
                    id: object_id,
                    subject_id: subject.id.0.to_string(),
                    target_id: target.id.0.to_string(),
                    kind,
                    trust,
                    tags,
                    starts_at: version.record.valid.start.map(|day| day.0),
                }),
                _ => {}
            }
        }
        people.sort_by(|a, b| a.label.cmp(&b.label).then(a.id.cmp(&b.id)));
        relationships.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.id.cmp(&b.id)));
        Ok(WorkspaceSnapshot {
            world: summary(world),
            day,
            people,
            relationships,
        })
    }

    fn explain(
        store: &Store,
        world: &World,
        bond: ObjectRef,
        source: Option<ObjectRef>,
        day: i64,
    ) -> Result<Option<CauseSummary>, String> {
        let context = view(world, day);
        let Some(effect) = store
            .effect_on(bond, context)
            .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        let event = store
            .get_object(effect.event.object, context)
            .map_err(|error| error.to_string())?
            .ok_or("Source event is not available at this historical day")?;
        let mut actor = None;
        let mut action = None;
        let mut intent = None;
        let mut outcome = None;
        if let Some(link) = effect.action {
            let version = store
                .get_object(link.object, context)
                .map_err(|error| error.to_string())?
                .ok_or("Source actor action is not available at this historical day")?;
            if let ObjectData::ActorAction {
                actor: actor_id,
                action: action_name,
                intent: action_intent,
                outcome: action_outcome,
                ..
            } = version.record.data
            {
                actor = Some(
                    store
                        .get_object(actor_id, context)
                        .map_err(|error| error.to_string())?
                        .ok_or("Event actor is unavailable")?
                        .record
                        .label,
                );
                action = Some(action_name);
                intent = Some(action_intent);
                outcome = Some(match action_outcome {
                    ActionOutcome::Attempted => "attempted",
                    ActionOutcome::Succeeded => "succeeded",
                    ActionOutcome::Failed => "failed",
                });
            } else {
                return Err("Effect references an object that is not an actor action".into());
            }
        }
        let source = if let Some(source) = source {
            let source = store
                .get_object(source, context)
                .map_err(|error| error.to_string())?
                .ok_or("Event source is unavailable")?;
            if let ObjectData::Source { citation } = source.record.data {
                citation
            } else {
                return Err("Event evidence does not reference a source".into());
            }
        } else {
            "No source cited".into()
        };
        Ok(Some(CauseSummary {
            day: effect.day.0,
            event: event.record.label,
            actor,
            action,
            intent,
            outcome,
            source,
        }))
    }

    fn create_person(&self, world_id: &str, name: &str) -> Result<(), String> {
        let name = nonempty(name, "Person name")?;
        let mut store = self.lock()?;
        let world = store
            .world(WorldId(id(world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let person = ObjectRef::new(world.id);
        commit(
            &mut store,
            &world,
            vec![Change::Create(
                person,
                Record {
                    label: name,
                    valid: ValidInterval::default(),
                    data: ObjectData::Person,
                    source: None,
                },
            )],
        )
    }

    fn create_relationship(&self, request: RelationshipInput) -> Result<(), String> {
        let kind = nonempty(&request.kind, "Relationship kind")?;
        let mut store = self.lock()?;
        let world = store
            .world(WorldId(id(&request.world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let subject = reference(world.id, &request.subject_id)?;
        let target = reference(world.id, &request.target_id)?;
        commit(
            &mut store,
            &world,
            vec![Change::Create(
                ObjectRef::new(world.id),
                Record {
                    label: format!("{kind}: {} → {}", subject.id.0, target.id.0),
                    valid: ValidInterval {
                        start: Some(WorldDay(request.day)),
                        end_exclusive: None,
                    },
                    data: ObjectData::Relationship {
                        subject,
                        target,
                        kind,
                        trust: request.trust,
                        tags: tags(&request.tags),
                    },
                    source: None,
                },
            )],
        )
    }

    fn record_event(&self, request: EventInput) -> Result<(), String> {
        let event_label = nonempty(&request.event_label, "Event name")?;
        let action_kind = nonempty(&request.action, "Actor action")?;
        let intent = nonempty(&request.intent, "Actor intent")?;
        let citation = nonempty(&request.source_citation, "Source citation")?;
        let outcome = match request.outcome.as_str() {
            "attempted" => ActionOutcome::Attempted,
            "succeeded" => ActionOutcome::Succeeded,
            "failed" => ActionOutcome::Failed,
            _ => return Err("Invalid action outcome".into()),
        };
        let mut store = self.lock()?;
        let world = store
            .world(WorldId(id(&request.world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let relationship = reference(world.id, &request.relationship_id)?;
        let actor = reference(world.id, &request.actor_id)?;
        let prior = store
            .get_object(relationship, view(&world, request.day))
            .map_err(|error| error.to_string())?
            .ok_or("Relationship does not exist at that date")?;
        let ObjectData::Relationship {
            subject,
            target,
            kind,
            ..
        } = prior.record.data
        else {
            return Err("Selected object is not a relationship".into());
        };
        let source = ObjectRef::new(world.id);
        let event = ObjectRef::new(world.id);
        let involvement = ObjectRef::new(world.id);
        let action = ObjectRef::new(world.id);
        let day = WorldDay(request.day);
        let occurrence = ValidInterval {
            start: Some(day),
            end_exclusive: None,
        };
        let sourced = |label: String, data| Record {
            label,
            valid: occurrence,
            data,
            source: Some(source),
        };
        commit(
            &mut store,
            &world,
            vec![
                Change::Create(
                    source,
                    Record {
                        label: format!("{event_label} source"),
                        valid: ValidInterval::default(),
                        data: ObjectData::Source { citation },
                        source: None,
                    },
                ),
                Change::Create(
                    event,
                    sourced(
                        event_label.clone(),
                        ObjectData::HistoricalEvent {
                            occurrence: EventDate::Exact(day),
                        },
                    ),
                ),
                Change::Create(
                    involvement,
                    sourced(
                        "Actor involvement".into(),
                        ObjectData::EventInvolvement {
                            event,
                            participant: actor,
                            role: EventRole::Actor,
                        },
                    ),
                ),
                Change::Create(
                    action,
                    sourced(
                        action_kind.clone(),
                        ObjectData::ActorAction {
                            event,
                            actor,
                            target: Some(subject),
                            action: action_kind,
                            intent,
                            outcome,
                        },
                    ),
                ),
                Change::ActionEffect {
                    event,
                    action,
                    target: relationship,
                    record: sourced(
                        format!("Relationship changed: {event_label}"),
                        ObjectData::Relationship {
                            subject,
                            target,
                            kind,
                            trust: request.trust,
                            tags: tags(&request.tags),
                        },
                    ),
                },
            ],
        )
    }
}

#[tauri::command]
pub fn workspace_worlds(
    workspace: State<'_, AuthorWorkspace>,
) -> Result<Vec<WorldSummary>, String> {
    workspace.worlds()
}

#[tauri::command]
pub fn workspace_create_world(
    workspace: State<'_, AuthorWorkspace>,
    name: String,
) -> Result<WorldSummary, String> {
    workspace.create_world(&name)
}

#[tauri::command]
pub fn workspace_snapshot(
    workspace: State<'_, AuthorWorkspace>,
    world_id: String,
    day: i64,
) -> Result<WorkspaceSnapshot, String> {
    workspace.snapshot(&world_id, day)
}

#[tauri::command]
pub fn workspace_create_person(
    workspace: State<'_, AuthorWorkspace>,
    world_id: String,
    name: String,
) -> Result<(), String> {
    workspace.create_person(&world_id, &name)
}

#[tauri::command]
pub fn workspace_create_relationship(
    workspace: State<'_, AuthorWorkspace>,
    request: RelationshipInput,
) -> Result<(), String> {
    workspace.create_relationship(request)
}

#[tauri::command]
pub fn workspace_record_event(
    workspace: State<'_, AuthorWorkspace>,
    request: EventInput,
) -> Result<(), String> {
    workspace.record_event(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn author_workspace_reopens_with_event_attributed_relationship_history()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let database = directory.path().join("author.sqlite");
        let world_id;
        let alice;
        let beatrice;
        let relationship;
        {
            let workspace = AuthorWorkspace::open(&database)?;
            let world = workspace.create_world("Archive")?;
            world_id = world.id;
            workspace.create_person(&world_id, "Alice")?;
            workspace.create_person(&world_id, "Beatrice")?;
            let people = workspace.snapshot(&world_id, 0)?.people;
            alice = people
                .iter()
                .find(|p| p.label == "Alice")
                .unwrap()
                .id
                .clone();
            beatrice = people
                .iter()
                .find(|p| p.label == "Beatrice")
                .unwrap()
                .id
                .clone();
            workspace.create_relationship(RelationshipInput {
                world_id: world_id.clone(),
                subject_id: alice.clone(),
                target_id: beatrice.clone(),
                kind: "friendship".into(),
                trust: Some(90),
                tags: "friends".into(),
                day: 0,
            })?;
            relationship = workspace.snapshot(&world_id, 0)?.relationships[0]
                .id
                .clone();
            workspace.record_event(EventInput {
                world_id: world_id.clone(),
                relationship_id: relationship.clone(),
                actor_id: beatrice.clone(),
                day: 10,
                event_label: "Disclosed letter".into(),
                action: "disclose_letter".into(),
                intent: "force a succession dispute".into(),
                outcome: "succeeded".into(),
                source_citation: "Northbridge folio 14".into(),
                trust: Some(12),
                tags: "estranged".into(),
            })?;
            // A second event cannot rewrite the state on the same fictional day.
            assert!(
                workspace
                    .record_event(EventInput {
                        world_id: world_id.clone(),
                        relationship_id: relationship.clone(),
                        actor_id: beatrice.clone(),
                        day: 10,
                        event_label: "Duplicate".into(),
                        action: "disclosure".into(),
                        intent: "repeat".into(),
                        outcome: "failed".into(),
                        source_citation: "Northbridge folio 15".into(),
                        trust: Some(20),
                        tags: "estranged".into(),
                    })
                    .is_err()
            );
            assert_eq!(
                workspace.snapshot(&world_id, 9)?.relationships[0].trust,
                Some(90)
            );
        }
        let reopened = AuthorWorkspace::open(&database)?;
        assert_eq!(reopened.worlds()?.len(), 1);
        let after = reopened.snapshot(&world_id, 10)?;
        assert_eq!(after.people.len(), 2);
        assert_eq!(after.relationships[0].id, relationship);
        assert_eq!(after.relationships[0].trust, Some(12));
        assert_eq!(after.relationships[0].tags, vec!["estranged"]);
        let cause = after.relationships[0].cause.as_ref().unwrap();
        assert_eq!(cause.event, "Disclosed letter");
        assert_eq!(cause.actor.as_deref(), Some("Beatrice"));
        assert_eq!(cause.outcome, Some("succeeded"));
        assert_eq!(cause.day, 10);
        assert_eq!(cause.source, "Northbridge folio 14");
        assert!(reopened.snapshot(&world_id, 9)?.relationships[0]
            .cause.is_none());
        assert_eq!(
            reopened.snapshot(&world_id, 9)?.relationships[0].trust,
            Some(90)
        );
        assert_eq!(after.world.revision, 4);
        Ok(())
    }
}
