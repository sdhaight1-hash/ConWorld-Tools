use atlas_store::*;
use uuid::Uuid;

fn command(world: &World, expected: AuthorRevision, changes: Vec<Change>) -> Command {
    Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Northbridge archivist".into(),
        rationale: "Record social history".into(),
        scope: Scope::Author,
        changes,
    }
}
fn person(name: &str) -> Record {
    Record {
        label: name.into(),
        valid: ValidInterval::default(),
        data: ObjectData::Person,
        source: None,
    }
}
fn source() -> Record {
    Record {
        label: "Letter from the archive".into(),
        data: ObjectData::Source {
            citation: "Northbridge folio 14".into(),
        },
        ..person("Letter")
    }
}
fn dated(day: i64, data: ObjectData, source: Option<ObjectRef>, label: &str) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval {
            start: Some(WorldDay(day)),
            end_exclusive: None,
        },
        data,
        source,
    }
}
fn bond(a: ObjectRef, b: ObjectRef, trust: u8, tags: &[&str]) -> ObjectData {
    ObjectData::Relationship {
        subject: a,
        target: b,
        kind: "friendship".into(),
        trust: Some(trust),
        tags: tags.iter().map(|s| s.to_string()).collect(),
    }
}
fn view(world: &World, revision: AuthorRevision, day: i64) -> ViewContext {
    ViewContext {
        world: world.id,
        branch: world.main,
        revision,
        day: WorldDay(day),
        scope: Scope::Author,
    }
}

#[test]
fn at_03_04_05_08_betrayal_preserves_directional_history_and_actor_evidence() -> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("northbridge.sqlite");
    let mut store = Store::open(&db)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let beatrice = ObjectRef::new(world.id);
    let letter = ObjectRef::new(world.id);
    let alice_feels = ObjectRef::new(world.id);
    let beatrice_feels = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, person("Alice Vale")),
            Change::Create(beatrice, person("Beatrice Thorne")),
            Change::Create(letter, source()),
            Change::Create(
                alice_feels,
                dated(
                    0,
                    bond(alice, beatrice, 90, &["friends"]),
                    None,
                    "Alice trusts Beatrice",
                ),
            ),
            Change::Create(
                beatrice_feels,
                dated(
                    0,
                    bond(beatrice, alice, 55, &["friends"]),
                    None,
                    "Beatrice trusts Alice",
                ),
            ),
        ],
    ))?;

    let betrayal = ObjectRef::new(world.id);
    let actor = ObjectRef::new(world.id);
    let witness = ObjectRef::new(world.id);
    let action = ObjectRef::new(world.id);
    let time = 3652; // fictional day 3652 in this fixture, corresponding to the 1908 phase
    let second = store.apply(&command(
        &world,
        first,
        vec![
            Change::Create(
                betrayal,
                dated(
                    time,
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(time)),
                    },
                    Some(letter),
                    "Disclosed letter",
                ),
            ),
            Change::Create(
                actor,
                dated(
                    time,
                    ObjectData::EventInvolvement {
                        event: betrayal,
                        participant: beatrice,
                        role: EventRole::Actor,
                    },
                    Some(letter),
                    "Beatrice involved in disclosure",
                ),
            ),
            Change::Create(
                witness,
                dated(
                    time,
                    ObjectData::EventInvolvement {
                        event: betrayal,
                        participant: alice,
                        role: EventRole::Witness,
                    },
                    Some(letter),
                    "Alice learned of the disclosure",
                ),
            ),
            Change::Create(
                action,
                dated(
                    time,
                    ObjectData::ActorAction {
                        event: betrayal,
                        actor: beatrice,
                        target: Some(alice),
                        action: "disclose_letter".into(),
                        intent: "force a public succession dispute".into(),
                        outcome: ActionOutcome::Succeeded,
                    },
                    Some(letter),
                    "Beatrice disclosed the letter",
                ),
            ),
            Change::ActionEffect {
                event: betrayal,
                action,
                target: alice_feels,
                record: dated(
                    time,
                    bond(alice, beatrice, 12, &["estranged"]),
                    Some(letter),
                    "Alice distrusts Beatrice",
                ),
            },
        ],
    ))?;

    assert_eq!(
        store
            .get_object(alice_feels, view(&world, second, 20))?
            .unwrap()
            .record
            .data,
        bond(alice, beatrice, 90, &["friends"])
    );
    assert_eq!(
        store
            .get_object(alice_feels, view(&world, second, time))?
            .unwrap()
            .record
            .data,
        bond(alice, beatrice, 12, &["estranged"])
    );
    assert_eq!(
        store
            .get_object(beatrice_feels, view(&world, second, time))?
            .unwrap()
            .record
            .data,
        bond(beatrice, alice, 55, &["friends"])
    );
    assert_eq!(
        store
            .get_object(alice_feels, view(&world, first, time))?
            .unwrap()
            .record
            .data,
        bond(alice, beatrice, 90, &["friends"])
    );
    assert_eq!(
        store
            .get_object(actor, view(&world, second, time))?
            .unwrap()
            .record
            .data,
        ObjectData::EventInvolvement {
            event: betrayal,
            participant: beatrice,
            role: EventRole::Actor
        }
    );
    assert_eq!(
        store
            .get_object(action, view(&world, second, time))?
            .unwrap()
            .record
            .data,
        ObjectData::ActorAction {
            event: betrayal,
            actor: beatrice,
            target: Some(alice),
            action: "disclose_letter".into(),
            intent: "force a public succession dispute".into(),
            outcome: ActionOutcome::Succeeded
        }
    );
    let effect = store
        .effect_on(alice_feels, view(&world, second, time))?
        .unwrap();
    assert_eq!(effect.event.object, betrayal);
    assert_eq!(effect.action.unwrap().object, action);
    assert_eq!(effect.before.recorded_at, first);
    assert_eq!(effect.after.recorded_at, second);
    assert!(
        store
            .effect_on(beatrice_feels, view(&world, second, time))?
            .is_none()
    );
    assert!(
        store
            .get_object(action, view(&world, second, 20))?
            .is_none()
    );

    let alliance = ObjectRef::new(world.id);
    let third = store.apply(&command(
        &world,
        second,
        vec![Change::Create(
            alliance,
            dated(
                5844,
                ObjectData::Relationship {
                    subject: alice,
                    target: beatrice,
                    kind: "political_alliance".into(),
                    trust: None,
                    tags: vec!["uneasy".into()],
                },
                Some(letter),
                "Uneasy 1914 alliance",
            ),
        )],
    ))?;
    assert!(
        store
            .get_object(alliance, view(&world, third, 5843))?
            .is_none()
    );
    assert!(
        store
            .get_object(alliance, view(&world, third, 5844))?
            .is_some()
    );
    assert_eq!(
        store
            .get_object(alice_feels, view(&world, third, 5844))?
            .unwrap()
            .record
            .data,
        bond(alice, beatrice, 12, &["estranged"])
    );
    drop(store);
    let reopened = Store::open(&db)?;
    assert!(
        reopened
            .get_object(alliance, view(&world, third, 5844))?
            .is_some()
    );
    assert_eq!(
        reopened
            .effect_on(alice_feels, view(&world, third, time))?
            .unwrap()
            .event
            .object,
        betrayal
    );
    reopened.verify_foreign_keys(Scope::Author)
}

