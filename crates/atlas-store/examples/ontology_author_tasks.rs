//! AE-01 baseline and controlled comparison using actual Rust persistence.
use atlas_store::{
    query::{ObjectKindFilter, QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec},
    *,
};
use uuid::Uuid;

fn baseline() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Estuary", Scope::Author)?;
    let source = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let create = |object, label: &str, data, source, valid| {
        Change::Create(
            object,
            Record {
                label: label.into(),
                data,
                source,
                valid,
            },
        )
    };
    let mut changes = vec![
        create(
            source,
            "Field notebook",
            ObjectData::Source {
                citation: "Survey folio 3".into(),
            },
            None,
            ValidInterval::default(),
        ),
        create(
            event,
            "Estuary survey",
            ObjectData::HistoricalEvent {
                occurrence: EventDate::Exact(WorldDay(10)),
            },
            None,
            ValidInterval::default(),
        ),
    ];
    let tasks = [
        (
            "language",
            "The survey recorded Estuary speech in port records.",
            "Estuary speech",
        ),
        (
            "area",
            "The bronze working site measured twelve hectares.",
            "twelve hectares",
        ),
        (
            "medium",
            "The aetheric drive used amber as its medium.",
            "amber",
        ),
    ];
    let mut expected = Vec::new();
    for (predicate, statement, phrase) in tasks {
        let claim = ObjectRef::new(world.id);
        changes.push(create(
            claim,
            predicate,
            ObjectData::Claim {
                subject: event,
                predicate: predicate.into(),
                statement: statement.into(),
                audience: ClaimAudience::Author,
            },
            Some(source),
            ValidInterval {
                start: Some(WorldDay(10)),
                end_exclusive: None,
            },
        ));
        expected.push((claim, phrase));
    }
    let revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: world.revision,
        id: Uuid::new_v4(),
        author: "Field researcher".into(),
        rationale: "Record survey testimony".into(),
        scope: Scope::Author,
        changes,
    })?;
    for (claim, phrase) in expected {
        let query = QuerySpec {
            schema_version: QUERY_SCHEMA_VERSION,
            limit: 10,
            filter: QueryFilter::And(vec![
                QueryFilter::Kind(ObjectKindFilter::Claim),
                QueryFilter::HasSource(source),
                QueryFilter::ClaimFullTextPhrase(phrase.into()),
            ]),
        };
        for day in [9, 10] {
            let view = ViewContext {
                world: world.id,
                branch: world.main,
                day: WorldDay(day),
                revision,
                scope: Scope::Author,
            };
            let result = store.execute_query(&query, view)?;
            let actual: Vec<_> = result.rows.iter().map(|row| row.object).collect();
            assert_eq!(actual, if day == 10 { vec![claim] } else { vec![] });
        }
        println!("{phrase}: original sourced claim found on day 10 only");
    }
    Ok(())
}

fn commit(store: &mut Store, view: &mut ViewContext, changes: Vec<Change>) -> Result<(), Error> {
    view.revision = store.apply(&Command {
        world: view.world,
        branch: view.branch,
        expected: view.revision,
        id: Uuid::new_v4(),
        author: "Comparison".into(),
        rationale: "Record survey observations".into(),
        scope: Scope::Author,
        changes,
    })?;
    Ok(())
}

fn rows(store: &Store, view: ViewContext, filter: QueryFilter) -> Result<Vec<ObjectRef>, Error> {
    Ok(store
        .execute_query(
            &QuerySpec {
                schema_version: QUERY_SCHEMA_VERSION,
                filter,
                limit: 20,
            },
            view,
        )?
        .rows
        .into_iter()
        .map(|version| version.object)
        .collect())
}

