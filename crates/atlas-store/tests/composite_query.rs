//! Phase 3 bounded, typed predicates over an authorized historical snapshot.
use atlas_store::{
    query::{ObjectKindFilter, QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec},
    *,
};
use uuid::Uuid;

fn command(world: &World, expected: AuthorRevision, changes: Vec<Change>) -> Command {
    Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Predicate tests".into(),
        rationale: "Check indexed-free compound historical filters".into(),
        scope: Scope::Author,
        changes,
    }
}

fn record(label: &str, start: Option<i64>, data: ObjectData, source: Option<ObjectRef>) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval {
            start: start.map(WorldDay),
            end_exclusive: None,
        },
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

fn query(filter: QueryFilter, limit: usize) -> QuerySpec {
    QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter,
        limit,
    }
}

#[test]
fn compound_predicates_obey_kind_source_boolean_and_historical_scope() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Historical archive", Scope::Author)?;
    let other = store.create_world("Other", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let diary = ObjectRef::new(world.id);
    let journal = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, record("Alice", None, ObjectData::Person, None)),
            Change::Create(
                source,
                record(
                    "Journal volume 4",
                    None,
                    ObjectData::Source {
                        citation: "J-4".into(),
                    },
                    None,
                ),
            ),
            Change::Create(
                event,
                record(
                    "Trial",
                    Some(10),
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(10)),
                    },
                    None,
                ),
            ),
        ],
    ))?;
    let claim = |label: &str, source| {
        record(
            label,
            Some(10),
            ObjectData::Claim {
                subject: event,
                predicate: "testified".into(),
                statement: "Alice was seen near the library".into(),
                audience: ClaimAudience::Author,
            },
            source,
        )
    };
    let second = store.apply(&command(
        &world,
        first,
        vec![
            Change::Create(diary, claim("Diary testimony", Some(source))),
            Change::Create(journal, claim("Independent testimony", Some(source))),
        ],
    ))?;
    let names = QueryFilter::Or(vec![
        QueryFilter::LabelContains("Diary".into()),
        QueryFilter::ExactLabel("Alice".into()),
    ]);
    let filter = QueryFilter::And(vec![
        QueryFilter::Or(vec![names, QueryFilter::Kind(ObjectKindFilter::Claim)]),
        QueryFilter::Not(Box::new(QueryFilter::ExactLabel(
            "Independent testimony".into(),
        ))),
    ]);
    let full = store.execute_query(&query(filter, 100), view(&world, second, 10))?;
    let returned: Vec<_> = full.rows.iter().map(|row| row.object).collect();
    assert_eq!(returned.len(), 2);
    assert!(returned.contains(&alice));
    assert!(returned.contains(&diary));
    let sourced = QueryFilter::And(vec![
        QueryFilter::Kind(ObjectKindFilter::Claim),
        QueryFilter::HasSource(source),
        QueryFilter::LabelContains("testimony".into()),
    ]);
    assert_eq!(
        store
            .execute_query(&query(sourced.clone(), 100), view(&world, second, 10))?
            .rows
            .len(),
        2
    );
    assert!(
        store
            .execute_query(&query(sourced.clone(), 1), view(&world, second, 10))?
            .truncated
    );
    assert!(
        store
            .execute_query(&query(sourced.clone(), 100), view(&world, first, 10))?
            .rows
            .is_empty()
    );
    assert!(
        store
            .execute_query(&query(sourced.clone(), 100), view(&world, second, 9))?
            .rows
            .is_empty()
    );

    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: WorldDay(10),
        expected: second,
        id: Uuid::new_v4(),
        author: "Predicate tests".into(),
        rationale: "Preserve historical claim".into(),
        scope: Scope::Author,
    })?;
    let before_creation = ViewContext {
        branch: fork.id,
        ..view(&world, second, 10)
    };
    assert!(matches!(
        store.execute_query(
            &query(QueryFilter::Kind(ObjectKindFilter::Person), 100),
            before_creation
        ),
        Err(Error::NotFound)
    ));
    let corrected = store.apply(&command(
        &world,
        fork.created_revision,
        vec![Change::Correct(
            diary,
            claim("Revised account", Some(source)),
        )],
    ))?;
    let diary_name = query(
        QueryFilter::And(vec![
            QueryFilter::Kind(ObjectKindFilter::Claim),
            QueryFilter::LabelContains("Diary".into()),
        ]),
        100,
    );
    assert!(
        store
            .execute_query(&diary_name, view(&world, corrected, 10))?
            .rows
            .is_empty()
    );
    assert_eq!(
        store
            .execute_query(&diary_name, view(&world, second, 10))?
            .rows[0]
            .object,
        diary
    );
    assert_eq!(
        store
            .execute_query(
                &diary_name,
                ViewContext {
                    branch: fork.id,
                    ..view(&world, corrected, 10)
                }
            )?
            .rows[0]
            .object,
        diary
    );
    assert!(matches!(
        store.execute_query(
            &query(sourced, 100),
            ViewContext {
                scope: Scope::Public,
                ..view(&world, corrected, 10)
            }
        ),
        Err(Error::CapabilityDenied)
    ));
    assert!(
        store
            .execute_query(
                &query(QueryFilter::Kind(ObjectKindFilter::Person), 100),
                view(&other, other.revision, 10)
            )?
            .rows
            .is_empty()
    );
    Ok(())
}

