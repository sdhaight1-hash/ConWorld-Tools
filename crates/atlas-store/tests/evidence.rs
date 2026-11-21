use atlas_store::*;
use uuid::Uuid;

fn change_set(world: &World, expected: AuthorRevision, changes: Vec<Change>) -> Command {
    Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Northbridge author".into(),
        rationale: "Document the succession".into(),
        scope: Scope::Author,
        changes,
    }
}

fn person(label: &str) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval::default(),
        data: ObjectData::Person,
        source: None,
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
fn sourced_claim_is_versioned_without_changing_its_subject() -> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("claims.sqlite");
    let mut store = Store::open(&db)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let letter = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let mut statement = Record {
        data: ObjectData::Claim {
            audience: atlas_store::ClaimAudience::Author,
            subject: alice,
            predicate: "holds_title".into(),
            statement: "The letter calls Alice the heir".into(),
        },
        source: Some(letter),
        ..person("Disputed letter")
    };
    let first = store.apply(&change_set(
        &world,
        world.revision,
        vec![
            Change::Create(alice, person("Alice Vale")),
            Change::Create(
                letter,
                Record {
                    data: ObjectData::Source {
                        citation: "Archive 14".into(),
                    },
                    ..person("Letter")
                },
            ),
            Change::Create(claim, statement.clone()),
        ],
    ))?;
    let original = statement.clone();
    if let ObjectData::Claim { statement, .. } = &mut statement.data {
        *statement = "The letter calls Alice a witness".into();
    }
    let second = store.apply(&change_set(
        &world,
        first,
        vec![Change::Correct(claim, statement.clone())],
    ))?;
    let mut unsourced = statement.clone();
    unsourced.source = None;
    assert!(matches!(
        store.apply(&change_set(
            &world,
            second,
            vec![Change::Correct(claim, unsourced)]
        )),
        Err(Error::InvalidInput(_))
    ));
    drop(store);
    let store = Store::open(db)?;
    assert_eq!(
        store
            .get_object(claim, view(&world, first, 9))?
            .unwrap()
            .record,
        original
    );
    assert_eq!(
        store
            .get_object(claim, view(&world, second, 9))?
            .unwrap()
            .record,
        statement
    );
    assert_eq!(
        store
            .get_object(alice, view(&world, second, 9))?
            .unwrap()
            .record,
        person("Alice Vale")
    );
    store.verify_foreign_keys(Scope::Author)
}

#[test]
fn explicit_event_effects_are_atomic_and_expose_both_versions() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let bren = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let first = store.apply(&change_set(
        &world,
        world.revision,
        vec![
            Change::Create(alice, person("Alice Vale")),
            Change::Create(bren, person("Bren Vale")),
            Change::Create(
                source,
                Record {
                    data: ObjectData::Source {
                        citation: "Archive 21".into(),
                    },
                    ..person("Treaty")
                },
            ),
        ],
    ))?;
    let effect = |target, day| Change::EventEffect {
        event,
        target,
        record: Record {
            source: Some(source),
            valid: ValidInterval {
                start: Some(WorldDay(day)),
                end_exclusive: None,
            },
            ..person("Vale of Northbridge")
        },
    };
    let occurrence = Change::Create(
        event,
        Record {
            data: ObjectData::HistoricalEvent {
                occurrence: EventDate::Exact(WorldDay(12)),
            },
            ..person("Treaty signed")
        },
    );
    let invalid = change_set(
        &world,
        first,
        vec![occurrence.clone(), effect(alice, 12), effect(bren, 11)],
    );
    assert!(matches!(
        store.apply(&invalid),
        Err(Error::InvalidOccurrence)
    ));
    assert_eq!(store.world(world.id, Scope::Author)?.revision, first);
    assert!(store.get_object(event, view(&world, first, 12))?.is_none());
    assert!(store.effect_on(alice, view(&world, first, 12))?.is_none());

    let second = store.apply(&change_set(
        &world,
        first,
        vec![occurrence, effect(alice, 12), effect(bren, 13)],
    ))?;
    for (target, day) in [(alice, 12), (bren, 13)] {
        let cause = store.effect_on(target, view(&world, second, day))?.unwrap();
        assert_eq!(cause.event.object, event);
        assert_eq!(cause.before.recorded_at, first);
        assert_eq!(cause.after.recorded_at, second);
        assert_eq!(cause.day, WorldDay(day));
        assert!(
            store
                .effect_on(target, view(&world, second, day - 1))?
                .is_none()
        );
    }
    store.verify_foreign_keys(Scope::Author)
}