fn compare(variant: char) -> Result<(), Error> {
    use atlas_store::{
        network::{NetworkInput, NetworkKind},
        semantic::{
            AreaReading, AreaTest, AreaUnit, ClassificationInput, ConceptProfile, DefinitionPin,
            FactFamily, FactRef, SiteAreaInput,
        },
    };
    use std::time::Instant;

    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Estuary comparison", Scope::Author)?;
    let mut view = ViewContext {
        world: world.id,
        branch: world.main,
        revision: world.revision,
        day: WorldDay(10),
        scope: Scope::Author,
    };
    let [
        source,
        event,
        culture,
        language,
        material,
        drive_a,
        drive_b,
        place,
        vocabulary,
        drive_tag,
        area_tag,
    ] = std::array::from_fn(|_| ObjectRef::new(world.id));
    let create = |object, label: &str, data| {
        Change::Create(
            object,
            Record {
                label: label.into(),
                data,
                source: None,
                valid: ValidInterval::default(),
            },
        )
    };
    let tag = |description: &str| ObjectData::Definition {
        definition: Definition::Tag,
        description: description.into(),
    };
    commit(
        &mut store,
        &mut view,
        vec![
            create(
                source,
                "Field notebook",
                ObjectData::Source {
                    citation: "Survey folio 3".into(),
                },
            ),
            create(
                event,
                "Estuary survey",
                ObjectData::HistoricalEvent {
                    occurrence: EventDate::Exact(WorldDay(10)),
                },
            ),
            create(culture, "Estuary culture", ObjectData::Concept),
            create(language, "Estuary speech", ObjectData::Concept),
            create(material, "Amber", ObjectData::Concept),
            create(drive_a, "Drive One", ObjectData::Concept),
            create(drive_b, "Drive Two", ObjectData::Concept),
            create(place, "Workshop", ObjectData::Place),
            create(vocabulary, "Vocabulary", tag("Curated profiles")),
            create(drive_tag, "Aetheric drive", tag("Local technology category")),
            create(area_tag, "Recorded area", tag("Survey area")),
        ],
    )?;
    // Identical seed for each alternative; report only task-specific commands.
    let claim = |label: &str, statement: &str| Record {
        label: label.into(),
        data: ObjectData::Claim {
            subject: event,
            predicate: label.into(),
            statement: statement.into(),
            audience: ClaimAudience::Author,
        },
        source: Some(source),
        valid: ValidInterval {
            start: Some(WorldDay(10)),
            end_exclusive: None,
        },
    };
    let find_claim = |store: &Store, at, phrase: &str| {
        rows(
            store,
            at,
            QueryFilter::And(vec![
                QueryFilter::Kind(ObjectKindFilter::Claim),
                QueryFilter::HasSource(source),
                QueryFilter::ClaimFullTextPhrase(phrase.into()),
            ]),
        )
    };
    let classify = |store: &mut Store, view: &mut ViewContext, subject, profile, descriptor| {
        let pin = DefinitionPin {
            version: store.get_object(descriptor, *view)?.unwrap().reference(),
        };
        store.record_classification(
            *view,
            ClassificationInput {
                subject,
                profile,
                descriptor: pin,
                source,
                starts_at: view.day,
                ends_before: None,
            },
            Uuid::new_v4(),
        )?;
        view.revision = AuthorRevision(view.revision.0 + 1);
        Ok::<(), Error>(())
    };
    let relate = |store: &mut Store, view: &mut ViewContext, subject, target, kind, role: &str| {
        let id = store.record_network_fact(
            *view,
            &NetworkInput {
                subject,
                target,
                kind,
                role: role.into(),
                source,
                starts_at: view.day.0,
                ends_before: None,
            },
            Uuid::new_v4(),
        )?;
        view.revision = AuthorRevision(view.revision.0 + 1);
        Ok::<Uuid, Error>(id)
    };
    for task in ["culture_language", "novel_category", "site_area"] {
        let before = view;
        let started = Instant::now();
        let mut rejected = 0;
        let structured = match (variant, task) {
            ('A', "culture_language") => {
                let note = ObjectRef::new(world.id);
                commit(
                    &mut store,
                    &mut view,
                    vec![Change::Create(
                        note,
                        claim("language", "Estuary culture documented Estuary speech."),
                    )],
                )?;
                assert_eq!(find_claim(&store, view, "Estuary speech")?, vec![note]);
                assert!(find_claim(&store, before, "Estuary speech")?.is_empty());
                false
            }
            (_, "culture_language") => {
                classify(
                    &mut store,
                    &mut view,
                    culture,
                    ConceptProfile::Culture,
                    vocabulary,
                )?;
                classify(
                    &mut store,
                    &mut view,
                    language,
                    ConceptProfile::Language,
                    vocabulary,
                )?;
                let invalid = NetworkInput {
                    subject: language,
                    target: culture,
                    source,
                    kind: NetworkKind::CultureLanguage,
                    role: "documented_language".into(),
                    starts_at: view.day.0,
                    ends_before: None,
                };
                assert!(matches!(
                    store.record_network_fact(view, &invalid, Uuid::new_v4()),
                    Err(Error::WrongKind)
                ));
                rejected += 1;
                let id = relate(
                    &mut store,
                    &mut view,
                    culture,
                    language,
                    NetworkKind::CultureLanguage,
                    "documented_language",
                )?;
                assert_eq!(
                    rows(
                        &store,
                        view,
                        QueryFilter::ClassifiedAs(ConceptProfile::Language)
                    )?,
                    vec![language]
                );
                let reference = FactRef {
                    world: world.id,
                    family: FactFamily::Network,
                    id,
                };
                assert!(store.resolve_semantic_fact(reference, before)?.is_none());
                assert!(store.resolve_semantic_fact(reference, view)?.is_some());
                assert!(
                    store
                        .semantic_fact_backlinks(source, view)?
                        .iter()
                        .any(|fact| fact.reference == reference)
                );
                true
            }
            ('A', "novel_category") => {
                let a = ObjectRef::new(world.id);
                let b = ObjectRef::new(world.id);
                commit(
                    &mut store,
                    &mut view,
                    vec![
                        Change::Create(a, claim("drive one", "The aetheric drive uses amber.")),
                        Change::Create(b, claim("drive two", "The aetheric drive uses amber.")),
                    ],
                )?;
                assert_eq!(find_claim(&store, view, "aetheric drive")?.len(), 2);
                assert!(find_claim(&store, before, "aetheric drive")?.is_empty());
                false
            }
            (_, "novel_category") => {
                classify(
                    &mut store,
                    &mut view,
                    material,
                    ConceptProfile::Material,
                    vocabulary,
                )?;
                for drive in [drive_a, drive_b] {
                    classify(
                        &mut store,
                        &mut view,
                        drive,
                        ConceptProfile::Technology,
                        drive_tag,
                    )?;
                    relate(
                        &mut store,
                        &mut view,
                        drive,
                        material,
                        NetworkKind::WorkingMaterial,
                        "working_medium",
                    )?;
                }
                let drives = rows(&store, view, QueryFilter::ClassifiedBy(drive_tag))?;
                assert_eq!(drives.len(), 2);
                assert!(drives.contains(&drive_a) && drives.contains(&drive_b));
                assert!(rows(&store, before, QueryFilter::ClassifiedBy(drive_tag))?.is_empty());
                let facts = store.network_facts(view, material)?;
                assert_eq!(facts.len(), 2);
                assert!(facts.iter().all(|fact| fact.input.source == source));
                true
            }
            ('C', "site_area") => {
                let pin = DefinitionPin {
                    version: store.get_object(area_tag, view)?.unwrap().reference(),
                };
                let input = SiteAreaInput {
                    place,
                    descriptor: pin,
                    source,
                    reading: AreaReading::Known {
                        amount: 12,
                        unit: AreaUnit::Hectare,
                    },
                    starts_at: view.day,
                    ends_before: None,
                };
                assert!(store.site_area(view, place)?.is_none());
                let first = store.record_site_area(view, input, None, Uuid::new_v4())?;
                view.revision = AuthorRevision(view.revision.0 + 1);
                let original = view;
                assert!(matches!(
                    store.record_site_area(view, input, None, Uuid::new_v4()),
                    Err(Error::InvalidInput("overlapping site area"))
                ));
                rejected += 1;
                let corrected = SiteAreaInput {
                    reading: AreaReading::Known {
                        amount: 13,
                        unit: AreaUnit::Hectare,
                    },
                    ..input
                };
                store.record_site_area(view, corrected, Some(first), Uuid::new_v4())?;
                view.revision = AuthorRevision(view.revision.0 + 1);
                assert_eq!(store.site_area(original, place)?.unwrap().input, input);
                assert_eq!(store.site_area(view, place)?.unwrap().input, corrected);
                let filter = QueryFilter::SiteArea(AreaTest::AtLeast {
                    amount: 13,
                    unit: AreaUnit::Hectare,
                });
                assert!(rows(&store, original, filter.clone())?.is_empty());
                assert_eq!(rows(&store, view, filter)?, vec![place]);
                true
            }
            (_, "site_area") => {
                let note = ObjectRef::new(world.id);
                commit(
                    &mut store,
                    &mut view,
                    vec![Change::Create(
                        note,
                        claim("area", "Workshop measured twelve hectares."),
                    )],
                )?;
                let original = view;
                commit(
                    &mut store,
                    &mut view,
                    vec![Change::Correct(
                        note,
                        claim("area", "Workshop measured thirteen hectares."),
                    )],
                )?;
                assert_eq!(find_claim(&store, original, "twelve hectares")?, vec![note]);
                assert_eq!(find_claim(&store, view, "thirteen hectares")?, vec![note]);
                assert!(find_claim(&store, view, "twelve hectares")?.is_empty());
                false
            }
            _ => unreachable!("fixed task list"),
        };
        println!(
            "variant={variant} task={task} completed=true structured={structured} commands={} rejected_invalid={rejected} elapsed_us={} history_verified=true evidence_verified=true",
            view.revision.0 - before.revision.0,
            started.elapsed().as_micros()
        );
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    baseline()?;
    for variant in ['A', 'B', 'C'] {
        compare(variant)?;
    }
    Ok(())
}
