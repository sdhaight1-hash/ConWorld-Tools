//! Phase 3 FTS5: a rebuildable, version-aware author search index.
use atlas_store::{
    query::{QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec},
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
        author: "FTS test".into(),
        rationale: "Validate historical indexed search".into(),
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

fn phrase(text: &str) -> QuerySpec {
    QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter: QueryFilter::FullTextPhrase(text.into()),
        limit: 100,
    }
}

fn ids(
    store: &Store,
    world: &World,
    revision: AuthorRevision,
    text: &str,
) -> Result<Vec<ObjectRef>, Error> {
    Ok(store
        .execute_query(&phrase(text), view(world, revision, 20))?
        .rows
        .into_iter()
        .map(|item| item.object)
        .collect())
}

#[test]
fn fts_phrase_indexes_current_version_without_resurrecting_corrections_or_other_worlds()
-> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let other = store.create_world("Not Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let duplicate = ObjectRef::new(world.id);
    let future = ObjectRef::new(world.id);
    let outside = ObjectRef::new(other.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, person("Alice Vale")),
            Change::Create(duplicate, person("Alice Vale")),
            Change::Create(
                future,
                Record {
                    valid: ValidInterval {
                        start: Some(WorldDay(40)),
                        end_exclusive: None,
                    },
                    ..person("Alice Vale")
                },
            ),
        ],
    ))?;
    store.apply(&command(
        &other,
        other.revision,
        vec![Change::Create(outside, person("Alice Vale"))],
    ))?;
    let initial = ids(&store, &world, first, "Alice Vale")?;
    assert_eq!(initial.len(), 2);
    assert!(initial.contains(&alice) && initial.contains(&duplicate));
    assert!(ids(&store, &world, first, "Alice")?.contains(&alice));
    assert!(ids(&store, &world, first, "Alice OR Vale")?.is_empty());
    assert!(ids(&store, &world, first, "\"\" OR Vale")?.is_empty());
    assert!(ids(&store, &world, AuthorRevision(0), "Alice")?.is_empty());

    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: WorldDay(10),
        expected: first,
        id: Uuid::new_v4(),
        author: "FTS test".into(),
        rationale: "Freeze earlier history".into(),
        scope: Scope::Author,
    })?;
    let second = store.apply(&command(
        &world,
        fork.created_revision,
        vec![Change::Correct(alice, person("Beatrice Thorne"))],
    ))?;
    assert_eq!(ids(&store, &world, first, "Alice Vale")?.len(), 2);
    assert_eq!(ids(&store, &world, second, "Alice Vale")?, vec![duplicate]);
    assert_eq!(ids(&store, &world, second, "Beatrice Thorne")?, vec![alice]);
    let future_hits = store.execute_query(&phrase("Alice Vale"), view(&world, second, 41))?;
    assert_eq!(future_hits.rows.len(), 2);
    let child = ViewContext {
        branch: fork.id,
        ..view(&world, second, 20)
    };
    assert_eq!(
        store
            .execute_query(&phrase("Alice Vale"), child)?
            .rows
            .len(),
        2,
        "later main-line corrections must not rewrite a pinned fork"
    );
    let invalid = ObjectRef::new(world.id);
    let faulty = store.apply(&command(
        &world,
        second,
        vec![
            Change::Correct(alice, person("Eve Deleted")),
            Change::Create(
                invalid,
                Record {
                    valid: ValidInterval {
                        start: Some(WorldDay(4)),
                        end_exclusive: Some(WorldDay(4)),
                    },
                    ..person("Invalid record")
                },
            ),
        ],
    ));
    assert!(matches!(faulty, Err(Error::InvalidInterval)));
    assert!(ids(&store, &world, second, "Eve Deleted")?.is_empty());
    assert_eq!(ids(&store, &world, second, "Beatrice Thorne")?, vec![alice]);

    let public = ViewContext {
        scope: Scope::Public,
        ..view(&world, second, 20)
    };
    assert!(matches!(
        store.execute_query(&phrase("Alice Vale"), public),
        Err(Error::CapabilityDenied)
    ));
    assert!(matches!(
        store.execute_query(&phrase("  "), view(&world, second, 20)),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        store.execute_query(
            &QuerySpec {
                limit: 0,
                ..phrase("Alice")
            },
            view(&world, second, 20)
        ),
        Err(Error::InvalidInput(_))
    ));
    Ok(())
}

#[test]
fn fts_index_rebuild_backup_and_v7_migration_preserve_historical_results() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("world.sqlite");
    let backup = dir.path().join("backup.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![Change::Create(alice, person("Archive Researcher"))],
    ))?;
    let before = ids(&store, &world, first, "Archive Researcher")?;
    assert_eq!(before, vec![alice]);
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(ids(&store, &world, first, "Archive Researcher")?, before);
    assert!(matches!(
        store.rebuild_search_index(Scope::Public),
        Err(Error::CapabilityDenied)
    ));
    store.backup_to(&backup, Scope::Author)?;
    drop(store);
    let reopened = Store::open(&backup)?;
    assert_eq!(ids(&reopened, &world, first, "Archive Researcher")?, before);
    drop(reopened);

    // Simulate a populated schema 7 file: the next open must backfill
    // the FTS index without rewriting the canonical author history.
    let conn = rusqlite::Connection::open(&path)?;
    conn.execute_batch(
        "DROP TABLE saved_query_versions;
         DROP TABLE saved_queries;
         DROP TRIGGER object_versions_search_ai;
         DROP TRIGGER object_versions_search_au;
         DROP TABLE object_search_fts;
         DROP TABLE object_search_documents;
         PRAGMA user_version = 7;",
    )?;
    drop(conn);
    let upgraded = Store::open(&path)?;
    assert_eq!(ids(&upgraded, &world, first, "Archive Researcher")?, before);
    upgraded.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn fts_narrow_queries_work_past_the_temporary_two_thousand_object_listing_limit()
-> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Large world", Scope::Author)?;
    let needle = ObjectRef::new(world.id);
    let mut changes = vec![Change::Create(needle, person("Rare Obsidian Navigator"))];
    for index in 0..2_005 {
        changes.push(Change::Create(
            ObjectRef::new(world.id),
            person(&format!("Ordinary inhabitant {index}")),
        ));
    }
    let revision = store.apply(&command(&world, world.revision, changes))?;
    assert!(matches!(
        store.execute_query(
            &QuerySpec {
                filter: QueryFilter::LabelContains("Obsidian".into()),
                ..phrase("Obsidian")
            },
            view(&world, revision, 0),
        ),
        Err(Error::QueryBudgetExceeded)
    ));
    assert_eq!(ids(&store, &world, revision, "Obsidian")?, vec![needle]);
    Ok(())
}
