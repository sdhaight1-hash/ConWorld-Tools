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

#[test]
fn later_fictional_state_preserves_earlier_state_at_same_author_revision() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("atlas.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![Change::Create(alice, person("Alice Vale"))],
    ))?;
    let later = Record {
        valid: ValidInterval {
            start: Some(WorldDay(10)),
            end_exclusive: None,
        },
        ..person("Alice Thorne")
    };
    let second = store.apply(&command(
        &world,
        first,
        vec![Change::RecordState(alice, later.clone())],
    ))?;
    assert!(matches!(
        store.apply(&command(
            &world,
            second,
            vec![Change::RecordState(alice, later.clone())],
        )),
        Err(Error::InvalidInterval)
    ));
    let corrected = Record {
        valid: ValidInterval {
            start: Some(WorldDay(12)),
            end_exclusive: None,
        },
        ..later
    };
    let third = store.apply(&command(
        &world,
        second,
        vec![Change::Correct(alice, corrected)],
    ))?;
    drop(store);
    let store = Store::open(&path)?;

    for (revision, day, expected) in [
        (first, 10, Some("Alice Vale")),
        (second, 9, Some("Alice Vale")),
        (second, 10, Some("Alice Thorne")),
        (third, 9, Some("Alice Vale")),
        (third, 10, None),
        (third, 12, Some("Alice Thorne")),
    ] {
        let version = store.get_object(alice, view(&world, revision, day))?;
        assert_eq!(
            version.as_ref().map(|v| v.record.label.as_str()),
            expected,
            "{revision:?} {day}"
        );
    }
    store.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn version_one_database_retains_corrections_on_upgrade() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("v1.sqlite");
    let conn = rusqlite::Connection::open(&path)?;
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.pragma_update(None, "application_id", 0x4357_4c44_i64)?;
    conn.pragma_update(None, "user_version", 1)?;
    conn.execute_batch(include_str!("../migrations/0001.sql"))?;
    let world = World {
        id: WorldId(Uuid::from_u128(1)),
        name: "Legacy Northbridge".into(),
        main: BranchId(Uuid::from_u128(2)),
        revision: AuthorRevision(2),
    };
    let event = ObjectRef {
        world: world.id,
        id: ObjectId(Uuid::from_u128(3)),
    };
    let world_id = world.id.0.as_bytes();
    let branch_id = world.main.0.as_bytes();
    conn.execute(
        "INSERT INTO worlds VALUES (?1, ?2)",
        rusqlite::params![world_id, world.name],
    )?;
    conn.execute(
        "INSERT INTO branches VALUES (?1, ?2, 1)",
        rusqlite::params![world_id, branch_id],
    )?;
    conn.execute(
        "INSERT INTO objects VALUES (?1, ?2, 'historical_event')",
        rusqlite::params![world_id, event.id.0.as_bytes()],
    )?;
    for (revision, day, label) in [(1, 10, "Old account"), (2, 12, "Corrected account")] {
        conn.execute(
            "INSERT INTO author_revisions (world_id, revision, branch_id, command_id, author, rationale)
             VALUES (?1, ?2, ?3, ?4, 'author', 'test')",
            rusqlite::params![world_id, revision, branch_id, Uuid::from_u128(10 + revision as u128).as_bytes()],
        )?;
        conn.execute(
            "INSERT INTO object_versions (world_id, object_id, branch_id, revision, kind, label, valid_start, valid_end, event_day)
             VALUES (?1, ?2, ?3, ?4, 'historical_event', ?5, ?6, ?7, ?6)",
            rusqlite::params![world_id, event.id.0.as_bytes(), branch_id, revision, label, day, day + 1],
        )?;
    }
    drop(conn);
    let store = Store::open_with_migration_backup(&path)?;
    let backups = std::fs::read_dir(dir.path())?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("v1.sqlite.pre-schema-1-")
        })
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
    let backup = rusqlite::Connection::open_with_flags(
        backups[0].path(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    let old_schema: i64 = backup.pragma_query_value(None, "user_version", |row| row.get(0))?;
    assert_eq!(old_schema, 1);
    let old_versions: i64 =
        backup.query_row("SELECT count(*) FROM object_versions", [], |row| row.get(0))?;
    assert_eq!(old_versions, 2);
    drop(backup);
    assert_eq!(
        store
            .get_object(event, view(&world, AuthorRevision(1), 10))?
            .unwrap()
            .record
            .label,
        "Old account"
    );
    assert!(
        store
            .get_object(event, view(&world, AuthorRevision(2), 10))?
            .is_none()
    );
    assert_eq!(
        store
            .get_object(event, view(&world, AuthorRevision(2), 12))?
            .unwrap()
            .record
            .label,
        "Corrected account"
    );
    store.verify_foreign_keys(Scope::Author)
}

fn fork_command(world: &World, parent: BranchId, at: i64, expected: AuthorRevision) -> ForkCommand {
    ForkCommand {
        world: world.id,
        parent,
        day: WorldDay(at),
        expected,
        id: Uuid::new_v4(),
        author: "Northbridge author".into(),
        rationale: "Alternate history".into(),
        scope: Scope::Author,
    }
}

#[test]
fn at_14_fork_pins_history_and_isolates_successive_child_states() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("atlas.sqlite");
    let backup = dir.path().join("snapshot.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, person("Alice Vale")),
            Change::Create(
                event,
                Record {
                    label: "Planned succession".into(),
                    data: ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(20)),
                    },
                    valid: ValidInterval::default(),
                    source: None,
                },
            ),
        ],
    ))?;
    let request = fork_command(&world, world.main, 10, first);
    let child = store.fork(&request)?;
    assert_eq!(child.inherited_revision, first);
    assert_eq!(child.parent, world.main);
    assert!(
        matches!(store.fork(&request), Err(Error::DuplicateCommand(revision)) if revision == child.created_revision)
    );

    let parent_revision = store.apply(&command(
        &world,
        child.created_revision,
        vec![Change::Correct(alice, person("Alice corrected on main"))],
    ))?;
    let mut edit = command(
        &world,
        parent_revision,
        vec![Change::RecordState(
            alice,
            Record {
                valid: ValidInterval {
                    start: Some(WorldDay(13)),
                    end_exclusive: None,
                },
                ..person("Alice of the Guild")
            },
        )],
    );
    edit.branch = child.id;
    let child_revision = store.apply(&edit)?;
    store.backup_to(&backup, Scope::Author)?;
    drop(store);

    let store = Store::open(backup)?;
    assert_eq!(
        store.branch_fork(world.id, child.id, child_revision, Scope::Author)?,
        Some(child)
    );
    assert_eq!(
        store.branch_fork(world.id, world.main, child_revision, Scope::Author)?,
        None
    );
    assert!(matches!(
        store.branch_fork(world.id, child.id, child_revision, Scope::Public),
        Err(Error::CapabilityDenied)
    ));
    let in_child = |revision, day| ViewContext {
        branch: child.id,
        ..view(&world, revision, day)
    };
    assert!(matches!(
        store.get_object(alice, in_child(first, 9)),
        Err(Error::NotFound)
    ));
    for (revision, day, label, source_branch) in [
        (child_revision, 9, "Alice Vale", world.main),
        (child_revision, 12, "Alice Vale", world.main),
        (child_revision, 13, "Alice of the Guild", child.id),
        (parent_revision, 13, "Alice Vale", world.main),
    ] {
        let result = store.get_object(alice, in_child(revision, day))?.unwrap();
        assert_eq!(result.record.label, label);
        assert_eq!(result.source_branch, source_branch);
    }
    assert!(
        store
            .get_object(event, in_child(child_revision, 20))?
            .is_none()
    );
    assert!(
        store
            .get_object(event, view(&world, child_revision, 20))?
            .is_some()
    );
    assert_eq!(
        store
            .get_object(alice, view(&world, child_revision, 13))?
            .unwrap()
            .record
            .label,
        "Alice corrected on main"
    );
    store.verify_foreign_keys(Scope::Author)
}