#[test]
fn event_anchored_fork_retains_calendared_provenance_after_correction() -> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("atlas.sqlite");
    let mut store = Store::open(&db)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let id = CalendarId(Uuid::new_v4());
    let defined = store.register_calendar(&CalendarCommand {
        world: world.id,
        calendar: id,
        expected: world.revision,
        command_id: Uuid::new_v4(),
        author: "Archivist".into(),
        rationale: "Define the date system".into(),
        scope: Scope::Author,
        rules: CalendarRules {
            name: "River calendar".into(),
            epoch: WorldDay(0),
            month_lengths: vec![28],
            leap: None,
            eras: Vec::new(),
        },
    })?;
    let calendar = store.calendar(world.id, id, defined, Scope::Author)?;
    let occurrence = calendar.exact_occurrence(
        CalendarDay {
            year: 0,
            month: 1,
            day: 13,
        },
        "13 Highwater",
    )?;
    let treaty = ObjectRef::new(world.id);
    let event = |occurrence| Record {
        data: ObjectData::HistoricalEvent { occurrence },
        ..person("Treaty")
    };
    let written = store.apply(&change_set(
        &world,
        defined,
        vec![Change::Create(treaty, event(occurrence.clone()))],
    ))?;
    let mut command = ForkCommand {
        world: world.id,
        parent: world.main,
        day: WorldDay(11),
        expected: written,
        id: Uuid::new_v4(),
        author: "Archivist".into(),
        rationale: "Fork at the treaty".into(),
        scope: Scope::Author,
    };
    assert!(matches!(
        store.fork_at_event(&command, treaty),
        Err(Error::InvalidOccurrence)
    ));
    command.day = WorldDay(12);
    let fork = store.fork_at_event(&command, treaty)?;
    assert_eq!(fork.anchor.unwrap().recorded_at, written);
    let corrected = store.apply(&change_set(
        &world,
        fork.created_revision,
        vec![Change::Correct(
            treaty,
            event(EventDate::Exact(WorldDay(14))),
        )],
    ))?;
    drop(store);
    let store = Store::open(db)?;
    assert_eq!(
        store.branch_fork(world.id, fork.id, corrected, Scope::Author)?,
        Some(fork)
    );
    let old = store
        .get_object(
            treaty,
            ViewContext {
                branch: fork.id,
                ..view(&world, corrected, 12)
            },
        )?
        .unwrap();
    assert_eq!(old.record.data, ObjectData::HistoricalEvent { occurrence });
    assert_eq!(
        store
            .get_object(treaty, view(&world, corrected, 14))?
            .unwrap()
            .record
            .data,
        ObjectData::HistoricalEvent {
            occurrence: EventDate::Exact(WorldDay(14))
        }
    );
    store.verify_foreign_keys(Scope::Author)
}

#[test]
fn restricted_evidence_cannot_replay_a_revoked_public_statement() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let mut record = Record {
        data: ObjectData::Claim {
            subject: event,
            predicate: "report".into(),
            statement: "The clerk reported a disturbance".into(),
            audience: ClaimAudience::Public,
        },
        source: Some(source),
        ..person("Report")
    };
    let first = store.apply(&change_set(
        &world,
        world.revision,
        vec![
            Change::Create(
                event,
                Record {
                    data: ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(0)),
                    },
                    ..person("Meeting")
                },
            ),
            Change::Create(
                source,
                Record {
                    data: ObjectData::Source {
                        citation: "Private note".into(),
                    },
                    ..person("Source")
                },
            ),
            Change::Create(claim, record.clone()),
        ],
    ))?;
    let published = ViewContext {
        scope: Scope::Public,
        ..view(&world, first, 0)
    };
    let items = store.evidence_at(event, published)?;
    assert_eq!(items.len(), 1);
    assert!(items[0].source.is_none());
    if let ObjectData::Claim { audience, .. } = &mut record.data {
        *audience = ClaimAudience::Author;
    }
    let second = store.apply(&change_set(
        &world,
        first,
        vec![Change::Correct(claim, record)],
    ))?;
    assert!(matches!(
        store.evidence_at(event, published),
        Err(Error::InvalidRevision)
    ));
    assert!(
        store
            .evidence_at(
                event,
                ViewContext {
                    revision: second,
                    ..published
                }
            )?
            .is_empty()
    );
    assert_eq!(store.evidence_at(event, view(&world, first, 0))?.len(), 1);
    assert!(store.get_object(claim, published).is_err());
    Ok(())
}