#[test]
fn phase_two_rejects_invalid_links_and_rolls_back_the_whole_command() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let other = store.create_world("Other", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let beatrice = ObjectRef::new(world.id);
    let letter = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, person("Alice")),
            Change::Create(beatrice, person("Beatrice")),
            Change::Create(letter, source()),
            Change::Create(
                event,
                dated(
                    10,
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(10)),
                    },
                    Some(letter),
                    "Betrayal",
                ),
            ),
        ],
    ))?;
    let relationship = ObjectRef::new(world.id);
    let invalid = [
        dated(0, bond(alice, alice, 30, &["friend"]), None, "Self bond"),
        dated(
            0,
            bond(alice, beatrice, 30, &["friends", "Friends"]),
            None,
            "Duplicate tag",
        ),
        dated(
            0,
            bond(ObjectRef::new(other.id), alice, 30, &[]),
            None,
            "Other world",
        ),
        dated(
            0,
            ObjectData::Relationship {
                subject: event,
                target: alice,
                kind: "friendship".into(),
                trust: Some(20),
                tags: vec![],
            },
            None,
            "Wrong kind",
        ),
        dated(
            0,
            ObjectData::Relationship {
                subject: alice,
                target: beatrice,
                kind: "friendship".into(),
                trust: Some(101),
                tags: vec![],
            },
            None,
            "Score out of range",
        ),
    ];
    for record in invalid {
        assert!(
            store
                .apply(&command(
                    &world,
                    first,
                    vec![Change::Create(relationship, record)]
                ))
                .is_err()
        );
        assert_eq!(store.world(world.id, Scope::Author)?.revision, first);
        assert!(
            store
                .get_object(relationship, view(&world, first, 0))?
                .is_none()
        );
    }
    let bad_action = ObjectRef::new(world.id);
    let wrong_time = dated(
        9,
        ObjectData::ActorAction {
            event,
            actor: beatrice,
            target: Some(alice),
            action: "disclosure".into(),
            intent: "influence succession".into(),
            outcome: ActionOutcome::Failed,
        },
        Some(letter),
        "Pre-event attempt",
    );
    assert!(matches!(
        store.apply(&command(
            &world,
            first,
            vec![
                Change::Create(
                    relationship,
                    dated(0, bond(alice, beatrice, 50, &[]), None, "Valid")
                ),
                Change::Create(bad_action, wrong_time),
            ]
        )),
        // The event is not visible at day 9: its valid interval begins on day 10.
        Err(Error::NotFound)
    ));
    assert_eq!(store.world(world.id, Scope::Author)?.revision, first);
    assert!(
        store
            .get_object(relationship, view(&world, first, 0))?
            .is_none()
    );
    assert!(
        store
            .get_object(bad_action, view(&world, first, 9))?
            .is_none()
    );
    store.verify_foreign_keys(Scope::Author)
}

