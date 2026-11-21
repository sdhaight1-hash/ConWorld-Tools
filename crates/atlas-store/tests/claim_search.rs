//! Phase 3: indexed, author-scoped historical claim statement search.
use atlas_store::{
    query::{QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec, SaveNamedLabelQuery},
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
        author: "Source search fixture".into(),
        rationale: "Preserve provenance across revisions".into(),
        scope: Scope::Author,
        changes,
    }
}

fn record(label: &str, data: ObjectData, source: Option<ObjectRef>, from: Option<i64>) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval {
            start: from.map(WorldDay),
            end_exclusive: None,
        },
        data,
        source,
    }
}

fn claim(event: ObjectRef, source: ObjectRef, statement: &str) -> Record {
    record(
        "Archivist testimony",
        ObjectData::Claim {
            subject: event,
            predicate: "testified".into(),
            statement: statement.into(),
            audience: ClaimAudience::Author,
        },
        Some(source),
        Some(10),
    )
}

fn context(world: &World, revision: AuthorRevision, day: i64) -> ViewContext {
    ViewContext {
        world: world.id,
        branch: world.main,
        day: WorldDay(day),
        revision,
        scope: Scope::Author,
    }
}

fn query(text: &str) -> QuerySpec {
    QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter: QueryFilter::ClaimFullTextPhrase(text.into()),
        limit: 100,
    }
}

fn seed(store: &mut Store, world: &World) -> Result<(ObjectRef, ObjectRef, ObjectRef, AuthorRevision), Error> {
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let account = ObjectRef::new(world.id);
    let first = store.apply(&command(
        world,
        world.revision,
        vec![
            Change::Create(event, record("Hearing", ObjectData::HistoricalEvent {
                occurrence: EventDate::Exact(WorldDay(10)),
            }, None, Some(10))),
            Change::Create(source, record("Court journal", ObjectData::Source {
                citation: "Archive folio 24".into(),
            }, None, None)),
        ],
    ))?;
    let second = store.apply(&command(
        world,
        first,
        vec![Change::Create(account, claim(event, source, "The hidden obsidian ledger was stolen."))],
    ))?;
    Ok((event, source, account, second))
}

#[test]
fn indexed_claims_respect_world_branch_day_and_recorded_revision() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("One", Scope::Author)?;
    let other = store.create_world("Two", Scope::Author)?;
    let (event, source, account, first) = seed(&mut store, &world)?;
    let (_other_event, _other_source, foreign, other_revision) = seed(&mut store, &other)?;
    assert_eq!(
        store.execute_query(&query("obsidian ledger"), context(&world, first, 10))?.rows[0].object,
        account
    );
    assert_eq!(
        store.execute_query(&query("obsidian"), context(&other, other_revision, 10))?.rows[0].object,
        foreign
    );
    assert!(store.execute_query(&query("obsidian"), context(&world, first, 9))?.rows.is_empty());
    assert!(store.execute_query(&query("obsidian"), context(&world, AuthorRevision(1), 10))?.rows.is_empty());
    assert!(store.execute_query(&query("Archive folio"), context(&world, first, 10))?.rows.is_empty(),
        "the content index must search statements, not citations or labels");
    assert!(store.execute_query(&query("obsidian OR ledger"), context(&world, first, 10))?.rows.is_empty(),
        "FTS operators must not be interpreted as queries");
    assert!(store.execute_query(&query("hidden \" OR obsidian"), context(&world, first, 10))?.rows.is_empty());
    assert!(matches!(
        store.execute_query(&query("  "), context(&world, first, 10)),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        store.execute_query(&query("obsidian"), ViewContext {
            scope: Scope::Public,
            ..context(&world, first, 10)
        }),
        Err(Error::CapabilityDenied)
    ));
    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: WorldDay(10),
        expected: first,
        id: Uuid::new_v4(),
        author: "Source search fixture".into(),
        rationale: "Fork testimony".into(),
        scope: Scope::Author,
    })?;
    let corrected = store.apply(&command(
        &world,
        fork.created_revision,
        vec![Change::Correct(account, claim(event, source, "The crimson ledger was recovered."))],
    ))?;
    assert!(store.execute_query(&query("obsidian"), context(&world, corrected, 10))?.rows.is_empty());
    assert_eq!(store.execute_query(&query("crimson ledger"), context(&world, corrected, 10))?.rows[0].object, account);
    let historical = store.execute_query(&query("obsidian"), context(&world, first, 10))?;
    assert_eq!(historical.rows[0].object, account);
    let fork_context = ViewContext {
        branch: fork.id,
        ..context(&world, corrected, 10)
    };
    assert_eq!(store.execute_query(&query("obsidian"), fork_context)?.rows[0].object, account,
        "the fork must retain the source version it inherited");
    assert!(store.execute_query(&query("crimson"), fork_context)?.rows.is_empty());
    Ok(())
}

