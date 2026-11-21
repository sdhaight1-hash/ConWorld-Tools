//! Versioned composite query definitions: typed ASTs, not cached results.
use atlas_store::{
    query::{ObjectKindFilter, QueryFilter, QuerySpec, SaveNamedLabelQuery, QUERY_SCHEMA_VERSION},
    *,
};
use tempfile::tempdir;
use uuid::Uuid;

fn record(label: &str, data: ObjectData, source: Option<ObjectRef>) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval::default(),
        data,
        source,
    }
}

fn command(world: &World, expected: AuthorRevision, changes: Vec<Change>) -> Command {
    Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Saved AST test".into(),
        rationale: "Exercise historic composite search".into(),
        scope: Scope::Author,
        changes,
    }
}

fn view(world: &World, revision: AuthorRevision) -> ViewContext {
    ViewContext {
        world: world.id,
        branch: world.main,
        revision,
        day: WorldDay(20),
        scope: Scope::Author,
    }
}

fn spec(filter: QueryFilter) -> QuerySpec {
    QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter,
        limit: 100,
    }
}

fn source_filter(source: ObjectRef) -> QueryFilter {
    QueryFilter::And(vec![
        QueryFilter::Kind(ObjectKindFilter::Claim),
        QueryFilter::HasSource(source),
        QueryFilter::Not(Box::new(QueryFilter::LabelContains("superseded".into()))),
    ])
}

fn save(
    store: &mut Store,
    world: &World,
    revision: AuthorRevision,
    id: Option<Uuid>,
    name: &str,
    filter: &QuerySpec,
) -> Result<atlas_store::query::SavedLabelQuery, Error> {
    store.save_named_label_query(SaveNamedLabelQuery {
        world_id: world.id,
        expected: revision,
        query_id: id,
        name,
        spec: filter,
        command_id: Uuid::new_v4(),
        scope: Scope::Author,
    })
}