#[test]
fn compound_validation_is_upfront_bounded_and_not_bypassable_by_boolean_short_circuit()
-> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("One", Scope::Author)?;
    let other = store.create_world("Two", Scope::Author)?;
    let foreign = ObjectRef::new(other.id);
    let current = view(&world, world.revision, 0);
    let invalids = [
        QueryFilter::And(Vec::new()),
        QueryFilter::Or(vec![QueryFilter::Kind(ObjectKindFilter::Person)]),
        QueryFilter::Not(Box::new(QueryFilter::FullTextPhrase(" ".into()))),
        QueryFilter::Or(vec![
            QueryFilter::Kind(ObjectKindFilter::Person),
            QueryFilter::Backlinks(foreign),
        ]),
        QueryFilter::And(vec![
            QueryFilter::Kind(ObjectKindFilter::Claim),
            QueryFilter::HasSource(foreign),
        ]),
        QueryFilter::LabelContains(" ".into()),
    ];
    for filter in invalids {
        assert!(store.execute_query(&query(filter, 100), current).is_err());
    }
    let mut deep = QueryFilter::Kind(ObjectKindFilter::Person);
    for _ in 0..6 {
        deep = QueryFilter::Not(Box::new(deep));
    }
    assert!(matches!(
        store.execute_query(&query(deep, 100), current),
        Err(Error::QueryBudgetExceeded)
    ));
    let wide = QueryFilter::And(
        (0..33)
            .map(|_| QueryFilter::Kind(ObjectKindFilter::Person))
            .collect(),
    );
    assert!(matches!(
        store.execute_query(&query(wide, 100), current),
        Err(Error::QueryBudgetExceeded)
    ));
    assert!(matches!(
        store.execute_query(&query(QueryFilter::HasSource(foreign), 100), current),
        Err(Error::WrongWorld)
    ));
    // Invalid branch is refused even if no currently visible objects match.
    let unknown = ViewContext {
        branch: BranchId(Uuid::new_v4()),
        ..current
    };
    assert!(matches!(
        store.objects_at(unknown, 100),
        Err(Error::NotFound)
    ));
    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: WorldDay(0),
        expected: world.revision,
        id: Uuid::new_v4(),
        author: "Predicate tests".into(),
        rationale: "Validate empty snapshot coordinates".into(),
        scope: Scope::Author,
    })?;
    let before_fork = ViewContext {
        branch: fork.id,
        ..current
    };
    assert!(matches!(
        store.objects_at(before_fork, 100),
        Err(Error::NotFound)
    ));
    assert!(
        store
            .objects_at(
                ViewContext {
                    revision: fork.created_revision,
                    ..before_fork
                },
                100
            )?
            .is_empty()
    );
    assert!(
        store
            .execute_query(
                &query(QueryFilter::Kind(ObjectKindFilter::Person), 100),
                unknown
            )
            .is_err()
    );
    Ok(())
}

