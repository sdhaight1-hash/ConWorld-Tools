use atlas_store::*;
use tempfile::tempdir;
use uuid::Uuid;

fn command(world: &World, expected: AuthorRevision, changes: Vec<Change>) -> Command {
    Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Northbridge author".into(),
        rationale: "Northbridge fixture".into(),
        scope: Scope::Author,
        changes,
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

fn person(label: &str) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval::default(),
        data: ObjectData::Person,
        source: None,
    }
}

#[test]
fn at_01_rename_keeps_identity_across_reopen_and_backup() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("northbridge.sqlite");
    let backup = dir.path().join("backup.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let letter = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(
                letter,
                Record {
                    label: "Letter".into(),
                    valid: ValidInterval::default(),
                    data: ObjectData::Source {
                        citation: "Northbridge Archive, L-1".into(),
                    },
                    source: None,
                },
            ),
            Change::Create(
                alice,
                Record {
                    source: Some(letter),
                    ..person("Alice Vale")
                },
            ),
        ],
    ))?;
    let second = store.apply(&command(
        &world,
        first,
        vec![Change::Correct(
            alice,
            Record {
                source: Some(letter),
                ..person("Alice Thorne")
            },
        )],
    ))?;
    store.backup_to(&backup, Scope::Author)?;
    assert!(matches!(
        store.backup_to(&backup, Scope::Author),
        Err(Error::Io(_))
    ));
    drop(store);

    for db in [path, backup] {
        let store = Store::open(db)?;
        let reopened = store.world(world.id, Scope::Author)?;
        assert_eq!(reopened.revision, second);
        assert_eq!(reopened.main, world.main);
        store.verify_foreign_keys(Scope::Author)?;
        let before = store.get_object(alice, view(&world, first, 0))?.unwrap();
        let after = store.get_object(alice, view(&world, second, 0))?.unwrap();
        assert_eq!(before.record.label, "Alice Vale");
        assert_eq!(after.record.label, "Alice Thorne");
        assert_eq!(after.object, alice);
        assert_eq!(after.record.source, Some(letter));
    }
    Ok(())
}

#[test]
fn at_15_correction_uses_same_event_and_preserves_recorded_history() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let event = ObjectRef::new(world.id);
    let record = |day| Record {
        label: "Disclosed letter".into(),
        valid: ValidInterval {
            start: Some(WorldDay(day)),
            end_exclusive: Some(WorldDay(day + 1)),
        },
        data: ObjectData::HistoricalEvent {
            occurrence: EventDate::Exact(WorldDay(day)),
        },
        source: None,
    };
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![Change::Create(event, record(1908))],
    ))?;
    let second = store.apply(&command(
        &world,
        first,
        vec![Change::Correct(event, record(1909))],
    ))?;
    assert_eq!(
        store
            .get_object(event, view(&world, first, 1908))?
            .unwrap()
            .record,
        record(1908)
    );
    assert!(
        store
            .get_object(event, view(&world, first, 1909))?
            .is_none()
    );
    assert!(
        store
            .get_object(event, view(&world, second, 1908))?
            .is_none()
    );
    let corrected = store
        .get_object(event, view(&world, second, 1909))?
        .unwrap();
    assert_eq!(corrected.object, event);
    assert_eq!(corrected.record, record(1909));
    assert!(
        store
            .get_object(event, view(&world, second, 1910))?
            .is_none()
    );
    Ok(())
}

#[test]
fn at_26_cross_world_and_wrong_kind_evidence_are_rejected() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let north = store.create_world("Northbridge", Scope::Author)?;
    let south = store.create_world("Southbridge", Scope::Author)?;
    let outside = ObjectRef::new(south.id);
    let alice = ObjectRef::new(north.id);
    let beatrice = ObjectRef::new(north.id);
    store.apply(&command(
        &south,
        south.revision,
        vec![Change::Create(
            outside,
            Record {
                label: "Southbridge letter".into(),
                valid: ValidInterval::default(),
                data: ObjectData::Source {
                    citation: "Southbridge Library, L-1".into(),
                },
                source: None,
            },
        )],
    ))?;
    let revision = store.apply(&command(
        &north,
        north.revision,
        vec![Change::Create(alice, person("Alice Vale"))],
    ))?;
    let cite = |source| {
        command(
            &north,
            revision,
            vec![Change::Create(
                beatrice,
                Record {
                    source: Some(source),
                    ..person("Beatrice Thorne")
                },
            )],
        )
    };
    assert!(matches!(
        store.apply(&cite(outside)),
        Err(Error::WrongWorld)
    ));
    assert!(matches!(store.apply(&cite(alice)), Err(Error::WrongKind)));
    assert!(matches!(
        store.get_object(outside, view(&north, revision, 0)),
        Err(Error::WrongWorld)
    ));
    let mut public = view(&north, revision, 0);
    public.scope = Scope::Public;
    assert!(matches!(
        store.get_object(alice, public),
        Err(Error::CapabilityDenied)
    ));
    Ok(())
}

