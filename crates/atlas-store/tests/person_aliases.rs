//! Phase 3 versioned person aliases participate in candidate discovery and exact historical lookup.
use atlas_store::{
    discovery::{MentionDisposition, ReviewMention, SavePersonAlias},
    query::{ObjectKindFilter, QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec, SaveNamedLabelQuery},
    *,
};
use tempfile::tempdir;
use uuid::Uuid;

fn command(world: &World, expected: AuthorRevision, changes: Vec<Change>) -> Command {
    Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Alias fixture".into(),
        rationale: "Source-backed nickname disambiguation".into(),
        scope: Scope::Author,
        changes,
    }
}
fn record(label: &str, data: ObjectData, source: Option<ObjectRef>) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval::default(),
        data,
        source,
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
fn save(
    store: &mut Store,
    world: &World,
    expected: AuthorRevision,
    person: ObjectRef,
    alias_id: Option<Uuid>,
    settings: (&str, bool, i64, Option<i64>),
) -> Result<atlas_store::discovery::PersonAlias, Error> {
    let (label, active, from, until) = settings;
    store.save_person_alias(SavePersonAlias {
        world: world.id,
        person,
        day: WorldDay(20),
        expected,
        alias_id,
        label,
        starts_at: Some(WorldDay(from)),
        ends_before: until.map(WorldDay),
        active,
        command_id: Uuid::new_v4(),
        scope: Scope::Author,
    })
}
fn backlinks(
    store: &Store,
    world: &World,
    rev: AuthorRevision,
    target: ObjectRef,
) -> Result<Vec<ObjectRef>, Error> {
    Ok(store
        .execute_query(
            &QuerySpec {
                schema_version: QUERY_SCHEMA_VERSION,
                filter: QueryFilter::Backlinks(target),
                limit: 100,
            },
            view(world, rev, 20),
        )?
        .rows
        .into_iter()
        .map(|row| row.object)
        .collect())
}

