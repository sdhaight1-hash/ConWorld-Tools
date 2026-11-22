//! Persistent mixed-domain competency fixture. No direct SQL or new schema.
use atlas_store::{
    archive::{inspect_archive, restore_archive},
    genealogy::{ParentageKind, RecordParentage},
    network::{NetworkInput, NetworkKind},
    query::{ObjectKindFilter, QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec},
    semantic::{
        AreaReading, AreaTest, AreaUnit, ClassificationInput, ConceptProfile, DefinitionPin,
        FactFamily, FactRef, SiteAreaInput,
    },
    *,
};
use std::collections::HashSet;
use uuid::Uuid;

const NOW: i64 = 109_500;
const SETTLEMENTS: u128 = 12;

fn reference(world: WorldId, id: u128) -> ObjectRef {
    ObjectRef {
        world,
        id: ObjectId(Uuid::from_u128(id)),
    }
}

fn create(object: ObjectRef, label: &str, data: ObjectData, start: Option<i64>) -> Change {
    Change::Create(
        object,
        Record {
            label: label.into(),
            data,
            source: None,
            valid: ValidInterval {
                start: start.map(WorldDay),
                end_exclusive: None,
            },
        },
    )
}

fn commit(store: &mut Store, view: &mut ViewContext, changes: Vec<Change>) -> Result<(), Error> {
    view.revision = store.apply(&Command {
        world: view.world,
        branch: view.branch,
        expected: view.revision,
        id: Uuid::new_v4(),
        author: "Alder Basin archivist".into(),
        rationale: "Integration world records".into(),
        scope: Scope::Author,
        changes,
    })?;
    Ok(())
}

fn edge(store: &mut Store, view: &mut ViewContext, input: NetworkInput) -> Result<FactRef, Error> {
    let id = store.record_network_fact(*view, &input, Uuid::new_v4())?;
    view.revision.0 += 1;
    Ok(FactRef {
        world: view.world,
        family: FactFamily::Network,
        id,
    })
}

fn rows(
    store: &Store,
    view: ViewContext,
    filter: QueryFilter,
) -> Result<HashSet<ObjectRef>, Error> {
    let result = store.execute_query(
        &QuerySpec {
            schema_version: QUERY_SCHEMA_VERSION,
            filter,
            limit: 100,
        },
        view,
    )?;
    assert!(!result.truncated);
    Ok(result.rows.into_iter().map(|row| row.object).collect())
}

fn within(root: ObjectRef) -> QueryFilter {
    QueryFilter::WithinPlace {
        root,
        kind: NetworkKind::ProperPartWithin,
        lens: "physical".into(),
        include_self: false,
        max_depth: 4,
    }
}

