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

#[test]
fn focused_discovery_and_source_delta_ignore_unrelated_records() -> Result<(), Error> {
    use atlas_store::discovery::{MentionRefresh, SavePersonAlias};
    use atlas_store::query::QueryControl;
    use std::time::Duration;

    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Large archive", Scope::Author)?;
    let person = ObjectRef::new(world.id);
    let other = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let citation = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let unrelated = ObjectRef::new(world.id);
    let mut changes = vec![
        Change::Create(person, record("Eleanor Vale", ObjectData::Person)),
        Change::Create(other, record("Eleanor Vale", ObjectData::Person)),
        Change::Create(
            event,
            record(
                "Meeting",
                ObjectData::HistoricalEvent {
                    occurrence: EventDate::Exact(WorldDay(0)),
                },
            ),
        ),
        Change::Create(
            citation,
            record(
                "Diary",
                ObjectData::Source {
                    citation: "Diary 1".into(),
                },
            ),
        ),
    ];
    for number in 0..2_100 {
        changes.push(Change::Create(
            ObjectRef::new(world.id),
            record(
                &format!("Unrelated archive {number}"),
                ObjectData::Source {
                    citation: format!("Folio {number}"),
                },
            ),
        ));
    }
    let initial = store.apply(&command(&world, world.revision, changes))?;
    let claim_record = |label: &str, statement: &str| Record {
        source: Some(citation),
        ..record(
            label,
            ObjectData::Claim {
                subject: event,
                predicate: "reported".into(),
                statement: statement.into(),
                audience: ClaimAudience::Author,
            },
        )
    };
    let written = store.apply(&command(
        &world,
        initial,
        vec![
            Change::Create(claim, claim_record("Focused", "Eleanor Vale arrived.")),
            Change::Create(
                unrelated,
                claim_record("Unrelated", "Eleanor Vale departed."),
            ),
        ],
    ))?;
    let before = view(&world, written, 0);
    assert!(matches!(
        store.objects_at(before, 2_000),
        Err(Error::QueryBudgetExceeded)
    ));
    let focused = store.discover_mentions_for_source(before, claim, 100)?;
    assert_eq!(focused.rows.len(), 2);
    assert!(
        focused
            .rows
            .iter()
            .all(|row| row.source.object == claim && row.collisions == 2)
    );
    assert_eq!(store.discover_mentions(before, 100)?.rows.len(), 4);
    let corrected = store.apply(&command(
        &world,
        written,
        vec![Change::Correct(
            claim,
            claim_record("Focused", "No names remain."),
        )],
    ))?;
    let after = view(&world, corrected, 0);
    assert_eq!(
        store.mention_refresh_plan(before, after)?,
        MentionRefresh::Sources(vec![claim])
    );
    assert!(
        store
            .discover_mentions_for_source(after, claim, 100)?
            .rows
            .is_empty()
    );
    assert_eq!(
        store.discover_mentions_for_source(before, claim, 100)?.rows,
        focused.rows
    );
    let alias = store.save_person_alias(SavePersonAlias {
        world: world.id,
        person,
        day: WorldDay(0),
        expected: corrected,
        alias_id: None,
        label: "Archivist",
        starts_at: None,
        ends_before: None,
        active: true,
        command_id: Uuid::new_v4(),
        scope: Scope::Author,
    })?;
    let renamed = view(&world, alias.recorded_at, 0);
    assert_eq!(
        store.mention_refresh_plan(after, renamed)?,
        MentionRefresh::Full
    );
    assert_eq!(
        store.mention_refresh_plan(renamed, view(&world, alias.recorded_at, 1))?,
        MentionRefresh::Full
    );
    assert_eq!(
        store.mention_refresh_plan(renamed, renamed)?,
        MentionRefresh::Sources(vec![])
    );
    let control = QueryControl::new(Duration::ZERO)?;
    assert!(matches!(
        store.discover_mentions_controlled(renamed, Some(claim), 100, &control),
        Err(Error::QueryDeadlineExceeded)
    ));
    assert!(matches!(
        store.discover_mentions_for_source(renamed, person, 100),
        Err(Error::WrongKind)
    ));
    assert!(matches!(
        store.discover_mentions_for_source(
            ViewContext {
                scope: Scope::Public,
                ..renamed
            },
            claim,
            100
        ),
        Err(Error::CapabilityDenied)
    ));
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(
        store.discover_mentions_for_source(before, claim, 100)?.rows,
        focused.rows
    );
    Ok(())
}