#[test]
fn at_33_failed_batch_rolls_back_and_allows_retry() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("northbridge.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let beatrice = ObjectRef::new(world.id);
    let invalid = Record {
        valid: ValidInterval {
            start: Some(WorldDay(5)),
            end_exclusive: Some(WorldDay(5)),
        },
        ..person("Beatrice")
    };
    let mut batch = command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, person("Alice")),
            Change::Create(beatrice, invalid),
        ],
    );
    assert!(matches!(store.apply(&batch), Err(Error::InvalidInterval)));
    assert_eq!(
        store.world(world.id, Scope::Author)?.revision,
        world.revision
    );
    let audit = rusqlite::Connection::open(&path)?;
    for table in [
        "objects",
        "object_versions",
        "author_revisions",
        "invalidations",
    ] {
        let count: i64 = audit.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })?;
        assert_eq!(count, 0, "{table}");
    }
    batch.changes[1] = Change::Create(beatrice, person("Beatrice"));
    let latest = store.apply(&batch)?;
    assert_eq!(latest, AuthorRevision(1));
    assert!(matches!(
        store.apply(&batch),
        Err(Error::DuplicateCommand(AuthorRevision(1)))
    ));
    let mut second_connection = Store::open(&path)?;
    assert!(matches!(
        second_connection.apply(&command(
            &world,
            world.revision,
            vec![Change::Correct(alice, person("Stale Alice"))]
        )),
        Err(Error::RevisionConflict {
            actual: AuthorRevision(1),
            ..
        })
    ));
    assert!(store.get_object(alice, view(&world, latest, 0))?.is_some());
    Ok(())
}

#[test]
fn refuses_future_schema_without_modifying_it() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("future.sqlite");
    let future = rusqlite::Connection::open(&path)?;
    future.pragma_update(None, "user_version", 999)?;
    assert!(matches!(Store::open(&path), Err(Error::SchemaTooNew(999))));
    assert_eq!(
        future.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))?,
        999
    );
    Ok(())
}

#[test]
fn at_16_uncertain_dates_keep_their_bounds_and_original_label() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let event = ObjectRef::new(world.id);
    let exact = EventDate::Exact(WorldDay(-20));
    let uncertain = EventDate::Uncertain {
        earliest: Some(WorldDay(-25)),
        latest: Some(WorldDay(-15)),
        label: "About twenty days before the epoch".into(),
    };
    let record = |occurrence| Record {
        label: "Letter delivered".into(),
        valid: ValidInterval::default(),
        data: ObjectData::HistoricalEvent { occurrence },
        source: None,
    };
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![Change::Create(event, record(uncertain.clone()))],
    ))?;
    let second = store.apply(&command(
        &world,
        first,
        vec![Change::Correct(event, record(exact.clone()))],
    ))?;
    for (revision, date) in [(first, uncertain.clone()), (second, exact.clone())] {
        assert_eq!(
            store
                .get_object(event, view(&world, revision, -20))?
                .unwrap()
                .record,
            record(date)
        );
    }
    assert_eq!(uncertain.certainty_on(WorldDay(-20)), Certainty::Possibly);
    assert_eq!(uncertain.certainty_on(WorldDay(-15)), Certainty::Possibly);
    assert_eq!(
        uncertain.certainty_on(WorldDay(-14)),
        Certainty::NotEstablished
    );
    assert_eq!(exact.certainty_on(WorldDay(-20)), Certainty::Definitely);
    assert_eq!(
        EventDate::Unknown.certainty_on(WorldDay(-20)),
        Certainty::NotEstablished
    );
    assert!(matches!(
        store.apply(&command(
            &world,
            second,
            vec![Change::Correct(
                event,
                record(EventDate::Uncertain {
                    earliest: Some(WorldDay(2)),
                    latest: Some(WorldDay(1)),
                    label: "Bad range".into(),
                })
            )]
        )),
        Err(Error::InvalidOccurrence)
    ));
    Ok(())
}