#[test]
fn aliases_preserve_revision_and_valid_day_collisions_with_reviewed_backlinks() -> Result<(), Error>
{
    let dir = tempdir()?;
    let path = dir.path().join("aliases.sqlite");
    let backup = dir.path().join("backup.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Forest", Scope::Author)?;
    let other = store.create_world("Other", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let bob = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let initial = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, record("Alice Rowan", ObjectData::Person, None)),
            Change::Create(bob, record("Bob Vale", ObjectData::Person, None)),
            Change::Create(
                event,
                record(
                    "Council",
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(10)),
                    },
                    None,
                ),
            ),
            Change::Create(
                source,
                record(
                    "Council notes",
                    ObjectData::Source {
                        citation: "Folio 11".into(),
                    },
                    None,
                ),
            ),
            Change::Create(
                claim,
                record(
                    "Secret witness",
                    ObjectData::Claim {
                        subject: event,
                        predicate: "reported".into(),
                        statement: "The Magpie met the council.".into(),
                        audience: ClaimAudience::Author,
                    },
                    Some(source),
                ),
            ),
        ],
    ))?;
    assert!(
        store
            .discover_mentions(view(&world, initial, 20), 100)?
            .rows
            .is_empty()
    );
    let first = save(
        &mut store,
        &world,
        initial,
        alice,
        None,
        ("Magpie", true, 10, Some(40)),
    )?;
    let second = save(
        &mut store,
        &world,
        first.recorded_at,
        bob,
        None,
        ("Magpie", true, 10, None),
    )?;
    assert!(
        store
            .discover_mentions(view(&world, second.recorded_at, 9), 100)?
            .rows
            .is_empty()
    );
    let candidates = store.discover_mentions(view(&world, second.recorded_at, 20), 100)?;
    assert_eq!(candidates.rows.len(), 2);
    assert!(
        candidates
            .rows
            .iter()
            .all(|c| c.collisions == 2 && c.matched_label == "Magpie")
    );
    assert!(
        candidates
            .rows
            .iter()
            .any(|c| c.target == alice && c.target_label == "Alice Rowan")
    );
    assert!(
        candidates
            .rows
            .iter()
            .any(|c| c.target == bob && c.target_label == "Bob Vale")
    );
    let selected = candidates.rows.iter().find(|c| c.target == alice).unwrap();
    let approved = store.review_mention(ReviewMention {
        view: view(&world, second.recorded_at, 20),
        expected: second.recorded_at,
        source: selected.source,
        target: alice,
        start_byte: selected.start_byte,
        end_byte: selected.end_byte,
        decision: MentionDisposition::Approved,
        command_id: Uuid::new_v4(),
    })?;
    assert_eq!(backlinks(&store, &world, approved, alice)?, vec![claim]);
    assert!(!backlinks(&store, &world, approved, bob)?.contains(&claim));
    let renamed = save(
        &mut store,
        &world,
        approved,
        alice,
        Some(first.id),
        ("Bluebird", true, 10, None),
    )?;
    let latest = store.discover_mentions(view(&world, renamed.recorded_at, 20), 100)?;
    assert_eq!(latest.rows.len(), 1);
    assert_eq!(latest.rows[0].target, bob);
    assert_eq!(
        backlinks(&store, &world, renamed.recorded_at, alice)?,
        vec![claim],
        "an already-approved stable ID link must survive a changed alias"
    );
    let historical = store.discover_mentions(view(&world, approved, 20), 100)?;
    assert_eq!(
        historical.rows.len(),
        2,
        "old author revision retains old aliases"
    );
    assert_eq!(
        historical
            .rows
            .iter()
            .find(|c| c.target == alice)
            .unwrap()
            .disposition,
        Some(MentionDisposition::Approved)
    );
    let stopped = save(
        &mut store,
        &world,
        renamed.recorded_at,
        bob,
        Some(second.id),
        ("Magpie", false, 10, None),
    )?;
    assert!(
        store
            .discover_mentions(view(&world, stopped.recorded_at, 20), 100)?
            .rows
            .is_empty()
    );
    assert!(
        store
            .person_aliases(view(&world, stopped.recorded_at, 20))?
            .iter()
            .any(|a| a.id == second.id && !a.active)
    );
    assert!(
        store
            .person_aliases(view(&world, stopped.recorded_at, 20))?
            .iter()
            .any(|a| a.id == first.id && a.label == "Bluebird")
    );
    assert!(matches!(
        save(
            &mut store,
            &world,
            stopped.recorded_at,
            ObjectRef::new(other.id),
            None,
            ("Foreign", true, 0, None)
        ),
        Err(Error::WrongWorld)
    ));
    assert_eq!(
        store.world(world.id, Scope::Author)?.revision,
        stopped.recorded_at
    );
    assert!(matches!(
        store.person_aliases(ViewContext {
            scope: Scope::Public,
            ..view(&world, stopped.recorded_at, 20)
        }),
        Err(Error::CapabilityDenied)
    ));
    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: WorldDay(10),
        expected: stopped.recorded_at,
        id: Uuid::new_v4(),
        author: "Alias fixture".into(),
        rationale: "Isolate metadata".into(),
        scope: Scope::Author,
    })?;
    assert!(
        store
            .person_aliases(ViewContext {
                branch: fork.id,
                ..view(&world, fork.created_revision, 20)
            })?
            .is_empty(),
        "main-line alias metadata must never leak to a fork"
    );
    assert!(matches!(
        store.save_person_alias(SavePersonAlias {
            world: world.id,
            person: alice,
            day: WorldDay(20),
            expected: fork.created_revision,
            alias_id: None,
            label: "Fork name",
            starts_at: None,
            ends_before: None,
            active: true,
            command_id: Uuid::new_v4(),
            scope: Scope::Public,
        }),
        Err(Error::CapabilityDenied)
    ));
    store.backup_to(&backup, Scope::Author)?;
    drop(store);
    let reopened = Store::open(&backup)?;
    assert_eq!(
        reopened
            .person_aliases(view(&world, stopped.recorded_at, 20))?
            .len(),
        2
    );
    reopened.verify_foreign_keys(Scope::Author)?;
    drop(reopened);

    let conn = rusqlite::Connection::open(&path)?;
    conn.execute_batch(
        "DROP TABLE person_alias_name_keys;
         DROP TABLE person_alias_versions;
         DROP TABLE person_aliases;
         PRAGMA user_version = 13;",
    )?;
    drop(conn);
    let upgraded = Store::open(&path)?;
    assert!(
        upgraded
            .person_aliases(view(&world, stopped.recorded_at, 20))?
            .is_empty()
    );
    upgraded.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn alias_input_rejects_invalid_intervals_labels_and_stale_writes() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("One", Scope::Author)?;
    let person = ObjectRef::new(world.id);
    let created = store.apply(&command(
        &world,
        world.revision,
        vec![Change::Create(
            person,
            record("Rowan", ObjectData::Person, None),
        )],
    ))?;
    assert!(matches!(
        save(
            &mut store,
            &world,
            created,
            person,
            None,
            (" ", true, 0, None)
        ),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        save(
            &mut store,
            &world,
            created,
            person,
            None,
            ("Old name", true, 50, Some(50))
        ),
        Err(Error::InvalidInterval)
    ));
    assert!(matches!(
        save(
            &mut store,
            &world,
            AuthorRevision(0),
            person,
            None,
            ("Old name", true, 0, None)
        ),
        Err(Error::RevisionConflict { .. })
    ));
    assert_eq!(store.world(world.id, Scope::Author)?.revision, created);
    Ok(())
}