#[test]
fn phase_two_relationship_identity_is_not_reassigned() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let beatrice = ObjectRef::new(world.id);
    let charlie = ObjectRef::new(world.id);
    let relation = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, person("Alice")),
            Change::Create(beatrice, person("Beatrice")),
            Change::Create(charlie, person("Charlie")),
            Change::Create(
                relation,
                dated(0, bond(alice, beatrice, 90, &[]), None, "Friendship"),
            ),
        ],
    ))?;
    assert!(matches!(
        store.apply(&command(
            &world,
            first,
            vec![Change::Correct(
                relation,
                dated(0, bond(alice, charlie, 90, &[]), None, "Different person")
            ),]
        )),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        store.apply(&command(
            &world,
            first,
            vec![Change::Correct(
                relation,
                dated(
                    0,
                    ObjectData::Relationship {
                        subject: alice,
                        target: beatrice,
                        kind: "rivalry".into(),
                        trust: Some(1),
                        tags: vec![]
                    },
                    None,
                    "Changed kind"
                )
            ),]
        )),
        Err(Error::InvalidInput(_))
    ));
    assert_eq!(store.world(world.id, Scope::Author)?.revision, first);
    assert_eq!(
        store
            .get_object(relation, view(&world, first, 0))?
            .unwrap()
            .record
            .data,
        bond(alice, beatrice, 90, &[])
    );
    store.verify_foreign_keys(Scope::Author)
}

#[test]
fn phase_two_action_effect_rejects_a_different_event_and_rolls_back() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let beatrice = ObjectRef::new(world.id);
    let source_id = ObjectRef::new(world.id);
    let first_event = ObjectRef::new(world.id);
    let second_event = ObjectRef::new(world.id);
    let bond_id = ObjectRef::new(world.id);
    let action = ObjectRef::new(world.id);
    let first = store.apply(&command(&world, world.revision, vec![
        Change::Create(alice, person("Alice")),
        Change::Create(beatrice, person("Beatrice")),
        Change::Create(source_id, source()),
        Change::Create(first_event, dated(10, ObjectData::HistoricalEvent {
            occurrence: EventDate::Exact(WorldDay(10)) }, Some(source_id), "First event")),
        Change::Create(second_event, dated(12, ObjectData::HistoricalEvent {
            occurrence: EventDate::Exact(WorldDay(12)) }, Some(source_id), "Second event")),
        Change::Create(bond_id, dated(0, bond(alice, beatrice, 80, &["friends"]), None, "Friendship")),
        Change::Create(action, dated(12, ObjectData::ActorAction {
            event: second_event, actor: beatrice, target: Some(alice),
            action: "disclosure".into(), intent: "secure a title".into(),
            outcome: ActionOutcome::Failed,
        }, Some(source_id), "Failed attempt")),
    ]))?;
    let invalid_effect = Change::ActionEffect {
        event: first_event, action, target: bond_id,
        record: dated(12, bond(alice, beatrice, 20, &["estranged"]), Some(source_id), "Distrust"),
    };
    assert!(matches!(store.apply(&command(&world, first, vec![invalid_effect])),
        Err(Error::InvalidInput(_))));
    assert_eq!(store.world(world.id, Scope::Author)?.revision, first);
    assert_eq!(store.get_object(bond_id, view(&world, first, 20))?.unwrap().record.data,
        bond(alice, beatrice, 80, &["friends"]));
    assert!(store.effect_on(bond_id, view(&world, first, 20))?.is_none());
    store.verify_foreign_keys(Scope::Author)
}