#[test]
fn indexed_names_ignore_large_unmatched_dictionary_and_preserve_exact_spans() -> Result<(), Error> {
    use atlas_store::discovery::SavePersonAlias;
    use tempfile::tempdir;

    let dir = tempdir()?;
    let path = dir.path().join("mention-index.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Large dictionary", Scope::Author)?;
    let ann = ObjectRef::new(world.id);
    let unicode = ObjectRef::new(world.id);
    let short = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let mut changes = vec![
        Change::Create(ann, record("Ann", ObjectData::Person)),
        Change::Create(unicode, record("Élodie", ObjectData::Person)),
        Change::Create(short, record("Q", ObjectData::Person)),
        Change::Create(
            event,
            record(
                "Meeting",
                ObjectData::HistoricalEvent {
                    occurrence: EventDate::Exact(WorldDay(0)),
                },
            ),
        ),
        Change::Create(
            source,
            record(
                "Diary",
                ObjectData::Source {
                    citation: "Folio 1".into(),
                },
            ),
        ),
    ];
    for number in 0..10_001 {
        changes.push(Change::Create(
            ObjectRef::new(world.id),
            record(&format!("Zzz resident {number}"), ObjectData::Person),
        ));
    }
    let initial = store.apply(&command(&world, world.revision, changes))?;
    let alias = store.save_person_alias(SavePersonAlias {
        world: world.id,
        person: unicode,
        day: WorldDay(0),
        expected: initial,
        alias_id: None,
        label: "Ann",
        starts_at: Some(WorldDay(0)),
        ends_before: Some(WorldDay(10)),
        active: true,
        command_id: Uuid::new_v4(),
        scope: Scope::Author,
    })?;
    let text = "ANN met Joanne. Élodie saw éLODIE and q; Annette.";
    let written = store.apply(&command(
        &world,
        alias.recorded_at,
        vec![Change::Create(
            claim,
            Record {
                source: Some(source),
                ..record(
                    "Entry",
                    ObjectData::Claim {
                        subject: event,
                        predicate: "reported".into(),
                        statement: text.into(),
                        audience: ClaimAudience::Author,
                    },
                )
            },
        )],
    ))?;
    let context = view(&world, written, 0);
    let rows = store
        .discover_mentions_for_source(context, claim, 100)?
        .rows;
    assert_eq!(rows.len(), 4);
    assert_eq!(rows.iter().filter(|row| row.collisions == 2).count(), 2);
    assert!(
        rows.iter()
            .any(|row| row.target == ann && &text[row.start_byte..row.end_byte] == "ANN")
    );
    assert!(
        rows.iter()
            .any(|row| row.target == unicode && &text[row.start_byte..row.end_byte] == "Élodie")
    );
    assert!(
        rows.iter()
            .any(|row| row.target == short && &text[row.start_byte..row.end_byte] == "q")
    );
    assert_eq!(
        store
            .discover_mentions(view(&world, written, 10), 100)?
            .rows
            .len(),
        3
    );
    let corrected = store.apply(&command(
        &world,
        written,
        vec![Change::Correct(ann, record("Anna", ObjectData::Person))],
    ))?;
    let current = store
        .discover_mentions(view(&world, corrected, 0), 100)?
        .rows;
    assert_eq!(current.len(), 3);
    assert!(!current.iter().any(|row| row.target == ann));
    assert_eq!(
        store
            .discover_mentions_for_source(context, claim, 100)?
            .rows,
        rows
    );
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(
        store
            .discover_mentions_for_source(context, claim, 100)?
            .rows,
        rows
    );
    drop(store);
    let conn = rusqlite::Connection::open(&path)?;
    conn.execute_batch("DROP INDEX discovery_person_prefix_idx; DROP INDEX discovery_alias_prefix_idx; PRAGMA user_version = 20;")?;
    drop(conn);
    let store = Store::open(&path)?;
    assert_eq!(
        store
            .discover_mentions_for_source(context, claim, 100)?
            .rows,
        rows
    );
    store.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn controlled_previews_refresh_only_changed_sources_and_match_fresh_reads() -> Result<(), Error> {
    use atlas_store::discovery::{MentionDisposition, ReviewMention};
    use atlas_store::query::QueryControl;
    use std::time::Duration;

    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Refresh archive", Scope::Author)?;
    let person = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let first = ObjectRef::new(world.id);
    let second = ObjectRef::new(world.id);
    let initial = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(person, record("Ann", ObjectData::Person)),
            Change::Create(
                event,
                record(
                    "Meeting",
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(0)),
                    },
                ),
            ),
            Change::Create(
                source,
                record(
                    "Diary",
                    ObjectData::Source {
                        citation: "Folio 1".into(),
                    },
                ),
            ),
        ],
    ))?;
    let claim = |label: &str, statement: &str| Record {
        source: Some(source),
        ..record(
            label,
            ObjectData::Claim {
                subject: event,
                predicate: "reported".into(),
                statement: statement.into(),
                audience: ClaimAudience::Author,
            },
        )
    };
    let written = store.apply(&command(
        &world,
        initial,
        vec![
            Change::Create(first, claim("First", "Ann arrived.")),
            Change::Create(second, claim("Second", "Ann departed.")),
        ],
    ))?;
    let read = |store: &Store, revision, focus, limit| {
        store.discover_mentions_controlled(
            view(&world, revision, 0),
            focus,
            limit,
            &QueryControl::new(Duration::from_secs(10))?,
        )
    };
    let before = read(&store, written, None, 100)?;
    assert_eq!(before.rows.len(), 2);
    let chosen = before
        .rows
        .iter()
        .find(|row| row.source.object == first)
        .unwrap();
    let approved = store.review_mention(ReviewMention {
        view: view(&world, written, 0),
        expected: written,
        source: chosen.source,
        target: chosen.target,
        start_byte: chosen.start_byte,
        end_byte: chosen.end_byte,
        decision: MentionDisposition::Approved,
        command_id: Uuid::new_v4(),
    })?;
    let updated = read(&store, approved, None, 100)?;
    assert_eq!(
        updated.rows,
        store
            .discover_mentions(view(&world, approved, 0), 100)?
            .rows
    );
    assert_eq!(
        updated
            .rows
            .iter()
            .find(|row| row.source.object == first)
            .unwrap()
            .disposition,
        Some(MentionDisposition::Approved)
    );
    let corrected = store.apply(&command(
        &world,
        approved,
        vec![Change::Correct(first, claim("First", "Nobody arrived."))],
    ))?;
    let refreshed = read(&store, corrected, None, 100)?;
    assert_eq!(refreshed.rows.len(), 1);
    assert_eq!(refreshed.rows[0].source.object, second);
    assert_eq!(
        refreshed.rows,
        store
            .discover_mentions(view(&world, corrected, 0), 100)?
            .rows
    );
    assert_eq!(read(&store, written, None, 100)?.rows, before.rows);

    // Incomplete results cannot serve as complete cached corpora.
    assert!(read(&store, written, None, 1)?.truncated);
    let focused = read(&store, corrected, Some(first), 100)?;
    assert!(focused.rows.is_empty());
    let unrelated = store.apply(&command(
        &world,
        corrected,
        vec![Change::Correct(second, claim("Second", "Ann returned."))],
    ))?;
    assert!(read(&store, unrelated, Some(first), 100)?.rows.is_empty());
    let rename = store.apply(&command(
        &world,
        unrelated,
        vec![Change::Correct(person, record("Anna", ObjectData::Person))],
    ))?;
    assert!(read(&store, rename, None, 100)?.rows.is_empty());
    assert!(matches!(
        store.discover_mentions_controlled(
            ViewContext {
                scope: Scope::Public,
                ..view(&world, rename, 0)
            },
            None,
            100,
            &QueryControl::new(Duration::from_secs(10))?,
        ),
        Err(Error::CapabilityDenied)
    ));
    let cancelled = QueryControl::new(Duration::from_secs(10))?;
    cancelled.cancel();
    assert!(matches!(
        store.discover_mentions_controlled(view(&world, rename, 0), None, 100, &cancelled),
        Err(Error::QueryCancelled)
    ));
    assert!(read(&store, rename, None, 100)?.rows.is_empty());
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(read(&store, written, None, 100)?.rows, before.rows);
    Ok(())
}
