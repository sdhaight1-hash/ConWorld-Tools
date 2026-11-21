//! Native-only Northbridge history fixture. Readable through a scoped Tauri command.
//! This intentionally uses the real Phase 1/2 Store, not a second frontend model.
use std::sync::Mutex;

use atlas_store::{
    ActionOutcome, AuthorRevision, Change, Command, Error, EventDate, EventRole, ObjectData,
    ObjectRef, Record, Scope, Store, ValidInterval, ViewContext, World, WorldDay,
};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

pub struct NorthbridgeHistory {
    store: Mutex<Store>,
    world: World,
    alice_bond: ObjectRef,
    beatrice_bond: ObjectRef,
    alliance: ObjectRef,
    event: ObjectRef,
    actor_role: ObjectRef,
    witness_role: ObjectRef,
    action: ObjectRef,
    source: ObjectRef,
}

#[derive(Debug, Serialize)]
pub struct BondSnapshot {
    subject: String,
    target: String,
    kind: String,
    trust: Option<u8>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EventSnapshot {
    event: String,
    actor: String,
    witness: String,
    action: String,
    intent: String,
    outcome: &'static str,
    source: String,
}

#[derive(Debug, Serialize)]
pub struct HistorySnapshot {
    day: i64,
    relationships: Vec<BondSnapshot>,
    cause: Option<EventSnapshot>,
}

fn record(label: &str, start: Option<i64>, data: ObjectData, source: Option<ObjectRef>) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval {
            start: start.map(WorldDay),
            end_exclusive: None,
        },
        data,
        source,
    }
}

fn relationship(
    a: ObjectRef,
    b: ObjectRef,
    kind: &str,
    trust: Option<u8>,
    tags: &[&str],
) -> ObjectData {
    ObjectData::Relationship {
        subject: a,
        target: b,
        kind: kind.into(),
        trust,
        tags: tags.iter().map(|tag| (*tag).into()).collect(),
    }
}

fn create(
    store: &mut Store,
    world: &World,
    expected: AuthorRevision,
    changes: Vec<Change>,
) -> Result<AuthorRevision, Error> {
    store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Northbridge archivist".into(),
        rationale: "Record the demonstration history".into(),
        scope: Scope::Author,
        changes,
    })
}

impl NorthbridgeHistory {
    pub fn seed() -> Result<Self, Error> {
        let mut store = Store::open(":memory:")?;
        let world = store.create_world("Northbridge", Scope::Author)?;
        let alice = ObjectRef::new(world.id);
        let beatrice = ObjectRef::new(world.id);
        let source = ObjectRef::new(world.id);
        let alice_bond = ObjectRef::new(world.id);
        let beatrice_bond = ObjectRef::new(world.id);
        let first = create(
            &mut store,
            &world,
            world.revision,
            vec![
                Change::Create(alice, record("Alice Vale", None, ObjectData::Person, None)),
                Change::Create(
                    beatrice,
                    record("Beatrice Thorne", None, ObjectData::Person, None),
                ),
                Change::Create(
                    source,
                    record(
                        "Succession letter",
                        None,
                        ObjectData::Source {
                            citation: "Northbridge folio 14".into(),
                        },
                        None,
                    ),
                ),
                Change::Create(
                    alice_bond,
                    record(
                        "Alice trusts Beatrice",
                        Some(0),
                        relationship(alice, beatrice, "friendship", Some(90), &["friends"]),
                        None,
                    ),
                ),
                Change::Create(
                    beatrice_bond,
                    record(
                        "Beatrice trusts Alice",
                        Some(0),
                        relationship(beatrice, alice, "friendship", Some(55), &["friends"]),
                        None,
                    ),
                ),
            ],
        )?;
        let event = ObjectRef::new(world.id);
        let actor_role = ObjectRef::new(world.id);
        let witness_role = ObjectRef::new(world.id);
        let action = ObjectRef::new(world.id);
        let second = create(
            &mut store,
            &world,
            first,
            vec![
                Change::Create(
                    event,
                    record(
                        "Disclosed succession letter",
                        Some(3652),
                        ObjectData::HistoricalEvent {
                            occurrence: EventDate::Exact(WorldDay(3652)),
                        },
                        Some(source),
                    ),
                ),
                Change::Create(
                    actor_role,
                    record(
                        "Discloser",
                        Some(3652),
                        ObjectData::EventInvolvement {
                            event,
                            participant: beatrice,
                            role: EventRole::Actor,
                        },
                        Some(source),
                    ),
                ),
                Change::Create(
                    witness_role,
                    record(
                        "Witness",
                        Some(3652),
                        ObjectData::EventInvolvement {
                            event,
                            participant: alice,
                            role: EventRole::Witness,
                        },
                        Some(source),
                    ),
                ),
                Change::Create(
                    action,
                    record(
                        "Beatrice disclosed the letter",
                        Some(3652),
                        ObjectData::ActorAction {
                            event,
                            actor: beatrice,
                            target: Some(alice),
                            action: "disclose_letter".into(),
                            intent: "force a succession dispute".into(),
                            outcome: ActionOutcome::Succeeded,
                        },
                        Some(source),
                    ),
                ),
                Change::ActionEffect {
                    event,
                    action,
                    target: alice_bond,
                    record: record(
                        "Alice distrusts Beatrice",
                        Some(3652),
                        relationship(alice, beatrice, "friendship", Some(12), &["estranged"]),
                        Some(source),
                    ),
                },
            ],
        )?;
        let alliance = ObjectRef::new(world.id);
        create(
            &mut store,
            &world,
            second,
            vec![Change::Create(
                alliance,
                record(
                    "Uneasy political alliance",
                    Some(5844),
                    relationship(alice, beatrice, "political_alliance", None, &["uneasy"]),
                    Some(source),
                ),
            )],
        )?;
        Ok(Self {
            store: Mutex::new(store),
            world,
            alice_bond,
            beatrice_bond,
            alliance,
            event,
            actor_role,
            witness_role,
            action,
            source,
        })
    }