#[test]
fn nested_forks_isolate_sibling_evidence_and_reject_pre_fork_edits() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![Change::Create(alice, person("Alice Vale"))],
    ))?;
    let child = store.fork(&fork_command(&world, world.main, 10, first))?;
    assert!(matches!(
        store.fork(&fork_command(&world, child.id, 9, child.created_revision)),
        Err(Error::BeforeFork)
    ));
    let child_source = ObjectRef::new(world.id);
    let mut edit = command(
        &world,
        child.created_revision,
        vec![
            Change::Create(
                child_source,
                Record {
                    data: ObjectData::Source {
                        citation: "Child ledger".into(),
                    },
                    ..person("Ledger")
                },
            ),
            Change::Correct(
                alice,
                Record {
                    source: Some(child_source),
                    ..person("Alice in child")
                },
            ),
        ],
    );
    edit.branch = child.id;
    let child_revision = store.apply(&edit)?;
    let grandchild = store.fork(&fork_command(&world, child.id, 12, child_revision))?;
    let sibling = store.fork(&fork_command(
        &world,
        world.main,
        10,
        grandchild.created_revision,
    ))?;
    let mut invalid = command(
        &world,
        sibling.created_revision,
        vec![Change::Correct(
            alice,
            Record {
                source: Some(child_source),
                ..person("Alice in sibling")
            },
        )],
    );
    invalid.branch = sibling.id;
    assert!(matches!(store.apply(&invalid), Err(Error::NotFound)));
    invalid.changes = vec![Change::Correct(
        alice,
        Record {
            valid: ValidInterval {
                start: Some(WorldDay(9)),
                end_exclusive: None,
            },
            ..person("Too early")
        },
    )];
    assert!(matches!(store.apply(&invalid), Err(Error::BeforeFork)));
    assert_eq!(
        store.world(world.id, Scope::Author)?.revision,
        sibling.created_revision
    );
    let inherited = ViewContext {
        branch: grandchild.id,
        ..view(&world, sibling.created_revision, 13)
    };
    assert_eq!(
        store.get_object(alice, inherited)?.unwrap().record.label,
        "Alice in child"
    );
    assert_eq!(
        store
            .get_object(child_source, inherited)?
            .unwrap()
            .source_branch,
        child.id
    );
    assert!(
        store
            .get_object(
                child_source,
                ViewContext {
                    branch: sibling.id,
                    ..inherited
                }
            )?
            .is_none()
    );
    Ok(())
}

