//! Phase 3 saved label query versions are author history, never cached query results.
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
        author: "Saved query test".into(),
        rationale: "Seed query fixture".into(),
        scope: Scope::Author,
        changes,
    }
}

fn search(text: &str) -> QuerySpec {
    QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter: QueryFilter::FullTextPhrase(text.into()),
        limit: 20,
    }
}

fn view(world: &World, revision: AuthorRevision) -> ViewContext {
    ViewContext {
        world: world.id,
        branch: world.main,
        revision,
        day: WorldDay(0),
        scope: Scope::Author,
    }
}

#[test]
fn saved_queries_keep_stable_ids_and_prior_author_revisions() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("saved.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let other = store.create_world("Other", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![Change::Create(
            alice,
            Record {
                label: "Alice Vale".into(),
                valid: ValidInterval::default(),
                data: ObjectData::Person,
                source: None,
            },
        )],
    ))?;
    let save_id = Uuid::new_v4();
    let saved = store.save_named_label_query(
        world.id,
        first,
        None,
        "Search Alice",
        &search("Alice Vale"),
        save_id,
        Scope::Author,
    )?;
    assert_eq!(saved.recorded_at, AuthorRevision(2));
    assert_eq!(
        store
            .saved_label_queries(world.id, first, Scope::Author)?
            .len(),
        0
    );
    let original = store.saved_label_queries(world.id, saved.recorded_at, Scope::Author)?;
    assert_eq!(original.len(), 1);
    assert_eq!(original[0].id, saved.id);
    assert_eq!(
        store
            .execute_query(&original[0].spec, view(&world, saved.recorded_at))?
            .rows[0]
            .object,
        alice
    );
    assert!(matches!(
        store.save_named_label_query(
            world.id,
            saved.recorded_at,
            None,
            "Duplicate",
            &search("Alice Vale"),
            save_id,
            Scope::Author,
        ),
        Err(Error::DuplicateCommand(_))
    ));

    let revised = store.save_named_label_query(
        world.id,
        saved.recorded_at,
        Some(saved.id),
        "Search renamed",
        &search("Beatrice Thorne"),
        Uuid::new_v4(),
        Scope::Author,
    )?;
    assert_eq!(revised.id, saved.id);
    assert_eq!(revised.recorded_at, AuthorRevision(3));
    assert_eq!(
        store.saved_label_queries(world.id, saved.recorded_at, Scope::Author)?[0].name,
        "Search Alice"
    );
    let current = store.saved_label_queries(world.id, revised.recorded_at, Scope::Author)?;
    assert_eq!(current[0].name, "Search renamed");
    assert_eq!(current[0].spec, search("Beatrice Thorne"));
    assert!(matches!(
        store.save_named_label_query(
            other.id,
            other.revision,
            Some(saved.id),
            "Cross-world",
            &search("Alice"),
            Uuid::new_v4(),
            Scope::Author,
        ),
        Err(Error::NotFound)
    ));
    assert!(matches!(
        store.saved_label_queries(world.id, revised.recorded_at, Scope::Public),
        Err(Error::CapabilityDenied)
    ));
    assert!(matches!(
        store.save_named_label_query(
            world.id,
            revised.recorded_at,
            None,
            "Public",
            &search("Alice"),
            Uuid::new_v4(),
            Scope::Public,
        ),
        Err(Error::CapabilityDenied)
    ));
    assert!(matches!(
        store.save_named_label_query(
            world.id,
            revised.recorded_at,
            None,
            "Empty search",
            &search(" "),
            Uuid::new_v4(),
            Scope::Author,
        ),
        Err(Error::InvalidInput(_))
    ));
    assert_eq!(
        store.world(world.id, Scope::Author)?.revision,
        revised.recorded_at
    );

    drop(store);
    let store = Store::open(&path)?;
    assert_eq!(
        store.saved_label_queries(world.id, revised.recorded_at, Scope::Author)?[0].id,
        saved.id
    );
    Ok(())
}

#[test]
fn schema_eight_upgrade_preserves_world_data_and_initializes_saved_query_tables()
-> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("old.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Legacy", Scope::Author)?;
    let object = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![Change::Create(
            object,
            Record {
                label: "Legacy Writer".into(),
                valid: ValidInterval::default(),
                data: ObjectData::Person,
                source: None,
            },
        )],
    ))?;
    drop(store);
    let connection = rusqlite::Connection::open(&path)?;
    connection.execute_batch(
        "DROP TABLE saved_query_versions;
         DROP TABLE saved_queries;
         PRAGMA user_version = 8;",
    )?;
    drop(connection);
    let store = Store::open(&path)?;
    assert!(
        store
            .saved_label_queries(world.id, first, Scope::Author)?
            .is_empty()
    );
    assert_eq!(
        store
            .execute_query(&search("Legacy Writer"), view(&world, first))?
            .rows[0]
            .object,
        object
    );
    store.verify_foreign_keys(Scope::Author)?;
    Ok(())
}