#[test]
fn alder_basin_persists_interconnected_history_and_authorized_replay()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = std::env::var_os("ATLAS_INTEGRATION_WORLD")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| dir.path().join("alder-basin.sqlite"));
    // Never overwrite an author database, even when optional retained output is requested.
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let mut store = Store::open(&path)?;
    let world = store.create_world("Alder Basin — integration fixture", Scope::Author)?;
    let object = |id| reference(world.id, id);
    let mut view = ViewContext {
        world: world.id,
        branch: world.main,
        revision: world.revision,
        day: WorldDay(NOW),
        scope: Scope::Author,
    };
    let source = object(1);
    let basin = object(2);
    let guild = object(3);
    let faction = object(4);
    let rival = object(5);
    let conflict = object(6);
    let descriptor = object(7);
    let area = object(8);
    let culture = object(9);
    let language = object(10);
    let material = object(11);
    let technology = object(12);
    let founding = object(20);
    let charter = object(21);
    let treaty = object(22);
    let tag = |description: &str| ObjectData::Definition {
        definition: Definition::Tag,
        description: description.into(),
    };
    let mut changes = vec![
        create(
            source,
            "Basin register",
            ObjectData::Source {
                citation: "Synthetic register, book I".into(),
            },
            None,
        ),
        create(basin, "Alder Basin", ObjectData::Place, None),
        create(guild, "River Guild", ObjectData::Organization, None),
        create(faction, "Harbor League", ObjectData::Faction, None),
        create(rival, "Hill League", ObjectData::Faction, None),
        create(conflict, "Toll dispute", ObjectData::Conflict, None),
        create(
            descriptor,
            "Regional vocabulary",
            tag("Semantic profiles"),
            None,
        ),
        create(area, "Survey area", tag("Measured site area"), None),
        create(culture, "Alder culture", ObjectData::Concept, None),
        create(language, "River Speech", ObjectData::Concept, None),
        create(material, "Copper", ObjectData::Concept, None),
        create(technology, "River pump", ObjectData::Concept, None),
        // Unsupported domain semantics remain named concepts, not invented typed facts.
        create(object(13), "Lantern faith", ObjectData::Concept, None),
        create(
            object(14),
            "Silver crown currency",
            ObjectData::Concept,
            None,
        ),
        create(object(15), "Salt commodity", ObjectData::Concept, None),
    ];
    for (event, label, day) in [
        (founding, "First settlement", 0),
        (charter, "River charter", 36_500),
        (treaty, "Toll treaty", 73_000),
    ] {
        changes.push(create(
            event,
            label,
            ObjectData::HistoricalEvent {
                occurrence: EventDate::Exact(WorldDay(day)),
            },
            None,
        ));
    }
    for d in 0..3 {
        changes.push(create(
            object(30 + d),
            &format!("District {d}"),
            ObjectData::Place,
            None,
        ));
    }
    for i in 0..SETTLEMENTS {
        changes.push(create(
            object(100 + i),
            &format!("Settlement {i:02}"),
            ObjectData::Place,
            None,
        ));
        changes.push(create(
            object(200 + i),
            &format!("House {i:02}"),
            ObjectData::FamilyHouse,
            None,
        ));
        changes.push(create(
            object(300 + i),
            &format!("Mill {i:02}"),
            ObjectData::Asset,
            None,
        ));
        for g in 0..4 {
            let born = 73_000 + 10_950 * g as i64;
            changes.push(create(
                object(1000 + i * 10 + g),
                &format!("House {i:02} generation {g}"),
                ObjectData::Person,
                Some(born),
            ));
        }
    }
    for (id, audience, statement) in [
        (40, ClaimAudience::Public, "A toll agreement was announced."),
        (
            41,
            ClaimAudience::Author,
            "The private register disputes the vote.",
        ),
        (
            42,
            ClaimAudience::Character(object(1000)),
            "The clerk witnessed a concealed payment.",
        ),
    ] {
        changes.push(Change::Create(
            object(id),
            Record {
                label: format!("Treaty account {id}"),
                data: ObjectData::Claim {
                    subject: treaty,
                    predicate: "treaty_account".into(),
                    statement: statement.into(),
                    audience,
                },
                source: Some(source),
                valid: ValidInterval {
                    start: Some(WorldDay(73_000)),
                    end_exclusive: None,
                },
            },
        ));
    }
    commit(&mut store, &mut view, changes)?;
    let before_facts = view;
    let input = |subject, target, kind, role: &str, starts_at| NetworkInput {
        subject,
        target,
        source,
        kind,
        role: role.into(),
        starts_at,
        ends_before: None,
    };

    let pin = DefinitionPin {
        version: store.get_object(descriptor, view)?.unwrap().reference(),
    };
    for (subject, profile) in [
        (culture, ConceptProfile::Culture),
        (language, ConceptProfile::Language),
        (material, ConceptProfile::Material),
        (technology, ConceptProfile::Technology),
    ] {
        store.record_classification(
            view,
            ClassificationInput {
                subject,
                profile,
                descriptor: pin,
                source,
                starts_at: WorldDay(0),
                ends_before: None,
            },
            Uuid::new_v4(),
        )?;
        view.revision.0 += 1;
    }
    let linguistic_fact = edge(
        &mut store,
        &mut view,
        input(
            culture,
            language,
            NetworkKind::CultureLanguage,
            "documented_language",
            0,
        ),
    )?;
    edge(
        &mut store,
        &mut view,
        input(
            technology,
            material,
            NetworkKind::WorkingMaterial,
            "working_medium",
            0,
        ),
    )?;
    for d in 0..3 {
        edge(
            &mut store,
            &mut view,
            input(
                object(30 + d),
                basin,
                NetworkKind::ProperPartWithin,
                "physical",
                0,
            ),
        )?;
    }
    let area_pin = DefinitionPin {
        version: store.get_object(area, view)?.unwrap().reference(),
    };
    for i in 0..SETTLEMENTS {
        let place = object(100 + i);
        let house = object(200 + i);
        let mill = object(300 + i);
        edge(
            &mut store,
            &mut view,
            input(
                place,
                object(30 + i % 3),
                NetworkKind::ProperPartWithin,
                "physical",
                36_500,
            ),
        )?;
        edge(
            &mut store,
            &mut view,
            input(house, place, NetworkKind::LocatedAt, "headquarters", 36_500),
        )?;
        edge(
            &mut store,
            &mut view,
            input(
                mill,
                place,
                NetworkKind::LocatedAt,
                "asset_placement",
                36_500,
            ),
        )?;
        edge(
            &mut store,
            &mut view,
            input(house, mill, NetworkKind::Ownership, "owner", 36_500),
        )?;
        store.record_site_area(
            view,
            SiteAreaInput {
                place,
                descriptor: area_pin,
                source,
                reading: match i {
                    0 => AreaReading::Unknown,
                    1 => AreaReading::Known {
                        amount: 0,
                        unit: AreaUnit::Hectare,
                    },
                    _ => AreaReading::Known {
                        amount: 12,
                        unit: AreaUnit::Hectare,
                    },
                },
                starts_at: WorldDay(73_000),
                ends_before: None,
            },
            None,
            Uuid::new_v4(),
        )?;
        view.revision.0 += 1;
        for g in 0..4 {
            let person = object(1000 + i * 10 + g);
            let born = 73_000 + 10_950 * g as i64;
            edge(
                &mut store,
                &mut view,
                input(
                    person,
                    house,
                    NetworkKind::Membership,
                    "family member",
                    born,
                ),
            )?;
            edge(
                &mut store,
                &mut view,
                input(person, place, NetworkKind::LocatedAt, "residence", born),
            )?;
            if g > 0 {
                store.record_parentage(
                    view,
                    RecordParentage {
                        parent: object(1000 + i * 10 + g - 1),
                        child: person,
                        kind: ParentageKind::Biological,
                        starts_at: WorldDay(born),
                        ends_before: None,
                        command_id: Uuid::new_v4(),
                        rationale: "Synthetic lineage register",
                    },
                )?;
                view.revision.0 += 1;
            }
        }
    }
    for (subject, target, kind, role, day) in [
        (
            faction,
            conflict,
            NetworkKind::ConflictPosition,
            "supports",
            73_000,
        ),
        (
            rival,
            conflict,
            NetworkKind::ConflictPosition,
            "opposes",
            73_000,
        ),
        (
            founding,
            charter,
            NetworkKind::ConfirmedCause,
            "recorded cause",
            36_500,
        ),
        (
            charter,
            treaty,
            NetworkKind::ConfirmedCause,
            "recorded cause",
            73_000,
        ),
        (
            treaty,
            object(100),
            NetworkKind::LocatedAt,
            "event_site",
            73_000,
        ),
        (
            object(1003),
            guild,
            NetworkKind::Membership,
            "millwright",
            105_850,
        ),
    ] {
        edge(
            &mut store,
            &mut view,
            input(subject, target, kind, role, day),
        )?;
    }

    let expected_places: HashSet<_> = (0..3)
        .map(|d| object(30 + d))
        .chain((0..SETTLEMENTS).map(|i| object(100 + i)))
        .collect();
    assert_eq!(rows(&store, view, within(basin))?, expected_places);
    let early = ViewContext {
        day: WorldDay(36_499),
        ..view
    };
    assert_eq!(
        rows(&store, early, within(basin))?,
        (0..3).map(|d| object(30 + d)).collect()
    );
    assert!(rows(&store, before_facts, within(basin))?.is_empty());
    assert_eq!(
        rows(&store, view, QueryFilter::Kind(ObjectKindFilter::Person))?.len(),
        48
    );
    assert_eq!(
        rows(
            &store,
            ViewContext {
                day: WorldDay(73_000),
                ..view
            },
            QueryFilter::Kind(ObjectKindFilter::Person),
        )?
        .len(),
        12
    );
    assert_eq!(
        rows(
            &store,
            view,
            QueryFilter::And(vec![
                QueryFilter::ClassifiedBy(descriptor),
                QueryFilter::ClassifiedAs(ConceptProfile::Language),
            ]),
        )?,
        HashSet::from([language])
    );
    assert_eq!(
        rows(&store, view, QueryFilter::SiteArea(AreaTest::Unknown))?,
        HashSet::from([object(100)])
    );
    assert_eq!(
        rows(
            &store,
            view,
            QueryFilter::SiteArea(AreaTest::AtLeast {
                amount: 0,
                unit: AreaUnit::Hectare,
            }),
        )?
        .len(),
        11
    );
    let evidence = store.evidence_at(treaty, view)?;
    assert_eq!(evidence.len(), 3);
    assert!(evidence.iter().all(|row| row.source.is_some()));
    let public = ViewContext {
        scope: Scope::Public,
        ..view
    };
    let reader = store.evidence_at(treaty, public)?;
    assert_eq!(reader.len(), 1);
    assert_eq!(reader[0].statement, "A toll agreement was announced.");
    assert!(reader[0].source.is_none());
    assert!(matches!(
        store.resolve_semantic_fact(linguistic_fact, public),
        Err(Error::CapabilityDenied)
    ));
    assert!(
        store
            .resolve_semantic_fact(linguistic_fact, before_facts)?
            .is_none()
    );
    let original = store.resolve_semantic_fact(linguistic_fact, view)?.unwrap();

    // Reject a same-lens containment cycle atomically.
    let cycle = input(
        basin,
        object(100),
        NetworkKind::ProperPartWithin,
        "physical",
        NOW,
    );
    assert!(matches!(
        store.record_network_fact(view, &cycle, Uuid::new_v4()),
        Err(Error::InvalidInput(_))
    ));
    assert_eq!(
        store.world(world.id, Scope::Author)?.revision,
        view.revision
    );
    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: view.day,
        expected: view.revision,
        id: Uuid::new_v4(),
        author: "Alder Basin archivist".into(),
        rationale: "Alternative treaty aftermath".into(),
        scope: Scope::Author,
    })?;
    let mut alternative = ViewContext {
        branch: fork.id,
        revision: fork.created_revision,
        ..view
    };
    let branch_fact = edge(
        &mut store,
        &mut alternative,
        input(
            object(1003),
            rival,
            NetworkKind::Membership,
            "alternate allegiance",
            NOW,
        ),
    )?;
    view.revision = alternative.revision;
    assert!(store.resolve_semantic_fact(branch_fact, view)?.is_none());
    let branch_record = store
        .resolve_semantic_fact(branch_fact, alternative)?
        .unwrap();
    assert_eq!(branch_record.source_branch, fork.id);
    assert_eq!(
        store.resolve_semantic_fact(linguistic_fact, alternative)?,
        Some(original.clone())
    );
    store.verify_foreign_keys(Scope::Author)?;
    drop(store);

    // Reopen and independent archive restore must preserve the same IDs and results.
    let manifest = inspect_archive(&path, Scope::Author)?;
    assert_eq!(manifest.worlds.len(), 1);
    assert_eq!(manifest.worlds[0].revision, view.revision);
    let restored = dir.path().join("restored.sqlite");
    assert_eq!(restore_archive(&path, &restored, Scope::Author)?, manifest);
    for db in [&path, &restored] {
        let reopened = Store::open(db)?;
        assert_eq!(rows(&reopened, view, within(basin))?, expected_places);
        assert_eq!(
            reopened.resolve_semantic_fact(linguistic_fact, view)?,
            Some(original.clone())
        );
        assert_eq!(
            reopened.resolve_semantic_fact(branch_fact, alternative)?,
            Some(branch_record.clone())
        );
        assert!(reopened.resolve_semantic_fact(branch_fact, view)?.is_none());
        reopened.verify_foreign_keys(Scope::Author)?;
    }
    println!("Alder Basin fixture: 108 objects, 300 years, 48 people, 12 settlements");
    Ok(())
}
