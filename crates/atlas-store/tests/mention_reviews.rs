use atlas_store::{
    discovery::{MentionDisposition, ReviewMention},
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
        author: "Review fixture".into(),
        rationale: "Author mention decision".into(),
        scope: Scope::Author,
        changes,
    }
}

fn view(world: &World, revision: AuthorRevision) -> ViewContext {
    ViewContext {
        world: world.id,
        branch: world.main,
        revision,
        day: WorldDay(10),
        scope: Scope::Author,
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

fn references(
    store: &Store,
    world: &World,
    rev: AuthorRevision,
    target: ObjectRef,
) -> Result<Vec<ObjectRef>, Error> {
    let result = store.execute_query(
        &QuerySpec {
            schema_version: QUERY_SCHEMA_VERSION,
            filter: QueryFilter::Backlinks(target),
            limit: 100,
        },
        view(world, rev),
    )?;
    Ok(result.rows.into_iter().map(|row| row.object).collect())
}

#[test]
fn reviewed_link_is_versioned_author_provenance_not_a_relationship() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("history.sqlite");
    let backup = dir.path().join("backup.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Valley", Scope::Author)?;
    let one = ObjectRef::new(world.id);
    let duplicate = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(one, record("Eleanor Vale", ObjectData::Person)),
            Change::Create(duplicate, record("Eleanor Vale", ObjectData::Person)),
            Change::Create(
                event,
                record(
                    "Council",
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(10)),
                    },
                ),
            ),
            Change::Create(
                source,
                record(
                    "Diary",
                    ObjectData::Source {
                        citation: "Folio 12".into(),
                    },
                ),
            ),
        ],
    ))?;
    let second = store.apply(&command(
        &world,
        first,
        vec![Change::Create(
            claim,
            Record {
                source: Some(source),
                ..record(
                    "Witness statement",
                    ObjectData::Claim {
                        subject: event,
                        predicate: "observed".into(),
                        statement: "Eleanor Vale spoke to the Guild.".into(),
                        audience: ClaimAudience::Author,
                    },
                )
            },
        )],
    ))?;
    let candidates = store.discover_mentions(view(&world, second), 100)?;
    assert_eq!(candidates.rows.len(), 2);
    assert!(
        candidates
            .rows
            .iter()
            .all(|row| row.collisions == 2 && row.disposition.is_none())
    );
    let candidate = candidates
        .rows
        .iter()
        .find(|row| row.target == one)
        .unwrap()
        .clone();
    let review = |expected, decision| ReviewMention {
        view: view(&world, expected),
        expected,
        source: candidate.source,
        target: one,
        start_byte: candidate.start_byte,
        end_byte: candidate.end_byte,
        decision,
        command_id: Uuid::new_v4(),
    };
    let approved = store.review_mention(review(second, MentionDisposition::Approved))?;
    assert_eq!(approved, AuthorRevision(3));
    let current = store.discover_mentions(view(&world, approved), 100)?;
    assert_eq!(
        current
            .rows
            .iter()
            .find(|row| row.target == one)
            .unwrap()
            .disposition,
        Some(MentionDisposition::Approved)
    );
    assert_eq!(
        current
            .rows
            .iter()
            .find(|row| row.target == duplicate)
            .unwrap()
            .disposition,
        None
    );
    assert!(references(&store, &world, second, one)?.is_empty());
    assert_eq!(references(&store, &world, approved, one)?, vec![claim]);
    assert!(references(&store, &world, approved, duplicate)?.is_empty());
    assert!(matches!(
        store.review_mention(ReviewMention {
            expected: approved,
            ..review(approved, MentionDisposition::Approved)
        }),
        Ok(AuthorRevision(4))
    ));
    let latest = AuthorRevision(4);
    let rejected = store.review_mention(review(latest, MentionDisposition::Dismissed))?;
    assert!(references(&store, &world, rejected, one)?.is_empty());
    assert_eq!(references(&store, &world, approved, one)?, vec![claim]);
    assert_eq!(
        store
            .discover_mentions(view(&world, rejected), 100)?
            .rows
            .iter()
            .find(|row| row.target == one)
            .unwrap()
            .disposition,
        Some(MentionDisposition::Dismissed)
    );

    let reapproved = store.review_mention(review(rejected, MentionDisposition::Approved))?;
    let renamed = store.apply(&command(
        &world,
        reapproved,
        vec![Change::Correct(
            one,
            record("Eleanor Thorne", ObjectData::Person),
        )],
    ))?;
    assert_eq!(
        references(&store, &world, renamed, one)?,
        vec![claim],
        "explicit stable-ID link survives a person rename"
    );
    let corrected = store.apply(&command(
        &world,
        renamed,
        vec![Change::Correct(
            claim,
            Record {
                source: Some(source),
                ..record(
                    "Witness statement",
                    ObjectData::Claim {
                        subject: event,
                        predicate: "observed".into(),
                        statement: "Nobody recognizable spoke to the Guild.".into(),
                        audience: ClaimAudience::Author,
                    },
                )
            },
        )],
    ))?;
    assert!(
        references(&store, &world, corrected, one)?.is_empty(),
        "old source-version approval must not follow a corrected statement"
    );
    assert!(
        store
            .discover_mentions(view(&world, corrected), 100)?
            .rows
            .is_empty()
    );
    store.backup_to(&backup, Scope::Author)?;
    drop(store);
    let reopened = Store::open(&backup)?;
    assert!(references(&reopened, &world, corrected, one)?.is_empty());
    assert_eq!(references(&reopened, &world, renamed, one)?, vec![claim]);
    reopened.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn invalid_reviews_and_old_schema_upgrade_are_safe() -> Result<(), Error> {
    let dir = tempdir()?;
    let path = dir.path().join("upgrade.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("One", Scope::Author)?;
    let foreign = store.create_world("Other", Scope::Author)?;
    let one = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(one, record("Morgan", ObjectData::Person)),
            Change::Create(
                event,
                record(
                    "Council",
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(10)),
                    },
                ),
            ),
            Change::Create(
                source,
                record(
                    "Diary",
                    ObjectData::Source {
                        citation: "Folio".into(),
                    },
                ),
            ),
        ],
    ))?;
    let second = store.apply(&command(
        &world,
        first,
        vec![Change::Create(
            claim,
            Record {
                source: Some(source),
                ..record(
                    "Witness",
                    ObjectData::Claim {
                        subject: event,
                        predicate: "observed".into(),
                        statement: "Morgan was seen.".into(),
                        audience: ClaimAudience::Author,
                    },
                )
            },
        )],
    ))?;
    let candidate = store
        .discover_mentions(view(&world, second), 100)?
        .rows
        .remove(0);
    let valid = ReviewMention {
        view: view(&world, second),
        expected: second,
        source: candidate.source,
        target: one,
        start_byte: candidate.start_byte,
        end_byte: candidate.end_byte,
        decision: MentionDisposition::Approved,
        command_id: Uuid::new_v4(),
    };
    assert!(matches!(
        store.review_mention(ReviewMention {
            start_byte: 1,
            ..valid
        }),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        store.review_mention(ReviewMention {
            target: ObjectRef::new(foreign.id),
            ..valid
        }),
        Err(Error::WrongWorld)
    ));
    assert!(matches!(
        store.review_mention(ReviewMention {
            view: ViewContext {
                scope: Scope::Public,
                ..valid.view
            },
            ..valid
        }),
        Err(Error::CapabilityDenied)
    ));
    assert_eq!(store.world(world.id, Scope::Author)?.revision, second);

    drop(store);
    // Simulate a complete schema-nine archive prior to mention review support.
    let conn = rusqlite::Connection::open(&path)?;
    conn.execute_batch(
        "DROP TRIGGER claim_versions_search_ai;
         DROP TRIGGER claim_versions_search_au;
         DROP TABLE claim_search_fts;
         DROP TABLE claim_search_documents;
         DROP TABLE mention_reviews;
         PRAGMA user_version = 9;",
    )?;
    drop(conn);
    let mut upgraded = Store::open(&path)?;
    assert_eq!(upgraded.review_mention(valid)?, AuthorRevision(3));
    upgraded.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn review_decisions_do_not_escape_into_alternate_history_branches() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Branch isolation", Scope::Author)?;
    let person = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(person, record("Morgan Vale", ObjectData::Person)),
            Change::Create(
                event,
                record(
                    "Meeting",
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(10)),
                    },
                ),
            ),
            Change::Create(
                source,
                record(
                    "Ledger",
                    ObjectData::Source {
                        citation: "Folio 4".into(),
                    },
                ),
            ),
        ],
    ))?;
    let second = store.apply(&command(
        &world,
        first,
        vec![Change::Create(
            claim,
            Record {
                source: Some(source),
                ..record(
                    "Statement",
                    ObjectData::Claim {
                        subject: event,
                        predicate: "presence".into(),
                        statement: "Morgan Vale was present.".into(),
                        audience: ClaimAudience::Author,
                    },
                )
            },
        )],
    ))?;
    let candidate = store
        .discover_mentions(view(&world, second), 100)?
        .rows
        .remove(0);
    let approved = store.review_mention(ReviewMention {
        view: view(&world, second),
        expected: second,
        source: candidate.source,
        target: person,
        start_byte: candidate.start_byte,
        end_byte: candidate.end_byte,
        decision: MentionDisposition::Approved,
        command_id: Uuid::new_v4(),
    })?;
    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: WorldDay(10),
        expected: approved,
        id: Uuid::new_v4(),
        author: "Reviewer".into(),
        rationale: "Independent branch history".into(),
        scope: Scope::Author,
    })?;
    let child = ViewContext {
        branch: fork.id,
        revision: fork.created_revision,
        ..view(&world, approved)
    };
    let child_candidates = store.discover_mentions(child, 100)?;
    assert_eq!(child_candidates.rows.len(), 1);
    assert_eq!(child_candidates.rows[0].disposition, None);
    assert_eq!(
        store
            .execute_query(
                &QuerySpec {
                    schema_version: QUERY_SCHEMA_VERSION,
                    filter: QueryFilter::Backlinks(person),
                    limit: 100,
                },
                child,
            )?
            .rows
            .len(),
        0
    );
    assert_eq!(
        references(&store, &world, fork.created_revision, person)?,
        vec![claim]
    );
    assert!(matches!(
        store.review_mention(ReviewMention {
            view: child,
            expected: fork.created_revision,
            source: candidate.source,
            target: person,
            start_byte: candidate.start_byte,
            end_byte: candidate.end_byte,
            decision: MentionDisposition::Dismissed,
            command_id: Uuid::new_v4(),
        }),
        Err(Error::CapabilityDenied)
    ));
    Ok(())
}