    fn read(&self, day: i64) -> Result<HistorySnapshot, Error> {
        let store = self
            .store
            .lock()
            .map_err(|_| Error::Integrity("Northbridge history lock".into()))?;
        let revision = store.world(self.world.id, Scope::Author)?.revision;
        let view = ViewContext {
            world: self.world.id,
            branch: self.world.main,
            revision,
            day: WorldDay(day),
            scope: Scope::Author,
        };
        let label = |id: ObjectRef| -> Result<String, Error> {
            Ok(store
                .get_object(id, view)?
                .ok_or(Error::NotFound)?
                .record
                .label)
        };
        let mut relationships = Vec::new();
        for id in [self.alice_bond, self.beatrice_bond, self.alliance] {
            if let Some(version) = store.get_object(id, view)? {
                if let ObjectData::Relationship {
                    subject,
                    target,
                    kind,
                    trust,
                    tags,
                } = version.record.data
                {
                    relationships.push(BondSnapshot {
                        subject: label(subject)?,
                        target: label(target)?,
                        kind,
                        trust,
                        tags,
                    });
                } else {
                    return Err(Error::WrongKind);
                }
            }
        }
        let cause = if let Some(effect) = store.effect_on(self.alice_bond, view)? {
            if effect.event.object != self.event || effect.action.map(|version| version.object) != Some(self.action) {
                return Err(Error::Integrity("unexpected effect origin".into()));
            }
            let role = store
                .get_object(self.actor_role, view)?
                .ok_or(Error::NotFound)?
                .record
                .data;
            let witness = store
                .get_object(self.witness_role, view)?
                .ok_or(Error::NotFound)?
                .record
                .data;
            let action = store
                .get_object(self.action, view)?
                .ok_or(Error::NotFound)?
                .record
                .data;
            let source = store
                .get_object(self.source, view)?
                .ok_or(Error::NotFound)?
                .record
                .data;
            let actor = match role {
                ObjectData::EventInvolvement {
                    event,
                    participant,
                    role: EventRole::Actor,
                } if event == self.event => label(participant)?,
                _ => return Err(Error::WrongKind),
            };
            let witness = match witness {
                ObjectData::EventInvolvement {
                    event,
                    participant,
                    role: EventRole::Witness,
                } if event == self.event => label(participant)?,
                _ => return Err(Error::WrongKind),
            };
            let (action, intent, outcome) = match action {
                ObjectData::ActorAction {
                    event,
                    actor: action_actor,
                    action,
                    intent,
                    outcome,
                    ..
                } if event == self.event && label(action_actor)? == actor => {
                    let outcome = match outcome {
                        ActionOutcome::Attempted => "attempted",
                        ActionOutcome::Succeeded => "succeeded",
                        ActionOutcome::Failed => "failed",
                    };
                    (action, intent, outcome)
                }
                _ => return Err(Error::WrongKind),
            };
            let citation = match source {
                ObjectData::Source { citation } => citation,
                _ => return Err(Error::WrongKind),
            };
            Some(EventSnapshot {
                event: label(self.event)?,
                actor,
                witness,
                action,
                intent,
                outcome,
                source: citation,
            })
        } else {
            None
        };
        Ok(HistorySnapshot {
            day,
            relationships,
            cause,
        })
    }
}

#[tauri::command]
pub fn northbridge_history(
    history: State<'_, NorthbridgeHistory>,
    day: i64,
) -> Result<HistorySnapshot, String> {
    // Phase 2 fixture: time-picking is intentionally limited to the three authored snapshots.
    if ![0, 3652, 5844].contains(&day) {
        return Err("Unsupported Northbridge demonstration date".into());
    }
    history.read(day).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_fixture_reconstructs_contradictory_trust_and_event_cause() -> Result<(), Error> {
        let fixture = NorthbridgeHistory::seed()?;
        let early = fixture.read(0)?;
        assert_eq!(early.relationships.len(), 2);
        assert_eq!(early.relationships[0].trust, Some(90));
        assert_eq!(early.relationships[1].trust, Some(55));
        assert!(early.cause.is_none());
        let later = fixture.read(3652)?;
        assert_eq!(later.relationships[0].trust, Some(12));
        assert_eq!(later.relationships[1].trust, Some(55));
        assert_eq!(later.cause.unwrap().source, "Northbridge folio 14");
        let alliance = fixture.read(5844)?;
        assert_eq!(alliance.relationships.len(), 3);
        assert_eq!(alliance.relationships[0].trust, Some(12));
        Ok(())
    }
}
