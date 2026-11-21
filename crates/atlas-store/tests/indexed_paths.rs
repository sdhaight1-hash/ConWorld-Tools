//! Phase 3 traversal uses authoritative versioned endpoint candidates.
use atlas_store::{
    query::{Direction, QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec},
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
        author: "Traversal fixture".into(),
        rationale: "Verify historical endpoint indexing".into(),
        scope: Scope::Author,
        changes,
    }
}

fn person(label: String) -> Record {
    Record {
        label,
        valid: ValidInterval::default(),
        data: ObjectData::Person,
        source: None,
    }
}

fn relationship(subject: ObjectRef, target: ObjectRef, end: Option<i64>) -> Record {
    Record {
        label: "Friendship".into(),
        valid: ValidInterval {
            start: Some(WorldDay(0)),
            end_exclusive: end.map(WorldDay),
        },
        source: None,
        data: ObjectData::Relationship {
            subject,
            target,
            kind: "friendship".into(),
            trust: None,
            tags: vec![],
        },
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

fn path(start: ObjectRef, direction: Direction, max_depth: u8, max_visited: usize) -> QuerySpec {
    QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter: QueryFilter::RelationshipPath {
            start,
            kind: "Friendship".into(),
            direction,
            max_depth,
            max_visited,
        },
        limit: 100,
    }
}

#[test]
fn indexed_paths_ignore_unrelated_population_and_obey_historical_context() -> Result<(), Error> {
    let dir = tempdir()?;
    let db = dir.path().join("path.sqlite");
    let backup = dir.path().join("backup.sqlite");
    let mut store = Store::open(&db)?;
    let world = store.create_world("Valley", Scope::Author)?;
    let other = store.create_world("Elsewhere", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let bob = ObjectRef::new(world.id);
    let carol = ObjectRef::new(world.id);
    let first_edge = ObjectRef::new(world.id);
    let second_edge = ObjectRef::new(world.id);
    let mut changes = vec![
        Change::Create(alice, person("Alice".into())),
        Change::Create(bob, person("Bob".into())),
        Change::Create(carol, person("Carol".into())),
        Change::Create(first_edge, relationship(alice, bob, Some(10))),
        Change::Create(second_edge, relationship(bob, carol, None)),
    ];
    for index in 0..2_010 {
        changes.push(Change::Create(
            ObjectRef::new(world.id),
            person(format!("Other resident {index}")),
        ));
    }
    let initial = store.apply(&command(&world, world.revision, changes))?;
    let current = view(&world, initial, 8);
    assert!(matches!(
        store.objects_at(current, 2_000),
        Err(Error::QueryBudgetExceeded)
    ));
    let outgoing = path(alice, Direction::Outgoing, 2, 3);
    let incoming = path(carol, Direction::Incoming, 2, 3);
    let both = path(bob, Direction::Both, 1, 3);
    let ids = |result: atlas_store::query::QueryResult| {
        result
            .rows
            .into_iter()
            .map(|row| row.object)
            .collect::<Vec<_>>()
    };
    let first = store.execute_query(&outgoing, current)?;
    assert!(!first.truncated);
    assert_eq!(ids(first), vec![bob, carol]);
    assert_eq!(
        ids(store.execute_query(&incoming, current)?),
        vec![alice, bob]
    );
    assert_eq!(
        ids(store.execute_query(&both, current)?),
        vec![alice, carol]
    );
    let bounded = store.execute_query(&path(alice, Direction::Outgoing, 2, 2), current)?;
    assert!(bounded.truncated);
    assert_eq!(ids(bounded), vec![bob]);
    let depth_limited = store.execute_query(&path(alice, Direction::Outgoing, 1, 3), current)?;
    assert!(!depth_limited.truncated);
    assert_eq!(ids(depth_limited), vec![bob]);
    let display_limited = store.execute_query(
        &QuerySpec {
            limit: 1,
            ..outgoing.clone()
        },
        current,
    )?;
    assert!(display_limited.truncated);
    assert_eq!(ids(display_limited), vec![bob]);
    assert!(
        store
            .execute_query(&outgoing, view(&world, initial, 10))?
            .rows
            .is_empty()
    );
    assert!(matches!(
        store.execute_query(
            &outgoing,
            ViewContext {
                scope: Scope::Public,
                ..current
            }
        ),
        Err(Error::CapabilityDenied)
    ));
    assert!(matches!(
        store.execute_query(
            &path(ObjectRef::new(other.id), Direction::Both, 1, 2),
            current
        ),
        Err(Error::WrongWorld)
    ));
    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: WorldDay(4),
        expected: initial,
        id: Uuid::new_v4(),
        author: "Traversal fixture".into(),
        rationale: "Pin earlier relation state".into(),
        scope: Scope::Author,
    })?;
    let corrected = store.apply(&command(
        &world,
        fork.created_revision,
        vec![Change::Correct(
            first_edge,
            relationship(alice, bob, Some(5)),
        )],
    ))?;
    assert!(
        store
            .execute_query(&outgoing, view(&world, corrected, 8))?
            .rows
            .is_empty()
    );
    assert_eq!(
        ids(store.execute_query(&outgoing, current)?),
        vec![bob, carol]
    );
    assert_eq!(
        ids(store.execute_query(
            &outgoing,
            ViewContext {
                branch: fork.id,
                ..view(&world, corrected, 8)
            }
        )?),
        vec![bob, carol]
    );
    store.backup_to(&backup, Scope::Author)?;
    drop(store);

    let conn = rusqlite::Connection::open(&db)?;
    conn.execute_batch("DROP INDEX relationship_target_idx; PRAGMA user_version = 15;")?;
    drop(conn);
    let upgraded = Store::open(&db)?;
    assert_eq!(
        ids(upgraded.execute_query(&incoming, current)?),
        vec![alice, bob]
    );
    upgraded.verify_foreign_keys(Scope::Author)?;
    let reopened = Store::open(&backup)?;
    assert_eq!(
        ids(reopened.execute_query(&incoming, current)?),
        vec![alice, bob]
    );
    reopened.verify_foreign_keys(Scope::Author)?;
    Ok(())
}