#[test]
fn indexed_compounds_preserve_exact_results_beyond_snapshot_budget() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Indexed archive", Scope::Author)?;
    let person = ObjectRef::new(world.id);
    let source = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let claim = ObjectRef::new(world.id);
    let excluded = ObjectRef::new(world.id);
    let claim_record = |label: &str, statement: &str| {
        record(
            label,
            Some(10),
            ObjectData::Claim {
                subject: event,
                predicate: "testified".into(),
                statement: statement.into(),
                audience: ClaimAudience::Author,
            },
            Some(source),
        )
    };
    let changes = vec![
        Change::Create(
            person,
            record("Obsidian navigator", None, ObjectData::Person, None),
        ),
        Change::Create(
            source,
            record(
                "Court journal",
                None,
                ObjectData::Source {
                    citation: "Folio 23".into(),
                },
                None,
            ),
        ),
        Change::Create(
            event,
            record(
                "Trial",
                Some(10),
                ObjectData::HistoricalEvent {
                    occurrence: EventDate::Exact(WorldDay(10)),
                },
                None,
            ),
        ),
        Change::Create(
            claim,
            claim_record("Witness notes", "The sealed ledger was found."),
        ),
        Change::Create(
            excluded,
            claim_record("A excluded witness notes", "The sealed ledger was found."),
        ),
    ];
    let first = store.apply(&command(&world, world.revision, changes))?;
    let conjunction = query(
        QueryFilter::And(vec![
            QueryFilter::ClaimFullTextPhrase("sealed ledger".into()),
            QueryFilter::FullTextPhrase("witness notes".into()),
            QueryFilter::Kind(ObjectKindFilter::Claim),
            QueryFilter::HasSource(source),
            QueryFilter::Not(Box::new(QueryFilter::LabelContains("excluded".into()))),
        ]),
        100,
    );
    let disjunction = query(
        QueryFilter::Or(vec![
            conjunction.filter.clone(),
            QueryFilter::FullTextPhrase("obsidian navigator".into()),
        ]),
        100,
    );
    let negated = query(
        QueryFilter::Not(Box::new(QueryFilter::FullTextPhrase(
            "obsidian navigator".into(),
        ))),
        100,
    );
    assert_eq!(
        store
            .execute_query(&negated, view(&world, first, 20))?
            .rows
            .len(),
        4
    );
    let mut changes = Vec::new();
    for number in 0..2_010 {
        changes.push(Change::Create(
            ObjectRef::new(world.id),
            record(
                &format!("Ordinary inhabitant {number}"),
                None,
                ObjectData::Person,
                None,
            ),
        ));
    }
    let large = store.apply(&command(&world, first, changes))?;
    assert!(matches!(
        store.objects_at(view(&world, large, 20), 2_000),
        Err(Error::QueryBudgetExceeded)
    ));
    let result = store.execute_query(
        &QuerySpec {
            limit: 1,
            ..conjunction.clone()
        },
        view(&world, large, 20),
    )?;
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].object, claim);
    assert!(!result.truncated);
    let union = store.execute_query(&disjunction, view(&world, large, 20))?;
    assert_eq!(union.rows.len(), 2);
    assert!(union.rows.iter().any(|row| row.object == person));
    assert!(union.rows.iter().any(|row| row.object == claim));
    assert!(
        store
            .execute_query(&conjunction, view(&world, large, 9))?
            .rows
            .is_empty()
    );
    assert!(matches!(
        store.execute_query(&negated, view(&world, large, 20)),
        Err(Error::QueryBudgetExceeded)
    ));
    let corrected = store.apply(&command(
        &world,
        large,
        vec![Change::Correct(
            claim,
            claim_record("Witness notes", "The document was destroyed."),
        )],
    ))?;
    assert!(
        store
            .execute_query(&conjunction, view(&world, corrected, 20))?
            .rows
            .is_empty()
    );
    assert_eq!(
        store
            .execute_query(&conjunction, view(&world, large, 20))?
            .rows[0]
            .object,
        claim
    );
    assert!(matches!(
        store.execute_query(
            &conjunction,
            ViewContext {
                scope: Scope::Public,
                ..view(&world, corrected, 20)
            }
        ),
        Err(Error::CapabilityDenied)
    ));
    Ok(())
}