#[test]
fn repeated_aliases_do_not_exhaust_distinct_candidate_budget() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Duplicate aliases", Scope::Author)?;
    let person = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let mut revision = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(person, record("Magpie", ObjectData::Person, None)),
            Change::Create(
                event,
                record(
                    "Council",
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(10)),
                    },
                    None,
                ),
            ),
            Change::Create(
                source,
                record(
                    "Notes",
                    ObjectData::Source {
                        citation: "Folio 1".into(),
                    },
                    None,
                ),
            ),
            Change::Create(
                claim,
                record(
                    "Witness",
                    ObjectData::Claim {
                        subject: event,
                        predicate: "reported".into(),
                        statement: "Magpie ".repeat(21),
                        audience: ClaimAudience::Author,
                    },
                    Some(source),
                ),
            ),
        ],
    ))?;
    for _ in 0..100 {
        revision = save(
            &mut store,
            &world,
            revision,
            person,
            None,
            ("MAGPIE", true, 0, None),
        )?
        .recorded_at;
    }
    let result = store.discover_mentions(view(&world, revision, 20), 100)?;
    assert_eq!(result.rows.len(), 21);
    assert!(!result.truncated);
    assert!(result.rows.iter().all(|candidate| {
        candidate.target == person
            && candidate.collisions == 1
            && candidate.matched_label == "Magpie"
    }));
    Ok(())
}

#[test]
fn indexed_aliases_preserve_collisions_and_historical_versions() -> Result<(), Error> {
    let dir = tempdir()?;
    let db = dir.path().join("alias-index.sqlite");
    let mut store = Store::open(&db)?;
    let world = store.create_world("Alias query", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let bob = ObjectRef::new(world.id);
    let mut changes = vec![
        Change::Create(alice, record("Alice", ObjectData::Person, None)),
        Change::Create(bob, record("Bob", ObjectData::Person, None)),
    ];
    for i in 0..2_005 {
        changes.push(Change::Create(
            ObjectRef::new(world.id),
            record(&format!("Resident {i}"), ObjectData::Person, None),
        ));
    }
    let initial = store.apply(&command(&world, world.revision, changes))?;
    let a = save(
        &mut store,
        &world,
        initial,
        alice,
        None,
        ("Étoile", true, 10, Some(30)),
    )?;
    let b = save(
        &mut store,
        &world,
        a.recorded_at,
        bob,
        None,
        ("ÉTOILE", true, 10, None),
    )?;
    let spec = QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter: QueryFilter::ExactAlias(" étoile ".into()),
        limit: 100,
    };
    let ids = |store: &Store, revision, day| -> Result<Vec<ObjectRef>, Error> {
        Ok(store
            .execute_query(&spec, view(&world, revision, day))?
            .rows
            .into_iter()
            .map(|row| row.object)
            .collect())
    };
    assert!(ids(&store, b.recorded_at, 9)?.is_empty());
    assert_eq!(ids(&store, b.recorded_at, 20)?, vec![alice, bob]);
    assert_eq!(ids(&store, b.recorded_at, 30)?, vec![bob]);
    let next = save(
        &mut store,
        &world,
        b.recorded_at,
        alice,
        Some(a.id),
        ("Different", true, 10, None),
    )?;
    assert_eq!(ids(&store, next.recorded_at, 20)?, vec![bob]);
    assert_eq!(ids(&store, b.recorded_at, 20)?, vec![alice, bob]);
    let composite = QuerySpec {
        filter: QueryFilter::And(vec![
            QueryFilter::ExactAlias("Étoile".into()),
            QueryFilter::Kind(ObjectKindFilter::Person),
            QueryFilter::Not(Box::new(QueryFilter::ExactLabel("Alice".into()))),
        ]),
        ..spec.clone()
    };
    let saved = store.save_named_label_query(SaveNamedLabelQuery {
        world_id: world.id,
        expected: next.recorded_at,
        query_id: None,
        name: "Other Étoile",
        spec: &composite,
        command_id: Uuid::new_v4(),
        scope: Scope::Author,
    })?;
    assert_eq!(
        store
            .execute_query(&composite, view(&world, b.recorded_at, 20))?
            .rows[0]
            .object,
        bob
    );
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(ids(&store, b.recorded_at, 20)?, vec![alice, bob]);
    assert!(matches!(
        store.execute_query(
            &spec,
            ViewContext {
                scope: Scope::Public,
                ..view(&world, next.recorded_at, 20)
            }
        ),
        Err(Error::CapabilityDenied)
    ));
    drop(store);
    let conn = rusqlite::Connection::open(&db)?;
    conn.execute_batch("DROP TABLE person_alias_name_keys; PRAGMA user_version = 16;")?;
    drop(conn);
    let reopened = Store::open(&db)?;
    assert_eq!(ids(&reopened, b.recorded_at, 20)?, vec![alice, bob]);
    assert_eq!(ids(&reopened, next.recorded_at, 20)?, vec![bob]);
    let definitions = reopened.saved_label_queries(world.id, saved.recorded_at, Scope::Author)?;
    assert_eq!(definitions[0].id, saved.id);
    assert_eq!(definitions[0].spec, composite);
    assert_eq!(
        reopened
            .execute_query(&definitions[0].spec, view(&world, saved.recorded_at, 20))?
            .rows[0]
            .object,
        bob
    );
    reopened.verify_foreign_keys(Scope::Author)?;
    Ok(())
}