#[test]
fn fork_respects_inherited_expiration_and_keeps_corrected_overlay_gaps() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let person_id = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![Change::Create(
            person_id,
            Record {
                valid: ValidInterval {
                    start: None,
                    end_exclusive: Some(WorldDay(14)),
                },
                ..person("Alice Vale")
            },
        )],
    ))?;
    let child = store.fork(&fork_command(&world, world.main, 10, first))?;
    let read = |rev, day| ViewContext {
        branch: child.id,
        ..view(&world, rev, day)
    };
    assert!(
        store
            .get_object(person_id, read(child.created_revision, 13))?
            .is_some()
    );
    assert!(
        store
            .get_object(person_id, read(child.created_revision, 14))?
            .is_none()
    );

    let mut edit = command(
        &world,
        child.created_revision,
        vec![Change::RecordState(
            person_id,
            Record {
                valid: ValidInterval {
                    start: Some(WorldDay(12)),
                    end_exclusive: None,
                },
                ..person("Alice of the Guild")
            },
        )],
    );
    edit.branch = child.id;
    let rev = store.apply(&edit)?;
    let mut correct = command(
        &world,
        rev,
        vec![Change::Correct(
            person_id,
            Record {
                valid: ValidInterval {
                    start: Some(WorldDay(13)),
                    end_exclusive: None,
                },
                ..person("Alice of the Guild, corrected")
            },
        )],
    );
    correct.branch = child.id;
    let corrected_rev = store.apply(&correct)?;
    assert_eq!(
        store
            .get_object(person_id, read(corrected_rev, 11))?
            .unwrap()
            .record
            .label,
        "Alice Vale"
    );
    assert!(
        store
            .get_object(person_id, read(corrected_rev, 12))?
            .is_none()
    );
    assert!(store.get_object(person_id, read(rev, 12))?.is_some());
    assert!(
        store
            .get_object(person_id, read(corrected_rev, 13))?
            .is_some()
    );

    let mut backdated_event = command(
        &world,
        corrected_rev,
        vec![Change::Create(
            ObjectRef::new(world.id),
            Record {
                data: ObjectData::HistoricalEvent {
                    occurrence: EventDate::Exact(WorldDay(9)),
                },
                ..person("Backdated succession")
            },
        )],
    );
    backdated_event.branch = child.id;
    assert!(matches!(
        store.apply(&backdated_event),
        Err(Error::BeforeFork)
    ));
    assert_eq!(
        store.world(world.id, Scope::Author)?.revision,
        corrected_rev
    );
    Ok(())
}
