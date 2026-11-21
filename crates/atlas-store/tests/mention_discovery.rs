use atlas_store::*;
use uuid::Uuid;

fn command(world: &World, expected: AuthorRevision, changes: Vec<Change>) -> Command {
    Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Discovery tester".into(),
        rationale: "Verify mention candidates without making world facts".into(),
        scope: Scope::Author,
        changes,
    }
}

fn record(label: &str, data: ObjectData) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval::default(),
        data,
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
fn collision_candidates_are_separate_and_never_assert_membership() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Valley", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let duplicate = ObjectRef::new(world.id);
    let ann = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let citation = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, record("Eleanor Vale", ObjectData::Person)),
            Change::Create(duplicate, record("Eleanor Vale", ObjectData::Person)),
            Change::Create(ann, record("Ann", ObjectData::Person)),
            Change::Create(
                citation,
                record(
                    "Guild archive",
                    ObjectData::Source {
                        citation: "Folio 1".into(),
                    },
                ),
            ),
            Change::Create(
                event,
                record(
                    "Council meeting",
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(10)),
                    },
                ),
            ),
        ],
    ))?;
    let statement = "ELEANOR VALE met the Guild. Eleanor Vale spoke with Joanne. Ann watched.";
    let second = store.apply(&command(
        &world,
        first,
        vec![Change::Create(
            claim,
            Record {
                valid: ValidInterval {
                    start: Some(WorldDay(10)),
                    end_exclusive: None,
                },
                source: Some(citation),
                ..record(
                    "Disputed eyewitness statement",
                    ObjectData::Claim {
                        subject: event,
                        predicate: "attended".into(),
                        statement: statement.into(),
                        audience: ClaimAudience::Author,
                    },
                )
            },
        )],
    ))?;

    let result = store.discover_mentions(view(&world, second, 10), 100)?;
    assert_eq!(result.revision, second);
    assert!(!result.truncated);
    assert_eq!(result.rows.len(), 5);
    let eleanor: Vec<_> = result
        .rows
        .iter()
        .filter(|item| item.target != ann)
        .collect();
    assert_eq!(eleanor.len(), 4);
    assert!(
        eleanor
            .iter()
            .all(|row| row.collisions == 2 && row.source.object == claim)
    );
    assert!(eleanor.iter().any(|row| row.target == alice));
    assert!(eleanor.iter().any(|row| row.target == duplicate));
    let ann_hits: Vec<_> = result.rows.iter().filter(|row| row.target == ann).collect();
    assert_eq!(ann_hits.len(), 1);
    assert_eq!(
        &statement[ann_hits[0].start_byte..ann_hits[0].end_byte],
        "Ann"
    );
    assert_eq!(ann_hits[0].collisions, 1);
    assert!(
        store
            .discover_mentions(view(&world, first, 10), 100)?
            .rows
            .is_empty()
    );
    assert!(
        store
            .discover_mentions(view(&world, second, 9), 100)?
            .rows
            .is_empty()
    );
    assert!(
        store
            .execute_query(
                &atlas_store::query::QuerySpec {
                    schema_version: atlas_store::query::QUERY_SCHEMA_VERSION,
                    filter: atlas_store::query::QueryFilter::Backlinks(alice),
                    limit: 20,
                },
                view(&world, second, 10),
            )?
            .rows
            .is_empty(),
        "mere detected text must not make an approved backlink"
    );
    let tiny = store.discover_mentions(view(&world, second, 10), 2)?;
    assert_eq!(tiny.rows.len(), 2);
    assert!(tiny.truncated);
    assert!(matches!(
        store.discover_mentions(
            ViewContext {
                scope: Scope::Public,
                ..view(&world, second, 10)
            },
            100
        ),
        Err(Error::CapabilityDenied)
    ));
    assert!(matches!(
        store.discover_mentions(view(&world, second, 10), 0),
        Err(Error::InvalidInput(_))
    ));

    // Correcting one identity leaves the other candidate; old recorded
    // revision still reports both stable IDs with no silent reassignment.
    let third = store.apply(&command(
        &world,
        second,
        vec![Change::Correct(
            alice,
            record("Eleanor Thorne", ObjectData::Person),
        )],
    ))?;
    let current = store.discover_mentions(view(&world, third, 10), 100)?;
    assert_eq!(
        current
            .rows
            .iter()
            .filter(|row| row.target == duplicate)
            .count(),
        2
    );
    assert!(!current.rows.iter().any(|row| row.target == alice));
    assert!(
        current
            .rows
            .iter()
            .filter(|row| row.target == duplicate)
            .all(|row| row.collisions == 1)
    );
    assert_eq!(
        store
            .discover_mentions(view(&world, second, 10), 100)?
            .rows
            .len(),
        5
    );

    Ok(())
}

#[test]
fn discovery_honors_world_boundaries_and_returns_original_claim_versions() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let first = store.create_world("One", Scope::Author)?;
    let other = store.create_world("Two", Scope::Author)?;
    let person = ObjectRef::new(other.id);
    let rev = store.apply(&command(
        &other,
        other.revision,
        vec![Change::Create(
            person,
            record("Morgan Vale", ObjectData::Person),
        )],
    ))?;
    assert_eq!(rev, AuthorRevision(1));
    let claim = ObjectRef::new(first.id);
    let source = ObjectRef::new(first.id);
    let event = ObjectRef::new(first.id);
    let revision = store.apply(&command(
        &first,
        first.revision,
        vec![
            Change::Create(
                source,
                record(
                    "Diary",
                    ObjectData::Source {
                        citation: "Diary #12".into(),
                    },
                ),
            ),
            Change::Create(
                event,
                record(
                    "Market",
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(0)),
                    },
                ),
            ),
        ],
    ))?;
    let written = store.apply(&command(
        &first,
        revision,
        vec![Change::Create(
            claim,
            Record {
                source: Some(source),
                ..record(
                    "Note",
                    ObjectData::Claim {
                        subject: event,
                        predicate: "mention".into(),
                        statement: "Morgan Vale arrived.".into(),
                        audience: ClaimAudience::Author,
                    },
                )
            },
        )],
    ))?;
    assert!(
        store
            .discover_mentions(view(&first, written, 0), 100)?
            .rows
            .is_empty()
    );
    assert!(
        store
            .discover_mentions(view(&other, rev, 0), 100)?
            .rows
            .is_empty()
    );
    Ok(())
}