#[test]
fn claim_fts_rebuild_backup_migration_and_saved_replay() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("main.sqlite");
    let backup = dir.path().join("backup.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Archive", Scope::Author)?;
    let (_event, _source, account, first) = seed(&mut store, &world)?;
    let saved = store.save_named_label_query(SaveNamedLabelQuery {
        world_id: world.id,
        expected: first,
        query_id: None,
        name: "Search the statements",
        spec: &query("obsidian ledger"),
        command_id: Uuid::new_v4(),
        scope: Scope::Author,
    })?;
    let current = saved.recorded_at;
    let definitions = store.saved_label_queries(world.id, current, Scope::Author)?;
    assert_eq!(definitions[0].spec, query("obsidian ledger"));
    assert_eq!(store.execute_query(&definitions[0].spec, context(&world, current, 10))?.rows[0].object, account);
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(store.execute_query(&query("obsidian"), context(&world, current, 10))?.rows[0].object, account);
    store.backup_to(&backup, Scope::Author)?;
    drop(store);
    let reopened = Store::open(&backup)?;
    assert_eq!(reopened.execute_query(&query("obsidian"), context(&world, current, 10))?.rows[0].object, account);
    reopened.verify_foreign_keys(Scope::Author)?;
    drop(reopened);

    // Reconstruct the schema-10 state with canonical claim versions intact,
    // so opening the database must backfill every historic statement.
    let conn = rusqlite::Connection::open(&path)?;
    conn.execute_batch(
        "DROP TRIGGER claim_versions_search_ai;
         DROP TRIGGER claim_versions_search_au;
         DROP TABLE claim_search_fts;
         DROP TABLE claim_search_documents;
         CREATE TABLE saved_query_versions_v10 (
             world_id BLOB NOT NULL, query_id BLOB NOT NULL,
             revision INTEGER NOT NULL CHECK(revision > 0),
             name TEXT NOT NULL, schema_version INTEGER NOT NULL CHECK(schema_version = 1),
             filter_kind TEXT NOT NULL CHECK(filter_kind IN ('contains','exact','fulltext')),
             filter_text TEXT NOT NULL, result_limit INTEGER NOT NULL CHECK(result_limit BETWEEN 1 AND 500),
             PRIMARY KEY(world_id, query_id, revision),
             FOREIGN KEY(world_id, query_id) REFERENCES saved_queries(world_id, id),
             FOREIGN KEY(world_id, revision) REFERENCES author_revisions(world_id, revision)
         ) STRICT;
         DROP TABLE saved_query_versions;
         ALTER TABLE saved_query_versions_v10 RENAME TO saved_query_versions;
         PRAGMA user_version = 10;",
    )?;
    drop(conn);
    let upgraded = Store::open(&path)?;
    assert_eq!(upgraded.execute_query(&query("obsidian"), context(&world, current, 10))?.rows[0].object, account);
    upgraded.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn claim_fts_finds_narrow_content_in_worlds_beyond_snapshot_scan_limit() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Large source corpus", Scope::Author)?;
    let (event, source, account, revision) = seed(&mut store, &world)?;
    let mut changes = Vec::new();
    for number in 0..2_010 {
        changes.push(Change::Create(
            ObjectRef::new(world.id),
            record(&format!("Observer {number}"), ObjectData::Person, None, None),
        ));
    }
    let later = store.apply(&command(&world, revision, changes))?;
    assert!(matches!(store.objects_at(context(&world, later, 10), 2_000),
        Err(Error::QueryBudgetExceeded)));
    let search = store.execute_query(&query("obsidian"), context(&world, later, 10))?;
    assert_eq!(search.rows.len(), 1);
    assert_eq!(search.rows[0].object, account);
    assert_eq!(search.rows[0].record.source, Some(source));
    assert!(matches!(search.rows[0].record.data, ObjectData::Claim { subject, .. } if subject == event));
    Ok(())
}