#[test]
fn composite_saved_queries_keep_history_replay_and_stable_ids() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("saved-composite.sqlite");
    let backup = dir.path().join("backup.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Northbridge", Scope::Author)?;
    let other = store.create_world("Elsewhere", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let created = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, record("Alice", ObjectData::Person, None)),
            Change::Create(source, record("Court journal", ObjectData::Source {
                citation: "Folio 10".into(),
            }, None)),
            Change::Create(event, record("Trial", ObjectData::HistoricalEvent {
                occurrence: EventDate::Exact(WorldDay(10)),
            }, None)),
        ],
    ))?;
    let record_claim = |label: &str| record(label, ObjectData::Claim {
        subject: event,
        predicate: "testified".into(),
        statement: "Ledger was missing".into(),
        audience: ClaimAudience::Author,
    }, Some(source));
    let documented = store.apply(&command(
        &world,
        created,
        vec![Change::Create(claim, record_claim("Witness note"))],
    ))?;
    let legacy = save(&mut store, &world, documented, None,
        "Search witness labels", &spec(QueryFilter::LabelContains("Witness".into())))?;
    let composite = spec(source_filter(source));
    let saved = save(&mut store, &world, legacy.recorded_at, None,
        "Claims cited to court", &composite)?;
    assert_eq!(saved.spec, composite);
    assert_eq!(store.saved_label_queries(world.id, legacy.recorded_at, Scope::Author)?.len(), 1);
    let definitions = store.saved_label_queries(world.id, saved.recorded_at, Scope::Author)?;
    assert_eq!(definitions.len(), 2);
    let chosen = definitions.iter().find(|item| item.id == saved.id).unwrap();
    assert_eq!(chosen.spec, composite);
    assert_eq!(store.execute_query(&chosen.spec, view(&world, saved.recorded_at))?.rows[0].object, claim);
    let revision = store.apply(&command(
        &world, saved.recorded_at,
        vec![Change::Correct(claim, record_claim("Superseded note"))],
    ))?;
    assert!(store.execute_query(&chosen.spec, view(&world, revision))?.rows.is_empty(),
        "saved query must rerun against new content, never cache old results");
    assert_eq!(store.execute_query(&chosen.spec, view(&world, saved.recorded_at))?.rows[0].object, claim);
    let revised_spec = spec(QueryFilter::Or(vec![
        QueryFilter::Kind(ObjectKindFilter::Person),
        QueryFilter::ExactLabel("Superseded note".into()),
    ]));
    let revised = save(&mut store, &world, revision, Some(saved.id), "People and corrected notes", &revised_spec)?;
    assert_eq!(revised.id, saved.id);
    assert_eq!(store.saved_label_queries(world.id, saved.recorded_at, Scope::Author)?
        .iter().find(|item| item.id == saved.id).unwrap().spec, composite);
    assert_eq!(store.saved_label_queries(world.id, revised.recorded_at, Scope::Author)?
        .iter().find(|item| item.id == saved.id).unwrap().spec, revised_spec);
    let foreign = ObjectRef::new(other.id);
    assert!(matches!(
        save(&mut store, &world, revised.recorded_at, None, "Cross-world",
            &spec(QueryFilter::Not(Box::new(QueryFilter::HasSource(foreign))))),
        Err(Error::WrongWorld)
    ));
    assert_eq!(store.world(world.id, Scope::Author)?.revision, revised.recorded_at);
    assert!(matches!(
        store.save_named_label_query(SaveNamedLabelQuery {
            world_id: world.id,
            expected: revised.recorded_at,
            query_id: None,
            name: "Public save",
            spec: &composite,
            command_id: Uuid::new_v4(),
            scope: Scope::Public,
        }),
        Err(Error::CapabilityDenied)
    ));
    store.backup_to(&backup, Scope::Author)?;
    drop(store);
    let reopened = Store::open(&backup)?;
    assert_eq!(reopened.saved_label_queries(world.id, revised.recorded_at, Scope::Author)?
        .iter().find(|item| item.id == saved.id).unwrap().spec, revised_spec);
    reopened.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn schema_eleven_upgrade_preserves_legacy_query_versions() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("schema-11.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Old world", Scope::Author)?;
    let before = save(&mut store, &world, world.revision, None,
        "Old name search", &spec(QueryFilter::FullTextPhrase("Vale".into())))?;
    drop(store);
    let conn = rusqlite::Connection::open(&path)?;
    conn.execute_batch(
        "CREATE TABLE saved_query_versions_v11 (
             world_id BLOB NOT NULL,
             query_id BLOB NOT NULL,
             revision INTEGER NOT NULL CHECK(revision > 0),
             name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 200),
             schema_version INTEGER NOT NULL CHECK(schema_version = 1),
             filter_kind TEXT NOT NULL CHECK(filter_kind IN ('contains','exact','fulltext','claimtext')),
             filter_text TEXT NOT NULL CHECK(length(trim(filter_text)) > 0 AND length(filter_text) <= 800),
             result_limit INTEGER NOT NULL CHECK(result_limit BETWEEN 1 AND 500),
             PRIMARY KEY (world_id, query_id, revision),
             FOREIGN KEY (world_id, query_id) REFERENCES saved_queries(world_id, id),
             FOREIGN KEY (world_id, revision) REFERENCES author_revisions(world_id, revision)
         ) STRICT;
         INSERT INTO saved_query_versions_v11 SELECT * FROM saved_query_versions;
         DROP TABLE saved_query_versions;
         ALTER TABLE saved_query_versions_v11 RENAME TO saved_query_versions;
         PRAGMA user_version = 11;",
    )?;
    drop(conn);
    let upgraded = Store::open(&path)?;
    let saved = upgraded.saved_label_queries(world.id, before.recorded_at, Scope::Author)?;
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].id, before.id);
    assert_eq!(saved[0].spec, before.spec);
    upgraded.verify_foreign_keys(Scope::Author)?;
    Ok(())
}
