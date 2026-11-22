use atlas_store::{
    access::can_view,
    archive::{ARCHIVE_FORMAT_VERSION, inspect_archive},
    conflict::{CausalLink, ConflictPosition, LinkStatus, Stance, positions_at},
    genealogy::{Parentage, ParentageKind, Pedigree},
    inheritance::{MODEL_LENGTH, MODEL_VERSION, fictional_segments},
    layout::{
        DisplayContent, ElementContent, ElementId, LAYOUT_SCHEMA_VERSION, LayoutDocument,
        LayoutElement,
    },
    query::{Direction, QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec},
    *,
};
use uuid::Uuid;

fn id(world: &World, number: u128) -> ObjectRef {
    ObjectRef {
        world: world.id,
        id: ObjectId(Uuid::from_u128(number)),
    }
}

fn interval(from: i64, until: Option<i64>) -> ValidInterval {
    ValidInterval {
        start: Some(WorldDay(from)),
        end_exclusive: until.map(WorldDay),
    }
}

fn view(world: &World, day: i64) -> ViewContext {
    ViewContext {
        world: world.id,
        branch: world.main,
        day: WorldDay(day),
        revision: AuthorRevision(1),
        scope: Scope::Author,
    }
}

fn query(filter: QueryFilter) -> QuerySpec {
    QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter,
        limit: 20,
    }
}

fn seed(store: &mut Store) -> Result<World, Error> {
    let world = store.create_world("Northbridge", Scope::Author)?;
    let object = |n| id(&world, n);
    let record = |n, label: &str, data, valid| {
        Change::Create(
            object(n),
            Record {
                label: label.into(),
                valid,
                data,
                source: None,
            },
        )
    };
    let default = ValidInterval::default();
    let bond = |subject, target| ObjectData::Relationship {
        subject: object(subject),
        target: object(target),
        kind: "friendship".into(),
        trust: None,
        tags: vec![],
    };
    let event = |day| ObjectData::HistoricalEvent {
        occurrence: EventDate::Exact(WorldDay(day)),
    };
    store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: world.revision,
        id: Uuid::new_v4(),
        author: "Fixture author".into(),
        rationale: "Exercise the roadmap foundations".into(),
        scope: Scope::Author,
        changes: vec![
            record(1, "Alice Vale", ObjectData::Person, default),
            record(2, "Beatrice Thorne", ObjectData::Person, default),
            record(3, "Alice Vale", ObjectData::Person, default),
            record(4, "Friendship", bond(1, 2), interval(0, Some(10))),
            record(5, "Friendship", bond(2, 3), interval(0, None)),
            record(6, "Letter disclosure", event(10), interval(10, None)),
            record(7, "Council recess", event(12), interval(12, None)),
            record(8, "Later coincidence", event(13), interval(13, None)),
            record(
                9,
                "Council minutes",
                ObjectData::Source {
                    citation: "Folio 14".into(),
                },
                default,
            ),
            record(
                10,
                "Beatrice acted",
                ObjectData::EventInvolvement {
                    event: object(6),
                    participant: object(2),
                    role: EventRole::Actor,
                },
                interval(10, None),
            ),
            record(11, "Same-day adjournment", event(12), interval(12, None)),
            record(12, "Adoptive parent", ObjectData::Person, default),
            record(13, "Disputed parent", ObjectData::Person, default),
            record(14, "Legal parent", ObjectData::Person, default),
            record(
                15,
                "Alternative biological parent",
                ObjectData::Person,
                default,
            ),
        ],
    })?;
    Ok(world)
}

#[test]
fn phase_3_queries_use_one_view_and_do_not_resolve_collisions() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let exact = query(QueryFilter::ExactLabel("alice vale".into()));
    let result = store.execute_query(&exact, view(&world, 10))?;
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.revision, AuthorRevision(1));
    let old_revision = ViewContext {
        revision: AuthorRevision(0),
        ..view(&world, 10)
    };
    assert!(store.execute_query(&exact, old_revision)?.rows.is_empty());
    let observer = ViewContext {
        scope: Scope::Public,
        ..view(&world, 10)
    };
    assert!(matches!(
        store.execute_query(&exact, observer),
        Err(Error::CapabilityDenied)
    ));
    let role = query(QueryFilter::EventRole {
        event: id(&world, 6),
        role: EventRole::Actor,
    });
    assert_eq!(
        store.execute_query(&role, view(&world, 10))?.rows[0].object,
        id(&world, 10)
    );
    let links = query(QueryFilter::Backlinks(id(&world, 6)));
    assert_eq!(
        store.execute_query(&links, view(&world, 10))?.rows[0].object,
        id(&world, 10)
    );
    assert!(
        store
            .execute_query(&links, view(&world, 9))?
            .rows
            .is_empty()
    );
    let path = |max_visited| {
        query(QueryFilter::RelationshipPath {
            start: id(&world, 1),
            kind: "friendship".into(),
            direction: Direction::Outgoing,
            max_depth: 2,
            max_visited,
        })
    };
    assert_eq!(
        store.execute_query(&path(3), view(&world, 9))?.rows.len(),
        2
    );
    let bounded = store.execute_query(&path(2), view(&world, 9))?;
    assert!(bounded.truncated);
    assert_eq!(bounded.rows[0].object, id(&world, 2));
    assert!(
        store
            .execute_query(&path(3), view(&world, 10))?
            .rows
            .is_empty()
    );
    Ok(())
}

#[test]
fn phase_4_reparent_preserves_binding_and_skips_hidden_subtrees() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let eid = |n| ElementId(Uuid::from_u128(n));
    let element = |n, audience, content| LayoutElement {
        id: eid(n),
        audience,
        content,
    };
    let binding = query(QueryFilter::LabelContains("Beatrice Thorne".into()));
    let slot = element(
        3,
        ClaimAudience::Public,
        ElementContent::Query(binding.clone()),
    );
    let bad_binding = QuerySpec {
        schema_version: 999,
        ..binding
    };
    let mut layout = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: id(&world, 1),
        root: element(
            1,
            ClaimAudience::Public,
            ElementContent::Stack(vec![
                element(2, ClaimAudience::Author, ElementContent::Stack(vec![slot])),
                element(4, ClaimAudience::Public, ElementContent::Stack(vec![])),
                element(5, ClaimAudience::Author, ElementContent::Query(bad_binding)),
            ]),
        ),
    };
    let public = ViewContext {
        scope: Scope::Public,
        ..view(&world, 10)
    };
    let before = layout.compile(&store, public)?;
    layout.reparent(eid(3), eid(4))?;
    assert_eq!(before, layout.compile(&store, public)?);
    let unchanged = layout.clone();
    assert!(layout.reparent(eid(4), eid(3)).is_err());
    assert_eq!(layout, unchanged);
    let ElementContent::Stack(children) = &mut layout.root.content else {
        unreachable!()
    };
    children.pop();
    let author = layout.compile(&store, view(&world, 10))?.unwrap();
    let DisplayContent::Stack(children) = author.content else {
        panic!("expected stack")
    };
    let DisplayContent::Stack(moved) = &children[1].content else {
        panic!("expected destination")
    };
    assert_eq!(moved[0].id, eid(3));
    assert_eq!(
        moved[0].content,
        DisplayContent::QueryRows {
            records: vec![(id(&world, 2), "Beatrice Thorne".into())],
            truncated: false,
        }
    );
    Ok(())
}

#[test]
fn phases_5_and_8_use_only_accepted_dated_biological_links_for_fictional_segments()
-> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let child = id(&world, 3);
    let link = |n, kind, accepted| Parentage {
        child,
        parent: id(&world, n),
        kind,
        valid: interval(1, None),
        accepted,
    };
    let mut links = vec![
        link(1, ParentageKind::Biological, true),
        link(2, ParentageKind::Biological, true),
        link(12, ParentageKind::Adoptive, true),
        link(13, ParentageKind::Biological, false),
    ];
    let prior = store.pedigree_at(view(&world, 0), &links)?;
    assert!(prior.parents(child, ParentageKind::Biological)?.is_empty());
    let pedigree = store.pedigree_at(view(&world, 1), &links)?;
    let mut wrong_kind = links.clone();
    wrong_kind.push(link(4, ParentageKind::Legal, true));
    assert!(store.pedigree_at(view(&world, 1), &wrong_kind).is_err());
    assert_eq!(
        pedigree.parents(child, ParentageKind::Adoptive)?,
        vec![id(&world, 12)]
    );
    let model = fictional_segments(&pedigree, child, 481, MODEL_VERSION)?;
    links.reverse();
    links.push(link(14, ParentageKind::Legal, true));
    let legal_change = store.pedigree_at(view(&world, 1), &links)?;
    assert_eq!(
        model,
        fictional_segments(&legal_change, child, 481, MODEL_VERSION)?
    );
    assert_eq!(
        (model[0].start, model[0].end, model[1].start, model[1].end),
        (0, model[1].start, model[0].end, MODEL_LENGTH)
    );
    let replacement = links
        .iter_mut()
        .find(|link| link.parent == id(&world, 2))
        .unwrap();
    replacement.parent = id(&world, 15);
    let biological_change = store.pedigree_at(view(&world, 1), &links)?;
    assert_ne!(
        model,
        fictional_segments(&biological_change, child, 481, MODEL_VERSION)?
    );
    links.push(Parentage {
        parent: child,
        child: id(&world, 1),
        ..link(1, ParentageKind::Biological, true)
    });
    assert!(Pedigree::at(world.id, WorldDay(1), &links).is_err());
    Ok(())
}

#[test]
fn phase_6_impact_ignores_rumors_and_conflict_positions_change_over_time() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let link = |cause, consequence, status| CausalLink {
        cause: id(&world, cause),
        consequence: id(&world, consequence),
        source: id(&world, 9),
        status,
    };
    let links = [
        link(6, 7, LinkStatus::Confirmed),
        link(6, 8, LinkStatus::Alleged),
    ];
    let start = id(&world, 6);
    assert_eq!(
        store
            .confirmed_impact(view(&world, 13), start, &links, 4, 8)?
            .events,
        vec![id(&world, 7)]
    );
    assert!(
        store
            .confirmed_impact(view(&world, 10), start, &links, 4, 8)?
            .events
            .is_empty()
    );
    assert!(
        store
            .confirmed_impact(view(&world, 13), start, &links, 4, 1)?
            .truncated
    );
    assert!(
        store
            .confirmed_impact(
                view(&world, 13),
                start,
                &[
                    link(7, 11, LinkStatus::Confirmed),
                    link(11, 7, LinkStatus::Confirmed)
                ],
                4,
                8
            )
            .is_err()
    );

    let position = |faction, stance, valid| ConflictPosition {
        issue: start,
        faction: id(&world, faction),
        stance,
        valid,
    };
    let positions = [
        position(1, Stance::Supports, interval(0, Some(10))),
        position(1, Stance::Opposes, interval(10, None)),
        position(2, Stance::Opposes, interval(0, None)),
        position(3, Stance::Mediates, interval(0, None)),
    ];
    assert_eq!(positions_at(start, WorldDay(9), &positions)?.len(), 3);
    assert_eq!(
        positions_at(start, WorldDay(10), &positions)?[0].stance,
        Stance::Opposes
    );
    Ok(())
}

#[test]
fn phase_7_evidence_and_layout_share_character_audience_rules() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let alice = id(&world, 1);
    let beatrice = id(&world, 2);
    let audience = ClaimAudience::Character(alice);
    assert!(can_view(Scope::Author, audience));
    assert!(can_view(Scope::Character(alice), audience));
    assert!(!can_view(Scope::Character(beatrice), audience));
    assert!(!can_view(Scope::Public, audience));
    let layout = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: alice,
        root: LayoutElement {
            id: ElementId(Uuid::from_u128(1)),
            audience: ClaimAudience::Public,
            content: ElementContent::Stack(vec![LayoutElement {
                id: ElementId(Uuid::from_u128(2)),
                audience,
                content: ElementContent::Text("Private account".into()),
            }]),
        },
    };
    let scoped = |scope| ViewContext {
        scope,
        ..view(&world, 10)
    };
    assert_ne!(
        layout.compile(&store, scoped(Scope::Character(alice)))?,
        layout.compile(&store, scoped(Scope::Public))?
    );
    assert_eq!(
        layout.compile(&store, scoped(Scope::Character(beatrice)))?,
        layout.compile(&store, scoped(Scope::Public))?
    );
    Ok(())
}

#[test]
fn phase_9_backup_is_verified_read_only_and_preserves_query_references() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("backup.sqlite");
    store.backup_to(&path, Scope::Author)?;
    let before = std::fs::read(&path)?;
    let manifest = inspect_archive(&path, Scope::Author)?;
    assert_eq!(manifest.format, ARCHIVE_FORMAT_VERSION);
    assert_eq!(manifest.worlds[0].revision, AuthorRevision(1));
    assert_eq!(manifest.worlds[0].id, world.id);
    assert_eq!(before, std::fs::read(&path)?);
    assert!(matches!(
        inspect_archive(&path, Scope::Public),
        Err(Error::CapabilityDenied)
    ));
    let reopened = Store::open(&path)?;
    let rows = reopened
        .execute_query(
            &query(QueryFilter::Backlinks(id(&world, 6))),
            view(&world, 13),
        )?
        .rows;
    assert_eq!(rows[0].object, id(&world, 10));
    let bogus = temp.path().join("unrelated.sqlite");
    rusqlite::Connection::open(&bogus)?.execute_batch("CREATE TABLE unrelated (value TEXT)")?;
    assert!(matches!(
        inspect_archive(&bogus, Scope::Author),
        Err(Error::WrongDatabase)
    ));
    Ok(())
}

#[test]
fn spatial_facts_preserve_temporal_lenses_and_branch_history() -> Result<(), Error> {
    use atlas_store::network::{IdentityKind, NetworkInput, NetworkKind};
    let db = tempfile::tempdir().unwrap();
    let path = db.path().join("spatial.sqlite");
    let mut store = Store::open(&path)?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let mut places = Vec::new();
    for label in ["Valley", "Town", "Manor"] {
        places.push(store.create_identity(author, IdentityKind::Place, label, Uuid::new_v4())?);
        author.revision.0 += 1;
    }
    let (valley, town, manor) = (places[0], places[1], places[2]);
    let make = |subject, target, kind, role: &str, starts_at, ends_before| NetworkInput {
        subject,
        target,
        source: id(&world, 9),
        kind,
        role: role.into(),
        starts_at,
        ends_before,
    };
    let first = make(
        town,
        valley,
        NetworkKind::ProperPartWithin,
        "physical",
        13,
        Some(20),
    );
    let fact = store.record_network_fact(author, &first, Uuid::new_v4())?;
    author.revision.0 += 1;
    let second = make(
        manor,
        town,
        NetworkKind::ProperPartWithin,
        "physical",
        13,
        None,
    );
    store.record_network_fact(author, &second, Uuid::new_v4())?;
    author.revision.0 += 1;
    let cycle = make(
        valley,
        manor,
        NetworkKind::ProperPartWithin,
        "physical",
        13,
        Some(20),
    );
    assert!(matches!(
        store.record_network_fact(author, &cycle, Uuid::new_v4()),
        Err(Error::InvalidInput("spatial containment cycle"))
    ));
    let disjoint = NetworkInput {
        starts_at: 20,
        ends_before: None,
        ..cycle
    };
    store.record_network_fact(author, &disjoint, Uuid::new_v4())?;
    author.revision.0 += 1;
    let admin = make(valley, town, NetworkKind::AdminPartOf, "county", 13, None);
    store.record_network_fact(author, &admin, Uuid::new_v4())?;
    author.revision.0 += 1;
    let placement = make(
        id(&world, 1),
        manor,
        NetworkKind::LocatedAt,
        "residence",
        13,
        None,
    );
    store.record_network_fact(author, &placement, Uuid::new_v4())?;
    author.revision.0 += 1;
    let spec = query(QueryFilter::WithinPlace {
        root: valley,
        kind: NetworkKind::ProperPartWithin,
        lens: "physical".into(),
        include_self: false,
        max_depth: 8,
    });
    let rows = store.execute_query(&spec, author)?.rows;
    assert_eq!(
        rows.iter().map(|row| row.object).collect::<Vec<_>>(),
        vec![manor, town]
    );
    assert!(
        store
            .execute_query(
                &spec,
                ViewContext {
                    day: WorldDay(20),
                    ..author
                }
            )?
            .rows
            .is_empty()
    );
    assert!(matches!(
        store.execute_query(
            &spec,
            ViewContext {
                scope: Scope::Public,
                ..author
            }
        ),
        Err(Error::CapabilityDenied)
    ));
    let fork = store.fork(&ForkCommand {
        world: author.world,
        parent: author.branch,
        day: author.day,
        expected: author.revision,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Spatial alternative".into(),
        scope: Scope::Author,
    })?;
    author.revision = fork.created_revision;
    let mut child = ViewContext {
        branch: fork.id,
        ..author
    };
    store.withdraw_network_fact(child, fact, Uuid::new_v4())?;
    child.revision.0 += 1;
    author.revision = child.revision;
    assert!(store.execute_query(&spec, child)?.rows.is_empty());
    assert_eq!(store.execute_query(&spec, author)?.rows.len(), 2);
    store.verify_foreign_keys(Scope::Author)?;
    drop(store);
    let store = Store::open(&path)?;
    assert_eq!(store.execute_query(&spec, author)?.rows.len(), 2);
    assert!(store.execute_query(&spec, child)?.rows.is_empty());
    Ok(())
}

#[test]
fn character_page_replays_author_queries_and_revokes_reader_access_after_restore()
-> Result<(), Error> {
    use atlas_store::archive::restore_archive;
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let entry = id(&world, 1);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let document = LayoutDocument {
        entry,
        schema_version: LAYOUT_SCHEMA_VERSION,
        root: block(
            ClaimAudience::Public,
            ElementContent::Stack(vec![
                block(
                    ClaimAudience::Public,
                    ElementContent::Text("Gazette biography".into()),
                ),
                block(
                    ClaimAudience::Author,
                    ElementContent::Stack(vec![
                        block(
                            ClaimAudience::Public,
                            ElementContent::Text("Sealed testimony".into()),
                        ),
                        block(
                            ClaimAudience::Public,
                            ElementContent::Query(query(QueryFilter::ExactLabel(
                                "Alice Vale".into(),
                            ))),
                        ),
                    ]),
                ),
            ]),
        ),
    };
    let author = view(&world, 13);
    let public = |revision| ViewContext {
        revision,
        scope: Scope::Public,
        ..author
    };
    assert!(
        store
            .display_entry(public(author.revision), entry)?
            .is_none()
    );
    let mut unsafe_binding = document.clone();
    if let ElementContent::Stack(blocks) = &mut unsafe_binding.root.content {
        blocks[1].audience = ClaimAudience::Public;
    }
    assert!(matches!(
        store.save_entry_layout(author, &unsafe_binding, true, Uuid::new_v4()),
        Err(Error::CapabilityDenied)
    ));
    let published = store.save_entry_layout(author, &document, true, Uuid::new_v4())?;
    assert!(matches!(
        store.save_entry_layout(author, &document, false, Uuid::new_v4()),
        Err(Error::RevisionConflict { .. })
    ));
    assert!(matches!(
        store.entry_layout(public(published), entry),
        Err(Error::CapabilityDenied)
    ));
    let page = store.display_entry(public(published), entry)?.unwrap();
    let DisplayContent::Stack(visible) = page.content else {
        panic!("expected page")
    };
    assert_eq!(visible.len(), 1);
    assert_eq!(
        visible[0].content,
        DisplayContent::Text("Gazette biography".into())
    );
    let hits = store
        .search_entry_text(public(published), entry, " GAZETTE ", 1)?
        .unwrap();
    assert_eq!(
        hits.matches,
        vec![(visible[0].id, "Gazette biography".into())]
    );
    assert!(!hits.truncated);
    let hidden = store
        .search_entry_text(public(published), entry, "Sealed testimony", 1)?
        .unwrap();
    assert!(hidden.matches.is_empty());
    assert!(!hidden.truncated);
    assert!(matches!(
        store.search_entry_text(public(published), entry, "", 1),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        store.search_entry_text(public(published), entry, "Gazette", 51),
        Err(Error::InvalidInput(_))
    ));
    let pinned = ViewContext {
        revision: published,
        ..author
    };
    assert_eq!(
        store
            .search_entry_text(pinned, entry, "Sealed", 1)?
            .unwrap()
            .matches
            .len(),
        1
    );
    let projected = store.display_entry(pinned, entry)?.unwrap();
    let DisplayContent::Stack(children) = projected.content else {
        panic!("expected root")
    };
    let DisplayContent::Stack(private) = &children[1].content else {
        panic!("expected notes")
    };
    assert!(
        matches!(&private[1].content, DisplayContent::QueryRows{records, ..} if records.len()==2)
    );
    let revoked = store.save_entry_layout(pinned, &document, false, Uuid::new_v4())?;
    let dir = tempfile::tempdir()?;
    let backup = dir.path().join("export.sqlite");
    let destination = dir.path().join("restored.sqlite");
    store.backup_to(&backup, Scope::Author)?;
    let manifest = restore_archive(&backup, &destination, Scope::Author)?;
    assert_eq!(manifest.worlds[0].revision, revoked);
    let before = std::fs::read(&destination)?;
    assert!(restore_archive(&backup, &destination, Scope::Author).is_err());
    assert_eq!(before, std::fs::read(&destination)?);
    let restored = Store::open(&destination)?;
    assert_eq!(
        restored.entry_layout(pinned, entry)?,
        Some((document, true))
    );
    assert!(restored.display_entry(public(revoked), entry)?.is_none());
    assert!(
        restored
            .search_entry_text(public(revoked), entry, "Gazette", 1)?
            .is_none()
    );
    assert!(matches!(
        restored.search_entry_text(public(published), entry, "Gazette", 1),
        Err(Error::InvalidRevision)
    ));
    assert!(matches!(
        restored.display_entry(public(published), entry),
        Err(Error::InvalidRevision)
    ));
    Ok(())
}

#[test]
fn focused_family_and_event_reads_survive_a_large_unrelated_population() -> Result<(), Error> {
    use atlas_store::inheritance::shared_segments;
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: AuthorRevision(1),
        id: Uuid::new_v4(),
        scope: Scope::Author,
        author: "Fixture".into(),
        rationale: "Unrelated residents".into(),
        changes: (0..2_001)
            .map(|_| {
                Change::Create(
                    ObjectRef::new(world.id),
                    Record {
                        label: "Resident".into(),
                        valid: ValidInterval::default(),
                        source: None,
                        data: ObjectData::Person,
                    },
                )
            })
            .collect(),
    })?;
    let view = ViewContext {
        revision,
        ..view(&world, 13)
    };
    assert!(matches!(
        store.objects_at(view, 2_000),
        Err(Error::QueryBudgetExceeded)
    ));
    let mut links = vec![];
    for child in [3, 12] {
        for parent in [1, 2] {
            links.push(Parentage {
                parent: id(&world, parent),
                child: id(&world, child),
                kind: ParentageKind::Biological,
                accepted: true,
                valid: ValidInterval::default(),
            });
        }
    }
    let pedigree = store.pedigree_at(view, &links)?;
    let a = id(&world, 3);
    let b = id(&world, 12);
    let shared = shared_segments(&pedigree, a, b, 481, MODEL_VERSION)?;
    assert_eq!(
        shared,
        shared_segments(&pedigree, b, a, 481, MODEL_VERSION)?
    );
    assert!(!shared.is_empty());
    for s in &shared {
        assert!(s.start < s.end && s.end <= MODEL_LENGTH);
    }
    let start = id(&world, 6);
    let impact = store.confirmed_impact(
        view,
        start,
        &[CausalLink {
            cause: start,
            consequence: id(&world, 7),
            source: id(&world, 9),
            status: LinkStatus::Confirmed,
        }],
        2,
        10,
    )?;
    assert_eq!(impact.events, vec![id(&world, 7)]);
    Ok(())
}

#[test]
fn stored_parentage_pins_days_revisions_and_rejects_future_cycles() -> Result<(), Error> {
    use ParentageKind::*;
    use atlas_store::genealogy::RecordParentage;
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("family.sqlite");
    let mut store = Store::open(&db)?;
    let world = seed(&mut store)?;
    let ctx = |rev, day| ViewContext {
        revision: AuthorRevision(rev),
        ..view(&world, day)
    };
    let link = |parent, child, kind, start, end: Option<i64>| RecordParentage {
        parent: id(&world, parent),
        child: id(&world, child),
        kind,
        starts_at: WorldDay(start),
        ends_before: end.map(WorldDay),
        command_id: Uuid::new_v4(),
        rationale: "Household register",
    };
    let first = store.record_parentage(ctx(1, 0), link(1, 2, Biological, 0, Some(20)))?;
    store.record_parentage(ctx(2, 0), link(2, 3, Adoptive, 10, Some(30)))?;
    assert!(matches!(
        store.record_parentage(ctx(3, 0), link(3, 1, Legal, 5, Some(15))),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        store.record_parentage(ctx(3, 0), link(1, 2, Biological, 5, None)),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        store.record_parentage(ctx(1, 0), link(3, 1, Legal, 20, None)),
        Err(Error::RevisionConflict { .. })
    ));
    store.record_parentage(ctx(3, 0), link(3, 1, Legal, 20, None))?;
    assert_eq!(store.family_links(ctx(4, 19), id(&world, 2))?.len(), 2);
    assert_eq!(store.family_links(ctx(4, 20), id(&world, 2))?.len(), 1);
    assert!(matches!(
        store.withdraw_parentage(
            ctx(4, 0),
            id(&world, 13),
            first,
            Uuid::new_v4(),
            "Other person"
        ),
        Err(Error::NotFound)
    ));
    store.withdraw_parentage(
        ctx(4, 0),
        id(&world, 2),
        first,
        Uuid::new_v4(),
        "Register corrected",
    )?;
    drop(store);
    let store = Store::open(&db)?;
    let before = store.family_links(ctx(4, 0), id(&world, 2))?;
    assert_eq!(before[0].rationale, "Household register");
    assert_eq!(before[0].parent, id(&world, 1));
    assert!(store.family_links(ctx(5, 0), id(&world, 2))?.is_empty());
    assert!(matches!(
        store.family_links(
            ViewContext {
                scope: Scope::Public,
                ..ctx(5, 0)
            },
            id(&world, 2)
        ),
        Err(Error::CapabilityDenied)
    ));
    Ok(())
}

#[test]
fn fictional_comparison_uses_saved_biological_links_at_the_selected_revision() -> Result<(), Error>
{
    use atlas_store::genealogy::RecordParentage;
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut context = view(&world, 1);
    let mut last = Uuid::nil();
    for (parent, child, kind) in [
        (1, 3, ParentageKind::Biological),
        (2, 3, ParentageKind::Biological),
        (1, 12, ParentageKind::Biological),
        (2, 12, ParentageKind::Biological),
        (13, 12, ParentageKind::Legal),
    ] {
        let recorded = store.record_parentage(
            context,
            RecordParentage {
                parent: id(&world, parent),
                child: id(&world, child),
                kind,
                starts_at: WorldDay(1),
                ends_before: None,
                rationale: "Register",
                command_id: Uuid::new_v4(),
            },
        )?;
        if kind == ParentageKind::Biological {
            last = recorded;
        }
        context.revision.0 += 1;
    }
    let compare = |s: &Store, v| s.family_match_at(v, id(&world, 3), id(&world, 12), 481);
    let segments = compare(&store, context)?;
    assert!(!segments.is_empty());
    store.withdraw_parentage(context, id(&world, 12), last, Uuid::new_v4(), "Correction")?;
    assert!(
        compare(
            &store,
            ViewContext {
                revision: AuthorRevision(context.revision.0 + 1),
                ..context
            }
        )
        .is_err()
    );
    assert_eq!(compare(&store, context)?, segments);
    Ok(())
}

#[test]
fn entry_find_limits_only_visible_blocks_in_layout_order() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let entry = id(&world, 1);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let first = block(
        ClaimAudience::Public,
        ElementContent::Text("First Étoile".into()),
    );
    let second = block(
        ClaimAudience::Public,
        ElementContent::Text("Second Étoile".into()),
    );
    let first_id = first.id;
    let second_id = second.id;
    let document = LayoutDocument {
        entry,
        schema_version: LAYOUT_SCHEMA_VERSION,
        root: block(
            ClaimAudience::Public,
            ElementContent::Stack(vec![
                first,
                block(
                    ClaimAudience::Author,
                    ElementContent::Stack(vec![block(
                        ClaimAudience::Public,
                        ElementContent::Text("Hidden Étoile".into()),
                    )]),
                ),
                second,
                block(
                    ClaimAudience::Character(id(&world, 2)),
                    ElementContent::Text("Reader Étoile".into()),
                ),
            ]),
        ),
    };
    let author = view(&world, 13);
    let revision = store.save_entry_layout(author, &document, true, Uuid::new_v4())?;
    let public = ViewContext {
        scope: Scope::Public,
        revision,
        ..author
    };
    let limited = store
        .search_entry_text(public, entry, "étoile", 1)?
        .unwrap();
    assert_eq!(limited.matches, vec![(first_id, "First Étoile".into())]);
    assert!(limited.truncated);
    let full = store
        .search_entry_text(public, entry, "ÉTOILE", 2)?
        .unwrap();
    assert_eq!(
        full.matches,
        vec![
            (first_id, "First Étoile".into()),
            (second_id, "Second Étoile".into())
        ]
    );
    assert!(!full.truncated);
    let reader = ViewContext {
        scope: Scope::Character(id(&world, 2)),
        ..public
    };
    let reader_hits = store
        .search_entry_text(reader, entry, "étoile", 3)?
        .unwrap();
    assert_eq!(reader_hits.matches.len(), 3);
    assert_eq!(reader_hits.matches[2].1, "Reader Étoile");
    assert!(!reader_hits.truncated);
    let other = ViewContext {
        scope: Scope::Character(id(&world, 3)),
        ..public
    };
    assert_eq!(
        store.search_entry_text(other, entry, "étoile", 2)?.unwrap(),
        full
    );
    Ok(())
}

#[test]
fn global_public_find_excludes_unreleased_and_historical_private_content() -> Result<(), Error> {
    use atlas_store::query::QueryControl;
    use std::time::Duration;

    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let foreign = seed(&mut store)?;
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let page = |entry, audience, text: &str| LayoutDocument {
        entry,
        schema_version: LAYOUT_SCHEMA_VERSION,
        root: block(
            audience,
            ElementContent::Stack(vec![
                block(ClaimAudience::Public, ElementContent::Text(text.into())),
                block(
                    ClaimAudience::Author,
                    ElementContent::Text("Secret gazette".into()),
                ),
            ]),
        ),
    };
    let mut author = view(&world, 13);
    let a = page(id(&world, 1), ClaimAudience::Public, "First gazette");
    let b = page(id(&world, 2), ClaimAudience::Public, "Second gazette");
    let draft = page(id(&world, 3), ClaimAudience::Public, "Draft gazette");
    let hidden = page(id(&world, 4), ClaimAudience::Author, "Hidden gazette");
    let future = page(id(&world, 7), ClaimAudience::Public, "Future gazette");
    for (document, release) in [(&a, true), (&b, true), (&draft, false), (&future, true)] {
        author.revision = store.save_entry_layout(author, document, release, Uuid::new_v4())?;
    }
    // The author-only root remains inaccessible even when the release flag is set.
    let early = ViewContext {
        day: WorldDay(5),
        ..author
    };
    author.revision = store.save_entry_layout(early, &hidden, true, Uuid::new_v4())?;
    let foreign_page = page(id(&foreign, 1), ClaimAudience::Public, "Foreign gazette");
    store.save_entry_layout(view(&foreign, 13), &foreign_page, true, Uuid::new_v4())?;

    let reader = |author: ViewContext, day| ViewContext {
        scope: Scope::Public,
        day: WorldDay(day),
        ..author
    };
    let control = QueryControl::new(Duration::from_secs(5))?;
    let public = reader(author, 5);
    let matches = store.search_public_pages(public, "GAZETTE", 2, &control)?;
    assert_eq!(matches.matches.len(), 2);
    assert_eq!(matches.matches[0].entry, a.entry);
    assert_eq!(matches.matches[1].entry, b.entry);
    assert_eq!(matches.matches[0].text, "First gazette");
    assert_eq!(matches.matches[1].text, "Second gazette");
    assert!(!matches.truncated);
    let limited = store.search_public_pages(public, "gazette", 1, &control)?;
    assert!(limited.truncated);
    assert_eq!(limited.matches[0], matches.matches[0]);
    let later = store.search_public_pages(reader(author, 13), "gazette", 5, &control)?;
    assert_eq!(later.matches.len(), 3);
    assert_eq!(later.matches[2].entry, future.entry);
    assert!(matches!(
        store.search_public_pages(author, "gazette", 5, &control),
        Err(Error::CapabilityDenied)
    ));
    author.revision = store.save_entry_layout(author, &a, false, Uuid::new_v4())?;
    assert!(matches!(
        store.search_public_pages(public, "gazette", 5, &control),
        Err(Error::InvalidRevision)
    ));
    let current = reader(author, 5);
    let revoked = store.search_public_pages(current, "gazette", 5, &control)?;
    assert_eq!(revoked.matches.len(), 1);
    assert_eq!(revoked.matches[0].entry, b.entry);
    control.cancel();
    assert!(matches!(
        store.search_public_pages(current, "gazette", 5, &control),
        Err(Error::QueryCancelled)
    ));
    Ok(())
}

#[test]
fn public_index_is_selective_rebuilt_and_revoked_atomically() -> Result<(), Error> {
    use atlas_store::query::QueryControl;
    use std::time::Duration;

    let dir = tempfile::tempdir()?;
    let path = dir.path().join("public-index.sqlite");
    let mut store = Store::open(&path)?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let people: Vec<_> = (0..513).map(|_| ObjectRef::new(world.id)).collect();
    author.revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: author.revision,
        id: Uuid::new_v4(),
        scope: Scope::Author,
        author: "Fixture".into(),
        rationale: "Large publication corpus".into(),
        changes: people
            .iter()
            .map(|person| {
                Change::Create(
                    *person,
                    Record {
                        label: "Unlisted author label".into(),
                        valid: ValidInterval::default(),
                        source: None,
                        data: ObjectData::Person,
                    },
                )
            })
            .collect(),
    })?;
    let page = |entry, text: &str| LayoutDocument {
        entry,
        schema_version: LAYOUT_SCHEMA_VERSION,
        root: LayoutElement {
            id: ElementId(Uuid::new_v4()),
            audience: ClaimAudience::Public,
            content: ElementContent::Stack(vec![
                LayoutElement {
                    id: ElementId(Uuid::new_v4()),
                    audience: ClaimAudience::Public,
                    content: ElementContent::Text(text.into()),
                },
                LayoutElement {
                    id: ElementId(Uuid::new_v4()),
                    audience: ClaimAudience::Author,
                    content: ElementContent::Text("Confidential needle".into()),
                },
            ]),
        },
    };
    for person in &people {
        author.revision = store.save_entry_layout(
            author,
            &page(*person, "Routine notice"),
            true,
            Uuid::new_v4(),
        )?;
    }
    let target = page(id(&world, 1), "A rare Étoile \"quoted\" entry");
    author.revision = store.save_entry_layout(author, &target, true, Uuid::new_v4())?;
    let public = |author| ViewContext {
        scope: Scope::Public,
        ..author
    };
    let find = |store: &Store, author, text| {
        let control = QueryControl::new(Duration::from_secs(10))?;
        store.search_public_pages(public(author), text, 50, &control)
    };
    let expected = find(&store, author, "ÉTOILE \"quoted\"")?;
    assert_eq!(expected.matches.len(), 1);
    assert_eq!(expected.matches[0].entry, target.entry);
    assert!(!expected.truncated);
    assert!(
        find(&store, author, "Confidential needle")?
            .matches
            .is_empty()
    );
    assert!(
        find(&store, author, "Unlisted author label")?
            .matches
            .is_empty()
    );
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(find(&store, author, "étoile \"quoted\"")?, expected);
    drop(store);

    // Recreate schema 23 and verify migration rebuilds only latest released text.
    let conn = rusqlite::Connection::open(&path)?;
    conn.execute_batch(
        "DROP TRIGGER public_page_search_ai;
         DROP TRIGGER public_page_search_ad;
         DROP TABLE public_page_search_fts;
         DROP TABLE public_page_search_documents;
         PRAGMA user_version=23;",
    )?;
    drop(conn);
    let mut store = Store::open(&path)?;
    assert_eq!(find(&store, author, "étoile \"quoted\"")?, expected);
    let backup = dir.path().join("backup.sqlite");
    store.backup_to(&backup, Scope::Author)?;
    let copied = Store::open(&backup)?;
    assert_eq!(find(&copied, author, "étoile \"quoted\"")?, expected);
    author.revision = store.save_entry_layout(author, &target, false, Uuid::new_v4())?;
    assert!(find(&store, author, "étoile")?.matches.is_empty());
    store.rebuild_search_index(Scope::Author)?;
    assert!(find(&store, author, "étoile")?.matches.is_empty());
    Ok(())
}

#[test]
fn saved_canvas_binding_pins_definition_and_replays_live_author_history() -> Result<(), Error> {
    use atlas_store::{
        layout::SavedQueryBinding,
        query::{ParameterBinding, ParameterValue, SaveNamedLabelQuery},
    };

    let dir = tempfile::tempdir()?;
    let path = dir.path().join("saved-layout.sqlite");
    let mut store = Store::open(&path)?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let saved = store.save_named_parameterized_query(SaveNamedLabelQuery {
        world_id: world.id,
        expected: author.revision,
        query_id: None,
        name: "People of interest",
        spec: &query(QueryFilter::ExactLabel("Alice Vale".into())),
        command_id: Uuid::new_v4(),
        scope: Scope::Author,
    })?;
    author.revision = saved.recorded_at;
    let binding = SavedQueryBinding {
        world: world.id,
        query_id: saved.id,
        definition_revision: saved.recorded_at.0,
        bindings: vec![ParameterBinding {
            name: "exact_1".into(),
            value: ParameterValue::Text {
                value: "Beatrice Thorne".into(),
            },
        }],
    };
    let entry = id(&world, 1);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let query_block = block(
        ClaimAudience::Author,
        ElementContent::SavedQuery(binding.clone()),
    );
    let query_id = query_block.id;
    let mut doc = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry,
        root: block(
            ClaimAudience::Public,
            ElementContent::Stack(vec![
                block(
                    ClaimAudience::Public,
                    ElementContent::Text("Released biography".into()),
                ),
                query_block,
            ]),
        ),
    };
    let control = atlas_store::query::QueryControl::new(std::time::Duration::from_secs(5))?;
    assert_eq!(
        store.preview_author_layout(author, &doc, &control)?,
        doc.compile(&store, author)?
    );
    assert!(store.entry_layout(author, entry)?.is_none());
    assert!(matches!(
        store.preview_author_layout(
            ViewContext {
                scope: Scope::Public,
                ..author
            },
            &doc,
            &control
        ),
        Err(Error::CapabilityDenied)
    ));
    let direct = store.execute_query(&binding.resolve(&store, author)?, author)?;
    assert_eq!(direct.rows.len(), 1);
    assert_eq!(direct.rows[0].object, id(&world, 2));
    let mut unsafe_doc = doc.clone();
    if let ElementContent::Stack(children) = &mut unsafe_doc.root.content {
        children[1].audience = ClaimAudience::Public;
    }
    assert!(matches!(
        store.save_entry_layout(author, &unsafe_doc, true, Uuid::new_v4()),
        Err(Error::CapabilityDenied)
    ));
    let mut foreign = binding.clone();
    foreign.world = WorldId(Uuid::new_v4());
    assert!(matches!(
        foreign.resolve(&store, author),
        Err(Error::WrongWorld)
    ));
    let mut nonexistent_version = binding.clone();
    nonexistent_version.definition_revision += 1;
    assert!(matches!(
        nonexistent_version.resolve(&store, author),
        Err(Error::InvalidRevision)
    ));
    let target = block(ClaimAudience::Author, ElementContent::Stack(vec![]));
    let target_id = target.id;
    if let ElementContent::Stack(children) = &mut doc.root.content {
        children.push(target);
    }
    doc.reparent(query_id, target_id)?;
    author.revision = store.save_entry_layout(author, &doc, true, Uuid::new_v4())?;
    let pinned = author;
    let rows = |store: &Store, context| -> Result<Vec<(ObjectRef, String)>, Error> {
        let page = store
            .display_entry(context, entry)?
            .ok_or(Error::NotFound)?;
        let DisplayContent::Stack(children) = page.content else {
            panic!("expected layout")
        };
        let DisplayContent::Stack(private) = &children[1].content else {
            panic!("expected private stack")
        };
        assert_eq!(private[0].id, query_id);
        let DisplayContent::QueryRows { records, .. } = &private[0].content else {
            panic!("expected bound query")
        };
        Ok(records.clone())
    };
    assert_eq!(
        rows(&store, pinned)?,
        vec![(id(&world, 2), "Beatrice Thorne".into())]
    );
    let revision = store.save_named_label_query(SaveNamedLabelQuery {
        world_id: world.id,
        expected: author.revision,
        query_id: Some(saved.id),
        name: "Revised definition",
        spec: &query(QueryFilter::ExactLabel("Alice Vale".into())),
        command_id: Uuid::new_v4(),
        scope: Scope::Author,
    })?;
    author.revision = revision.recorded_at;
    assert_eq!(rows(&store, author)?, rows(&store, pinned)?);
    author.revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: author.revision,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Correct person label".into(),
        scope: Scope::Author,
        changes: vec![Change::Correct(
            id(&world, 2),
            Record {
                label: "Beatrice Florin".into(),
                valid: ValidInterval::default(),
                source: None,
                data: ObjectData::Person,
            },
        )],
    })?;
    assert!(rows(&store, author)?.is_empty());
    assert_eq!(rows(&store, pinned)?.len(), 1);
    let public = ViewContext {
        scope: Scope::Public,
        ..author
    };
    let page = store.display_entry(public, entry)?.unwrap();
    assert!(matches!(page.content, DisplayContent::Stack(children) if children.len() == 1));
    assert!(matches!(
        binding.resolve(&store, public),
        Err(Error::CapabilityDenied)
    ));
    drop(store);
    let store = Store::open(&path)?;
    assert_eq!(store.entry_layout(pinned, entry)?, Some((doc, true)));
    assert_eq!(rows(&store, pinned)?.len(), 1);
    assert!(rows(&store, author)?.is_empty());
    Ok(())
}

#[test]
fn caption_search_reauthorizes_destinations_without_resaving_sources() -> Result<(), Error> {
    use atlas_store::{layout::ObjectLink, query::QueryControl};

    let dir = tempfile::tempdir()?;
    let path = dir.path().join("captions.sqlite");
    let mut store = Store::open(&path)?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let alice = id(&world, 1);
    let future = id(&world, 7);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let caption = |label: &str| {
        block(
            ClaimAudience::Public,
            ElementContent::ObjectLink(ObjectLink {
                target: future,
                label: label.into(),
            }),
        )
    };
    let visible = caption("Étoile \"quoted\"");
    let element = visible.id;
    let mut source = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: alice,
        root: block(
            ClaimAudience::Public,
            ElementContent::Stack(vec![
                visible,
                block(
                    ClaimAudience::Author,
                    ElementContent::Stack(vec![caption("Secret étoile")]),
                ),
            ]),
        ),
    };
    let target = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: future,
        root: block(ClaimAudience::Public, ElementContent::Stack(vec![])),
    };
    let public = |view| ViewContext {
        scope: Scope::Public,
        ..view
    };
    let find = |store: &Store, context, term, limit| {
        let control = QueryControl::new(std::time::Duration::from_secs(10))?;
        store.search_public_pages(public(context), term, limit, &control)
    };
    author.revision = store.save_entry_layout(author, &source, true, Uuid::new_v4())?;
    assert!(find(&store, author, "étoile", 1)?.matches.is_empty());
    assert!(
        store
            .search_entry_text(public(author), alice, "étoile", 1)?
            .unwrap()
            .matches
            .is_empty()
    );
    author.revision = store.save_entry_layout(author, &target, true, Uuid::new_v4())?;
    let released = author;
    let found = find(&store, author, "ÉTOILE \"quoted\"", 1)?;
    assert_eq!(found.matches.len(), 1);
    assert_eq!(found.matches[0].entry, alice);
    assert_eq!(found.matches[0].element, element);
    assert_eq!(found.matches[0].text, "Étoile \"quoted\"");
    assert!(!found.truncated);
    assert_eq!(find(&store, author, "ét", 1)?, found);
    assert_eq!(find(&store, author, "étoile", 1)?, found);
    assert!(find(&store, author, "secret", 1)?.matches.is_empty());
    let local = store
        .search_entry_text(public(author), alice, "étoile", 1)?
        .unwrap();
    assert_eq!(local.matches, vec![(element, "Étoile \"quoted\"".into())]);
    assert!(!local.truncated);
    let early = ViewContext {
        day: WorldDay(11),
        ..author
    };
    assert!(find(&store, early, "étoile", 1)?.matches.is_empty());
    assert!(
        store
            .search_entry_text(author, alice, "secret", 1)?
            .unwrap()
            .matches
            .len()
            == 1
    );
    // Simulate an actual schema-26 database without any caption index.
    drop(store);
    let conn = rusqlite::Connection::open(&path)?;
    conn.execute_batch(
        "DROP TRIGGER entry_caption_ai;
         DROP TRIGGER entry_caption_ad;
         DROP TABLE entry_caption_fts;
         DROP TABLE entry_caption_candidates;
         PRAGMA user_version=26;",
    )?;
    drop(conn);
    let mut store = Store::open(&path)?;
    assert_eq!(find(&store, author, "étoile", 1)?, found);
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(find(&store, author, "étoile", 1)?, found);
    author.revision = store.save_entry_layout(author, &target, false, Uuid::new_v4())?;
    assert!(find(&store, author, "étoile", 1)?.matches.is_empty());
    assert!(matches!(
        find(&store, released, "étoile", 1),
        Err(Error::InvalidRevision)
    ));
    author.revision = store.save_entry_layout(author, &target, true, Uuid::new_v4())?;
    assert_eq!(find(&store, author, "étoile", 1)?, found);
    if let ElementContent::Stack(children) = &mut source.root.content {
        children.clear();
    }
    author.revision = store.save_entry_layout(author, &source, true, Uuid::new_v4())?;
    assert!(find(&store, author, "étoile", 1)?.matches.is_empty());
    Ok(())
}

#[test]
fn authored_links_follow_publication_and_historical_backlinks() -> Result<(), Error> {
    use atlas_store::{layout::ObjectLink, query::QueryControl};

    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let alice = id(&world, 1);
    let beatrice = id(&world, 2);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let link = |target, label: &str| {
        ElementContent::ObjectLink(ObjectLink {
            target,
            label: label.into(),
        })
    };
    let public = |view| ViewContext {
        scope: Scope::Public,
        ..view
    };
    let target = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: beatrice,
        root: block(
            ClaimAudience::Public,
            ElementContent::Stack(vec![block(
                ClaimAudience::Public,
                link(alice, "Return to Alice"),
            )]),
        ),
    };
    let visible = block(ClaimAudience::Public, link(beatrice, "Read Beatrice"));
    let element = visible.id;
    let mut source = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: alice,
        root: block(
            ClaimAudience::Public,
            ElementContent::Stack(vec![
                visible,
                block(
                    ClaimAudience::Author,
                    ElementContent::Stack(vec![block(
                        ClaimAudience::Public,
                        link(beatrice, "Hidden reference"),
                    )]),
                ),
            ]),
        ),
    };
    let control = QueryControl::new(std::time::Duration::from_secs(10))?;
    let find =
        |store: &Store, context, limit| store.entry_backlinks(context, beatrice, limit, &control);
    let mut invalid = source.clone();
    if let ElementContent::Stack(children) = &mut invalid.root.content {
        children.push(block(
            ClaimAudience::Author,
            link(id(&world, 999_999), "Unknown identity"),
        ));
    }
    assert!(matches!(
        store.save_entry_layout(author, &invalid, true, Uuid::new_v4()),
        Err(Error::NotFound)
    ));
    assert!(store.entry_layout(author, alice)?.is_none());
    author.revision = store.save_entry_layout(author, &source, true, Uuid::new_v4())?;
    assert!(find(&store, public(author), 1)?.matches.is_empty());
    let display = store.display_entry(public(author), alice)?.unwrap();
    assert!(matches!(display.content, DisplayContent::Stack(children) if children.is_empty()));
    author.revision = store.save_entry_layout(author, &target, true, Uuid::new_v4())?;
    let pinned = author;
    let found = find(&store, public(author), 1)?;
    assert_eq!(found.matches.len(), 1);
    assert_eq!(found.matches[0].element, element);
    assert_eq!(found.matches[0].source, alice);
    assert!(!found.truncated);
    let private = find(&store, author, 1)?;
    assert!(private.truncated);
    assert_eq!(private.matches, found.matches);
    let page = store.display_entry(public(author), alice)?.unwrap();
    let DisplayContent::Stack(children) = page.content else {
        panic!("expected stack")
    };
    assert_eq!(children.len(), 1);
    assert_eq!(
        children[0].content,
        DisplayContent::ObjectLink(ObjectLink {
            target: beatrice,
            label: "Read Beatrice".into(),
        })
    );
    assert!(store.display_entry(public(author), beatrice)?.is_some());
    author.revision = store.save_entry_layout(author, &target, false, Uuid::new_v4())?;
    assert!(find(&store, public(author), 50)?.matches.is_empty());
    assert_eq!(find(&store, author, 50)?.matches.len(), 2);
    assert!(matches!(
        find(&store, public(pinned), 50),
        Err(Error::InvalidRevision)
    ));
    if let ElementContent::Stack(children) = &mut source.root.content {
        children.clear();
    }
    author.revision = store.save_entry_layout(author, &source, true, Uuid::new_v4())?;
    assert!(find(&store, author, 50)?.matches.is_empty());
    let historical = find(&store, pinned, 50)?;
    assert_eq!(historical.matches.len(), 2);
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(find(&store, pinned, 50)?, historical);
    Ok(())
}

#[test]
fn grid_and_panel_preserve_visibility_search_backlinks_and_moved_bindings() -> Result<(), Error> {
    use atlas_store::{layout::ObjectLink, query::QueryControl};

    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let source = id(&world, 1);
    let target = id(&world, 2);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let destination = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: target,
        root: block(ClaimAudience::Public, ElementContent::Stack(vec![])),
    };
    author.revision = store.save_entry_layout(author, &destination, true, Uuid::new_v4())?;
    let link = block(
        ClaimAudience::Public,
        ElementContent::ObjectLink(ObjectLink {
            target,
            label: "Council caption".into(),
        }),
    );
    let link_id = link.id;
    let query_block = block(
        ClaimAudience::Public,
        ElementContent::Query(query(QueryFilter::ExactLabel("Beatrice Thorne".into()))),
    );
    let query_id = query_block.id;
    let grid = block(
        ClaimAudience::Public,
        ElementContent::Grid {
            columns: 2,
            children: vec![
                block(
                    ClaimAudience::Public,
                    ElementContent::Text("Council biography".into()),
                ),
                link,
            ],
        },
    );
    let grid_id = grid.id;
    let mut document = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: source,
        root: block(
            ClaimAudience::Public,
            ElementContent::Stack(vec![
                grid,
                block(
                    ClaimAudience::Author,
                    ElementContent::Panel(vec![
                        query_block,
                        block(
                            ClaimAudience::Public,
                            ElementContent::Text("Secret council testimony".into()),
                        ),
                    ]),
                ),
            ]),
        ),
    };
    let mut invalid = document.clone();
    if let ElementContent::Stack(children) = &mut invalid.root.content
        && let ElementContent::Grid { columns, .. } = &mut children[0].content
    {
        *columns = 5;
    }
    assert!(matches!(invalid.validate(), Err(Error::InvalidInput(_))));
    let public = |view| ViewContext {
        scope: Scope::Public,
        ..view
    };
    let before = document.compile(&store, public(author))?;
    document.reparent(query_id, grid_id)?;
    assert_eq!(document.compile(&store, public(author))?, before);
    let projected = document.compile(&store, author)?.unwrap();
    let DisplayContent::Stack(children) = projected.content else {
        panic!("expected stack")
    };
    let DisplayContent::Grid { columns, children } = &children[0].content else {
        panic!("expected grid")
    };
    assert_eq!(*columns, 2);
    assert_eq!(children[2].id, query_id);
    assert_eq!(
        children[2].content,
        DisplayContent::QueryRows {
            records: vec![(target, "Beatrice Thorne".into())],
            truncated: false,
        }
    );
    author.revision = store.save_entry_layout(author, &document, true, Uuid::new_v4())?;
    let control = QueryControl::new(std::time::Duration::from_secs(10))?;
    let found = store.search_public_pages(public(author), "council", 2, &control)?;
    assert_eq!(found.matches.len(), 2);
    assert_eq!(found.matches[1].element, link_id);
    assert!(!found.truncated);
    assert!(
        store
            .search_public_pages(public(author), "secret", 1, &control)?
            .matches
            .is_empty()
    );
    let backlinks = store.entry_backlinks(public(author), target, 1, &control)?;
    assert_eq!(backlinks.matches.len(), 1);
    assert_eq!(backlinks.matches[0].element, link_id);
    store.rebuild_search_index(Scope::Author)?;
    assert_eq!(
        store.search_public_pages(public(author), "council", 2, &control)?,
        found
    );
    Ok(())
}

#[test]
fn versioned_template_keeps_queries_private_and_persists_independent_instances() -> Result<(), Error>
{
    use atlas_store::layout::TemplateKind;

    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let template = block(
        ClaimAudience::Author,
        ElementContent::Template {
            key: TemplateKind::Character,
            version: 1,
            children: vec![
                block(
                    ClaimAudience::Public,
                    ElementContent::Text("Private template biography".into()),
                ),
                block(
                    ClaimAudience::Public,
                    ElementContent::Query(query(QueryFilter::ExactLabel("Alice Vale".into()))),
                ),
            ],
        },
    );
    let mut document = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: id(&world, 1),
        root: block(ClaimAudience::Public, ElementContent::Stack(vec![template])),
    };
    let mut invalid = document.clone();
    if let ElementContent::Stack(children) = &mut invalid.root.content
        && let ElementContent::Template { version, .. } = &mut children[0].content
    {
        *version = 2;
    }
    assert!(matches!(invalid.validate(), Err(Error::InvalidInput(_))));
    if let ElementContent::Stack(children) = &mut invalid.root.content
        && let ElementContent::Template { version, .. } = &mut children[0].content
    {
        *version = 1;
        children[0].audience = ClaimAudience::Public;
    }
    assert!(matches!(
        store.save_entry_layout(author, &invalid, true, Uuid::new_v4()),
        Err(Error::CapabilityDenied)
    ));
    author.revision = store.save_entry_layout(author, &document, true, Uuid::new_v4())?;
    let public = ViewContext {
        scope: Scope::Public,
        ..author
    };
    let page = store.display_entry(public, document.entry)?.unwrap();
    assert_eq!(page.content, DisplayContent::Stack(vec![]));
    assert_eq!(
        store.entry_layout(author, document.entry)?,
        Some((document.clone(), true))
    );
    let prior = author;
    if let ElementContent::Stack(children) = &mut document.root.content
        && let ElementContent::Template { children, .. } = &mut children[0].content
    {
        children[0].content = ElementContent::Text("Revised private biography".into());
    }
    author.revision = store.save_entry_layout(author, &document, true, Uuid::new_v4())?;
    assert_ne!(
        store.entry_layout(prior, document.entry)?,
        store.entry_layout(author, document.entry)?
    );
    Ok(())
}

#[test]
fn family_canvas_component_uses_accepted_parentage_at_selected_day_and_revision()
-> Result<(), Error> {
    use atlas_store::genealogy::RecordParentage;

    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let person = id(&world, 2);
    let mut author = view(&world, 0);
    let link_id = store.record_parentage(
        author,
        RecordParentage {
            parent: id(&world, 1),
            child: person,
            kind: ParentageKind::Biological,
            starts_at: WorldDay(10),
            ends_before: Some(WorldDay(20)),
            command_id: Uuid::new_v4(),
            rationale: "Accepted household register",
        },
    )?;
    author.revision = AuthorRevision(2);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let document = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: person,
        root: block(
            ClaimAudience::Public,
            ElementContent::Stack(vec![block(
                ClaimAudience::Author,
                ElementContent::FamilyTable(person),
            )]),
        ),
    };
    let at = |revision, day| ViewContext {
        revision,
        day: WorldDay(day),
        ..author
    };
    let before = document.compile(&store, at(author.revision, 9))?.unwrap();
    let DisplayContent::Stack(before) = before.content else {
        panic!("expected stack")
    };
    assert_eq!(
        before[0].content,
        DisplayContent::QueryRows {
            records: vec![],
            truncated: false,
        }
    );
    let revision = store.save_entry_layout(author, &document, true, Uuid::new_v4())?;
    let page = store.display_entry(at(revision, 10), person)?.unwrap();
    let DisplayContent::Stack(rows) = page.content else {
        panic!("expected stack")
    };
    assert_eq!(
        rows[0].content,
        DisplayContent::QueryRows {
            records: vec![(id(&world, 1), "Alice Vale · biological parent".into())],
            truncated: false,
        }
    );
    let public = ViewContext {
        scope: Scope::Public,
        ..at(revision, 10)
    };
    assert_eq!(
        store.display_entry(public, person)?.unwrap().content,
        DisplayContent::Stack(vec![])
    );
    store.withdraw_parentage(
        at(revision, 10),
        person,
        link_id,
        Uuid::new_v4(),
        "Register corrected",
    )?;
    let corrected = AuthorRevision(revision.0 + 1);
    let after = store.display_entry(at(corrected, 10), person)?.unwrap();
    let DisplayContent::Stack(rows) = after.content else {
        panic!("expected stack")
    };
    assert_eq!(
        rows[0].content,
        DisplayContent::QueryRows {
            records: vec![],
            truncated: false,
        }
    );
    assert_ne!(
        store.display_entry(at(revision, 10), person)?,
        store.display_entry(at(corrected, 10), person)?
    );
    Ok(())
}

#[test]
fn institutional_network_ghost_and_public_projection() -> Result<(), Error> {
    use atlas_store::{
        layout::ObjectLink,
        network::{IdentityKind, NetworkInput, NetworkKind},
        query::ObjectKindFilter,
    };

    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let group =
        store.create_identity(author, IdentityKind::Organization, "Guild", Uuid::new_v4())?;
    author.revision = AuthorRevision(2);
    let kind = query(QueryFilter::Kind(ObjectKindFilter::Organization));
    assert_eq!(store.execute_query(&kind, author)?.rows[0].object, group);
    let member = NetworkInput {
        subject: id(&world, 1),
        target: group,
        source: id(&world, 9),
        kind: NetworkKind::Membership,
        role: "Treasurer".into(),
        starts_at: 13,
        ends_before: Some(20),
    };
    let membership = store.record_network_fact(author, &member, Uuid::new_v4())?;
    author.revision = AuthorRevision(3);
    assert_eq!(store.network_facts(author, group)?[0].input, member);
    assert!(matches!(
        store.record_network_fact(author, &member, Uuid::new_v4()),
        Err(Error::InvalidInput(_))
    ));
    let cause = NetworkInput {
        subject: id(&world, 7),
        target: id(&world, 11),
        source: id(&world, 9),
        kind: NetworkKind::ConfirmedCause,
        role: "Recorded adjournment".into(),
        starts_at: 13,
        ends_before: None,
    };
    store.record_network_fact(author, &cause, Uuid::new_v4())?;
    author.revision = AuthorRevision(4);
    assert_eq!(
        store.persisted_impact(author, cause.subject, 4, 10)?.events,
        vec![cause.target]
    );
    let reversed = NetworkInput {
        subject: cause.target,
        target: cause.subject,
        ..cause
    };
    assert!(matches!(
        store.record_network_fact(author, &reversed, Uuid::new_v4()),
        Err(Error::InvalidInput(_))
    ));
    let claim = ObjectRef::new(world.id);
    author.revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: author.revision,
        id: Uuid::new_v4(),
        author: "Fixture author".into(),
        rationale: "Record claim source".into(),
        scope: Scope::Author,
        changes: vec![Change::Create(
            claim,
            Record {
                label: "Minute excerpt".into(),
                valid: ValidInterval::default(),
                data: ObjectData::Claim {
                    subject: group,
                    predicate: "mention".into(),
                    statement: "New Society convened.".into(),
                    audience: ClaimAudience::Author,
                },
                source: Some(id(&world, 9)),
            },
        )],
    })?;
    let proposal = store.propose_ghost(
        author,
        claim,
        (0, 11),
        IdentityKind::Organization,
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(6);
    let prior = author;
    assert_eq!(store.execute_query(&kind, author)?.rows.len(), 1);
    let promoted = store
        .review_ghost(author, proposal, true, Uuid::new_v4())?
        .unwrap();
    author.revision = AuthorRevision(7);
    assert_eq!(store.execute_query(&kind, author)?.rows.len(), 2);
    assert!(!store.ghost_proposals(prior)?[0].reviewed);
    assert_eq!(store.ghost_proposals(author)?[0].promoted, Some(promoted));
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let document = |entry, children| LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry,
        root: block(ClaimAudience::Public, ElementContent::Stack(children)),
    };
    let destination = document(
        group,
        vec![
            block(
                ClaimAudience::Public,
                ElementContent::PublishedReferences(group),
            ),
            block(ClaimAudience::Author, ElementContent::NetworkTable(group)),
        ],
    );
    author.revision = store.save_entry_layout(author, &destination, true, Uuid::new_v4())?;
    let source = document(
        id(&world, 1),
        vec![block(
            ClaimAudience::Public,
            ElementContent::ObjectLink(ObjectLink {
                target: group,
                label: "Guild charter".into(),
            }),
        )],
    );
    author.revision = store.save_entry_layout(author, &source, true, Uuid::new_v4())?;
    let public = |context| ViewContext {
        scope: Scope::Public,
        ..context
    };
    let page = store.display_entry(public(author), group)?.unwrap();
    let DisplayContent::Stack(rows) = page.content else {
        panic!("expected stack")
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].content,
        DisplayContent::ReferenceRows {
            links: vec![ObjectLink {
                target: source.entry,
                label: "Guild charter".into(),
            }],
            truncated: false,
        }
    );
    assert!(matches!(
        store.network_facts(public(author), group),
        Err(Error::CapabilityDenied)
    ));
    author.revision = store.save_entry_layout(author, &source, false, Uuid::new_v4())?;
    let page = store.display_entry(public(author), group)?.unwrap();
    let DisplayContent::Stack(rows) = page.content else {
        panic!("expected stack")
    };
    assert_eq!(
        rows[0].content,
        DisplayContent::ReferenceRows {
            links: vec![],
            truncated: false,
        }
    );
    let old = author;
    store.withdraw_network_fact(author, membership, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    assert!(store.network_facts(author, group)?.is_empty());
    assert_eq!(store.network_facts(old, group)?.len(), 1);
    store.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn conflict_positions_are_dated_sourced_and_distinct_from_membership() -> Result<(), Error> {
    use atlas_store::network::{IdentityKind, NetworkInput, NetworkKind};

    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let faction = store.create_identity(
        author,
        IdentityKind::Faction,
        "Workers Guild",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(2);
    let conflict =
        store.create_identity(author, IdentityKind::Conflict, "Strike", Uuid::new_v4())?;
    author.revision = AuthorRevision(3);
    let position = NetworkInput {
        subject: faction,
        target: conflict,
        source: id(&world, 9),
        kind: NetworkKind::ConflictPosition,
        role: "opposes".into(),
        starts_at: 13,
        ends_before: Some(20),
    };
    store.record_network_fact(author, &position, Uuid::new_v4())?;
    author.revision = AuthorRevision(4);
    let invalid = NetworkInput {
        role: "unrecognized".into(),
        ..position.clone()
    };
    assert!(matches!(
        store.record_network_fact(author, &invalid, Uuid::new_v4()),
        Err(Error::InvalidInput(_))
    ));
    let changed = NetworkInput {
        role: "mediates".into(),
        starts_at: 20,
        ends_before: None,
        ..position
    };
    store.record_network_fact(author, &changed, Uuid::new_v4())?;
    author.revision = AuthorRevision(5);
    assert_eq!(
        store.network_facts(author, conflict)?[0].input.role,
        "opposes"
    );
    author.day = WorldDay(20);
    assert_eq!(
        store.network_facts(author, conflict)?[0].input.role,
        "mediates"
    );
    Ok(())
}

#[test]
fn custom_template_pins_and_rebinds_without_copying_canonical_objects() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let original = id(&world, 1);
    let destination = id(&world, 2);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let page = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: original,
        root: block(
            ClaimAudience::Public,
            ElementContent::Stack(vec![block(
                ClaimAudience::Author,
                ElementContent::FamilyTable(original),
            )]),
        ),
    };
    author.revision = store.save_entry_layout(author, &page, false, Uuid::new_v4())?;
    let first =
        store.save_custom_template(author, None, original, "Family profile", Uuid::new_v4())?;
    author.revision = AuthorRevision(first.revision);
    let instance =
        store.instantiate_custom_template(author, first.id, first.revision, destination)?;
    let ElementContent::CustomTemplate {
        definition_revision,
        children,
        ..
    } = &instance.content
    else {
        panic!("expected custom template");
    };
    assert_eq!(*definition_revision, first.revision);
    assert_ne!(children[0].id, page.root.id);
    let ElementContent::Stack(children) = &children[0].content else {
        panic!("expected copied root");
    };
    assert_eq!(
        children[0].content,
        ElementContent::FamilyTable(destination)
    );
    let new_page = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: destination,
        root: block(ClaimAudience::Public, ElementContent::Stack(vec![instance])),
    };
    author.revision = store.save_entry_layout(author, &new_page, false, Uuid::new_v4())?;
    let updated = store.save_custom_template(
        author,
        Some(first.id),
        destination,
        "Family profile revised",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(updated.revision);
    assert_eq!(store.custom_templates(author)?[0], updated);
    assert_eq!(
        store.entry_layout(author, destination)?,
        Some((new_page, false))
    );
    assert!(
        store
            .instantiate_custom_template(author, first.id, first.revision, original)
            .is_ok()
    );
    assert!(matches!(
        store.custom_templates(ViewContext {
            scope: Scope::Public,
            ..author
        }),
        Err(Error::CapabilityDenied)
    ));
    Ok(())
}

#[test]
fn fork_metadata_and_founder_ancestry_survive_parent_corrections_and_archive() -> Result<(), Error>
{
    use atlas_store::{
        archive::restore_archive,
        genealogy::RecordParentage,
        inheritance::{FOUNDER_MODEL_VERSION, MODEL_LENGTH},
        network::{IdentityKind, NetworkInput, NetworkKind},
    };
    let dir = tempfile::tempdir()?;
    let mut store = Store::open(dir.path().join("source.sqlite"))?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let person = id(&world, 2);
    for (parent, child) in [(1, 2), (12, 2), (13, 1), (14, 1)] {
        store.record_parentage(
            author,
            RecordParentage {
                parent: id(&world, parent),
                child: id(&world, child),
                kind: ParentageKind::Biological,
                starts_at: WorldDay(0),
                ends_before: None,
                rationale: "Accepted historical register",
                command_id: Uuid::new_v4(),
            },
        )?;
        author.revision = AuthorRevision(author.revision.0 + 1);
    }
    let group = store.create_identity(
        author,
        IdentityKind::FamilyHouse,
        "House Vale",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let member = NetworkInput {
        subject: person,
        target: group,
        source: id(&world, 9),
        kind: NetworkKind::Membership,
        role: "member".into(),
        starts_at: 13,
        ends_before: None,
    };
    let fact = store.record_network_fact(author, &member, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let future = NetworkInput {
        starts_at: 20,
        role: "later member".into(),
        subject: id(&world, 1),
        ..member.clone()
    };
    store.record_network_fact(author, &future, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let genome = store.founder_genome(author, person, 72)?;
    assert_eq!(genome.model, FOUNDER_MODEL_VERSION);
    assert_eq!(genome, store.founder_genome(author, person, 72)?);
    for segments in &genome.homologues {
        assert_eq!(segments[0].start, 0);
        assert_eq!(segments.last().unwrap().end, MODEL_LENGTH);
        assert!(
            segments
                .windows(2)
                .all(|window| window[0].end == window[1].start)
        );
    }
    assert!(
        genome
            .homologues
            .iter()
            .flatten()
            .any(|s| s.allele.founder == id(&world, 13) || s.allele.founder == id(&world, 14))
    );
    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: author.day,
        expected: author.revision,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Alternative history".into(),
        scope: Scope::Author,
    })?;
    author.revision = fork.created_revision;
    let parent = store
        .family_links(author, person)?
        .into_iter()
        .find(|link| link.parent == id(&world, 1))
        .unwrap();
    store.withdraw_parentage(
        author,
        person,
        parent.id,
        Uuid::new_v4(),
        "Register corrected",
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    store.withdraw_network_fact(author, fact, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let branch = ViewContext {
        branch: fork.id,
        day: WorldDay(21),
        ..author
    };
    assert_eq!(store.network_facts(branch, group)?.len(), 1);
    assert_eq!(store.network_facts(branch, group)?[0].id, fact);
    assert_ne!(store.founder_genome(author, person, 72)?, genome);
    assert_eq!(store.founder_genome(branch, person, 72)?, genome);
    assert!(matches!(
        store.record_network_fact(branch, &member, Uuid::new_v4()),
        Err(Error::InvalidInput(_))
    ));
    let backup = dir.path().join("snapshot.sqlite");
    let restored = dir.path().join("restored.sqlite");
    store.backup_to(&backup, Scope::Author)?;
    let _ = restore_archive(&backup, &restored, Scope::Author)?;
    let reopened = Store::open(&restored)?;
    assert_eq!(reopened.founder_genome(branch, person, 72)?, genome);
    assert_eq!(reopened.network_facts(branch, group)?[0].id, fact);
    reopened.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn ghost_suggestions_resolve_existing_identity_and_public_dynamic_search_revokes()
-> Result<(), Error> {
    use atlas_store::{network::IdentityKind, query::QueryControl};
    use std::time::Duration;

    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let event = id(&world, 6);
    let claim = ObjectRef::new(world.id);
    let secret = ObjectRef::new(world.id);
    let record = |statement: &str, audience| Record {
        label: "Recorded account".into(),
        valid: ValidInterval::default(),
        data: ObjectData::Claim {
            subject: event,
            predicate: "account".into(),
            statement: statement.into(),
            audience,
        },
        source: Some(id(&world, 9)),
    };
    author.revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: author.revision,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Record accounts".into(),
        scope: Scope::Author,
        changes: vec![
            Change::Create(
                claim,
                record("Alice Vale encountered New Society.", ClaimAudience::Public),
            ),
            Change::Create(
                secret,
                record(
                    "Private testimony about Secret Circle.",
                    ClaimAudience::Author,
                ),
            ),
        ],
    })?;
    let candidates = store.ghost_candidates(author, claim)?;
    let alice = candidates
        .iter()
        .find(|item| item.label == "Alice Vale")
        .unwrap();
    assert_eq!(alice.existing.len(), 2);
    assert!(
        candidates
            .iter()
            .any(|item| item.label == "New Society" && item.existing.is_empty())
    );
    let proposal = store.propose_ghost(
        author,
        claim,
        (alice.start_byte, alice.end_byte),
        IdentityKind::Person,
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let previous = author;
    assert!(matches!(
        store.resolve_ghost_to_existing(author, proposal, id(&world, 2), Uuid::new_v4()),
        Err(Error::InvalidInput(_))
    ));
    store.resolve_ghost_to_existing(author, proposal, id(&world, 1), Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    assert_eq!(
        store.ghost_proposals(author)?[0].promoted,
        Some(id(&world, 1))
    );
    assert!(!store.ghost_proposals(previous)?[0].reviewed);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let page = |entry, children| LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry,
        root: block(ClaimAudience::Public, ElementContent::Stack(children)),
    };
    let target = page(
        event,
        vec![block(
            ClaimAudience::Public,
            ElementContent::Text("Released event".into()),
        )],
    );
    author.revision = store.save_entry_layout(author, &target, true, Uuid::new_v4())?;
    let entry = page(
        id(&world, 1),
        vec![block(
            ClaimAudience::Public,
            ElementContent::PublishedStatements(event),
        )],
    );
    author.revision = store.save_entry_layout(author, &entry, true, Uuid::new_v4())?;
    let read = |context| ViewContext {
        scope: Scope::Public,
        ..context
    };
    let control = QueryControl::new(Duration::from_secs(10))?;
    assert_eq!(
        store
            .search_public_pages(read(author), "New Society", 10, &control)?
            .matches[0]
            .entry,
        entry.entry
    );
    let control = QueryControl::new(Duration::from_secs(10))?;
    assert!(
        store
            .search_public_pages(read(author), "Secret Circle", 10, &control)?
            .matches
            .is_empty()
    );
    author.revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: author.revision,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Withdraw statement audience".into(),
        scope: Scope::Author,
        changes: vec![Change::Correct(
            claim,
            record("Alice Vale encountered New Society.", ClaimAudience::Author),
        )],
    })?;
    let control = QueryControl::new(Duration::from_secs(10))?;
    assert!(
        store
            .search_public_pages(read(author), "New Society", 10, &control)?
            .matches
            .is_empty()
    );
    Ok(())
}

#[test]
fn title_rules_offices_and_issue_positions_have_typed_dated_constraints() -> Result<(), Error> {
    use atlas_store::network::{IdentityKind, NetworkInput, NetworkKind};
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let mut make = |kind, label| -> Result<ObjectRef, Error> {
        let item = store.create_identity(author, kind, label, Uuid::new_v4())?;
        author.revision = AuthorRevision(author.revision.0 + 1);
        Ok(item)
    };
    let title = make(IdentityKind::Title, "Duchess")?;
    let rule = make(IdentityKind::SuccessionRule, "House charter")?;
    let faction = make(IdentityKind::Faction, "Council")?;
    let issue = make(IdentityKind::ConflictIssue, "Wage dispute")?;
    let holding = NetworkInput {
        subject: id(&world, 1),
        target: title,
        source: id(&world, 9),
        kind: NetworkKind::TitleHolding,
        role: "holder".into(),
        starts_at: 13,
        ends_before: Some(20),
    };
    store.record_network_fact(author, &holding, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let duplicate = NetworkInput {
        subject: id(&world, 2),
        ..holding.clone()
    };
    assert!(matches!(
        store.record_network_fact(author, &duplicate, Uuid::new_v4()),
        Err(Error::InvalidInput(_))
    ));
    let next = NetworkInput {
        starts_at: 20,
        ends_before: None,
        ..duplicate
    };
    store.record_network_fact(author, &next, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let charter = NetworkInput {
        subject: rule,
        target: title,
        source: id(&world, 9),
        kind: NetworkKind::RuleOf,
        role: "absolute_primogeniture".into(),
        starts_at: 13,
        ends_before: None,
    };
    store.record_network_fact(author, &charter, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let stance = NetworkInput {
        subject: faction,
        target: issue,
        source: id(&world, 9),
        kind: NetworkKind::ConflictPosition,
        role: "opposes".into(),
        starts_at: 13,
        ends_before: None,
    };
    store.record_network_fact(author, &stance, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    assert_eq!(store.network_facts(author, issue)?[0].input, stance);
    assert!(
        store
            .network_facts(author, title)?
            .iter()
            .any(|f| f.input == holding)
    );
    author.day = WorldDay(20);
    assert!(
        store
            .network_facts(author, title)?
            .iter()
            .any(|f| f.input == next)
    );
    Ok(())
}

#[test]
fn branch_local_network_and_parentage_corrections_never_change_parent_or_sibling()
-> Result<(), Error> {
    use atlas_store::{
        genealogy::RecordParentage,
        network::{IdentityKind, NetworkInput, NetworkKind},
    };
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let house = store.create_identity(
        author,
        IdentityKind::FamilyHouse,
        "House Vale",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let person = id(&world, 2);
    let fact = NetworkInput {
        subject: person,
        target: house,
        source: id(&world, 9),
        kind: NetworkKind::Membership,
        role: "member".into(),
        starts_at: 13,
        ends_before: None,
    };
    let member = store.record_network_fact(author, &fact, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let parent = store.record_parentage(
        author,
        RecordParentage {
            parent: id(&world, 1),
            child: person,
            kind: ParentageKind::Biological,
            starts_at: WorldDay(0),
            ends_before: None,
            command_id: Uuid::new_v4(),
            rationale: "Original register",
        },
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let fork = |store: &mut Store, context: ViewContext| {
        store.fork(&ForkCommand {
            world: context.world,
            parent: context.branch,
            day: context.day,
            expected: context.revision,
            id: Uuid::new_v4(),
            author: "Fixture".into(),
            rationale: "Divergence".into(),
            scope: Scope::Author,
        })
    };
    let child = fork(&mut store, author)?;
    author.revision = child.created_revision;
    let sibling = fork(&mut store, author)?;
    author.revision = sibling.created_revision;
    let mut branch = ViewContext {
        branch: child.id,
        ..author
    };
    let prior = branch;
    let member_ref = atlas_store::semantic::FactRef {
        world: world.id,
        family: atlas_store::semantic::FactFamily::Network,
        id: member,
    };
    let parent_ref = atlas_store::semantic::FactRef {
        world: world.id,
        family: atlas_store::semantic::FactFamily::Parentage,
        id: parent,
    };
    let original = store.resolve_semantic_fact(member_ref, prior)?.unwrap();
    assert_eq!(original.source_branch, world.main);
    assert_eq!(
        original.data,
        atlas_store::semantic::SemanticFactData::Network(fact.clone())
    );
    assert!(store.resolve_semantic_fact(parent_ref, prior)?.is_some());
    assert!(
        store
            .resolve_semantic_fact(
                atlas_store::semantic::FactRef {
                    family: atlas_store::semantic::FactFamily::Parentage,
                    ..member_ref
                },
                prior,
            )?
            .is_none()
    );
    assert!(matches!(
        store.resolve_semantic_fact(
            member_ref,
            ViewContext {
                scope: Scope::Public,
                ..prior
            }
        ),
        Err(Error::CapabilityDenied)
    ));
    store.withdraw_network_fact(branch, member, Uuid::new_v4())?;
    branch.revision = AuthorRevision(branch.revision.0 + 1);
    store.withdraw_parentage(
        branch,
        person,
        parent,
        Uuid::new_v4(),
        "Alternative register",
    )?;
    branch.revision = AuthorRevision(branch.revision.0 + 1);
    let changed = NetworkInput {
        role: "exile".into(),
        ..fact.clone()
    };
    store.record_network_fact(branch, &changed, Uuid::new_v4())?;
    branch.revision = AuthorRevision(branch.revision.0 + 1);
    store.record_parentage(
        branch,
        RecordParentage {
            parent: id(&world, 12),
            child: person,
            kind: ParentageKind::Biological,
            starts_at: WorldDay(13),
            ends_before: None,
            command_id: Uuid::new_v4(),
            rationale: "Alternative register",
        },
    )?;
    branch.revision = AuthorRevision(branch.revision.0 + 1);
    assert_eq!(store.network_facts(branch, house)?[0].input.role, "exile");
    assert_eq!(store.network_facts(prior, house)?[0].input.role, "member");
    assert_eq!(
        store.family_links(branch, person)?[0].parent,
        id(&world, 12)
    );
    let main = ViewContext {
        revision: branch.revision,
        ..author
    };
    let sibling = ViewContext {
        branch: sibling.id,
        ..main
    };
    assert!(store.resolve_semantic_fact(member_ref, branch)?.is_none());
    assert!(store.resolve_semantic_fact(parent_ref, branch)?.is_none());
    assert_eq!(
        store.resolve_semantic_fact(member_ref, prior)?,
        Some(original.clone())
    );
    for context in [main, sibling] {
        assert_eq!(
            store.resolve_semantic_fact(member_ref, context)?,
            Some(original.clone())
        );
        assert!(store.resolve_semantic_fact(parent_ref, context)?.is_some());
        assert_eq!(store.network_facts(context, house)?[0].id, member);
        assert_eq!(store.family_links(context, person)?[0].id, parent);
    }
    assert_eq!(
        store.family_links(
            ViewContext {
                day: WorldDay(12),
                ..branch
            },
            person
        )?[0]
            .id,
        parent
    );
    assert!(matches!(
        store.record_network_fact(
            ViewContext {
                day: WorldDay(12),
                ..branch
            },
            &fact,
            Uuid::new_v4()
        ),
        Err(Error::BeforeFork)
    ));
    let grandchild = fork(&mut store, branch)?;
    assert_eq!(
        store.network_facts(
            ViewContext {
                branch: grandchild.id,
                revision: grandchild.created_revision,
                ..branch
            },
            house
        )?[0]
            .input
            .role,
        "exile"
    );
    store.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn designated_succession_and_reversible_identity_redirect_are_explicit() -> Result<(), Error> {
    use atlas_store::network::{IdentityKind, NetworkInput, NetworkKind};
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let title = store.create_identity(author, IdentityKind::Title, "Crown", Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let rule = store.create_identity(
        author,
        IdentityKind::SuccessionRule,
        "Charter",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let make = |subject, target, kind, role: &str| NetworkInput {
        subject,
        target,
        kind,
        role: role.into(),
        source: id(&world, 9),
        starts_at: 13,
        ends_before: None,
    };
    store.record_network_fact(
        author,
        &make(rule, title, NetworkKind::RuleOf, "designated_heir"),
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    assert!(!store.succession_preview(author, title)?.eligible);
    store.record_network_fact(
        author,
        &make(
            id(&world, 2),
            title,
            NetworkKind::HeirDesignation,
            "charter nominee",
        ),
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    assert!(store.succession_preview(author, title)?.eligible);
    store.enact_designated_succession(author, title, id(&world, 9), Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    assert_eq!(
        store.succession_preview(author, title)?.incumbent,
        Some(id(&world, 2))
    );
    assert!(!store.succession_preview(author, title)?.eligible);
    let duplicate = id(&world, 12);
    let target = id(&world, 1);
    let link = store.record_network_fact(
        author,
        &make(
            duplicate,
            target,
            NetworkKind::IdentityRedirect,
            "reviewed identity",
        ),
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    assert_eq!(
        store.canonical_identity(author, duplicate)?,
        vec![duplicate, target]
    );
    assert!(matches!(
        store.record_network_fact(
            author,
            &make(target, duplicate, NetworkKind::IdentityRedirect, "cycle"),
            Uuid::new_v4()
        ),
        Err(Error::InvalidInput(_))
    ));
    let old = author;
    store.withdraw_network_fact(author, link, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    assert_eq!(
        store.canonical_identity(author, duplicate)?,
        vec![duplicate]
    );
    assert_eq!(
        store.canonical_identity(old, duplicate)?,
        vec![duplicate, target]
    );
    assert!(store.get_object(duplicate, author)?.is_some());
    Ok(())
}

#[test]
fn image_hotspots_authorization_and_asset_archive_roundtrip() -> Result<(), Error> {
    use atlas_store::{
        archive::restore_archive,
        layout::{ImageBlock, ImageHotspot, ObjectLink},
        query::QueryControl,
    };
    use std::time::Duration;
    let dir = tempfile::tempdir()?;
    let mut store = Store::open(dir.path().join("source.sqlite"))?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let mut data = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut data, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[1, 2, 3, 255]).unwrap();
        writer.finish().unwrap();
    }
    assert!(
        store
            .import_png(author, "bad", b"<svg/>", Uuid::new_v4())
            .is_err()
    );
    let image = store.import_png(author, "Private filename", &data, Uuid::new_v4())?;
    author.revision = AuthorRevision(image.revision);
    let block = |audience, content| LayoutElement {
        id: ElementId(Uuid::new_v4()),
        audience,
        content,
    };
    let page = |entry, children| LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry,
        root: block(ClaimAudience::Public, ElementContent::Stack(children)),
    };
    for target in [id(&world, 2), id(&world, 12)] {
        let p = page(
            target,
            vec![block(
                ClaimAudience::Public,
                ElementContent::Text("Released target".into()),
            )],
        );
        author.revision = store.save_entry_layout(author, &p, true, Uuid::new_v4())?;
    }
    let hotspot = |target, label: &str, audience| ImageHotspot {
        id: ElementId(Uuid::new_v4()),
        x: 5000,
        y: 5000,
        audience,
        link: ObjectLink {
            target,
            label: label.into(),
        },
    };
    let document = page(
        id(&world, 1),
        vec![block(
            ClaimAudience::Public,
            ElementContent::Image(ImageBlock {
                asset: image.id,
                alt: "Public painted portrait".into(),
                hotspots: vec![
                    hotspot(id(&world, 2), "Released companion", ClaimAudience::Public),
                    hotspot(id(&world, 12), "Hidden accomplice", ClaimAudience::Author),
                ],
            }),
        )],
    );
    author.revision = store.save_entry_layout(author, &document, true, Uuid::new_v4())?;
    let public = ViewContext {
        scope: Scope::Public,
        ..author
    };
    let projection = store.display_entry(public, document.entry)?.unwrap();
    let encoded = serde_json::to_string(&projection).unwrap();
    assert!(encoded.contains("Released companion"));
    assert!(!encoded.contains("Hidden accomplice"));
    assert!(!encoded.contains("Private filename"));
    assert!(!encoded.contains(&image.id.to_string()));
    assert!(matches!(
        store.media_assets(public),
        Err(Error::CapabilityDenied)
    ));
    let control = QueryControl::new(Duration::from_secs(10))?;
    assert_eq!(
        store
            .search_public_pages(public, "Released companion", 10, &control)?
            .matches
            .len(),
        1
    );
    let control = QueryControl::new(Duration::from_secs(10))?;
    assert!(
        store
            .search_public_pages(public, "Hidden accomplice", 10, &control)?
            .matches
            .is_empty()
    );
    let backup = dir.path().join("backup.sqlite");
    let restored = dir.path().join("restored.sqlite");
    store.backup_to(&backup, Scope::Author)?;
    restore_archive(&backup, &restored, Scope::Author)?;
    let reopened = Store::open(&restored)?;
    assert_eq!(
        reopened.display_entry(public, document.entry)?,
        Some(projection)
    );
    assert_eq!(reopened.media_assets(author)?[0], image);
    Ok(())
}

#[test]
fn chromosome_painter_keeps_contiguous_toy_homologues_and_deterministic_seed() -> Result<(), Error>
{
    use atlas_store::{
        genealogy::RecordParentage,
        inheritance::{CHROMOSOME_MODEL_VERSION, MODEL_LENGTH},
    };
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let person = id(&world, 2);
    for parent in [1, 12] {
        store.record_parentage(
            author,
            RecordParentage {
                parent: id(&world, parent),
                child: person,
                kind: ParentageKind::Biological,
                starts_at: WorldDay(0),
                ends_before: None,
                command_id: Uuid::new_v4(),
                rationale: "Accepted biological link",
            },
        )?;
        author.revision = AuthorRevision(author.revision.0 + 1);
    }
    let painter = store.chromosome_painter(author, person, 72, 8)?;
    assert_eq!(painter.model, CHROMOSOME_MODEL_VERSION);
    assert_eq!(painter.chromosomes.len(), 8);
    assert_eq!(painter, store.chromosome_painter(author, person, 72, 8)?);
    assert_ne!(
        painter.chromosomes[0].homologues,
        painter.chromosomes[1].homologues
    );
    for chromosome in painter.chromosomes {
        for homologue in chromosome.homologues {
            assert_eq!(homologue.first().unwrap().start, 0);
            assert_eq!(homologue.last().unwrap().end, MODEL_LENGTH);
            assert!(
                homologue
                    .windows(2)
                    .all(|pair| pair[0].end == pair[1].start)
            );
        }
    }
    assert!(store.chromosome_painter(author, person, 72, 9).is_err());
    assert!(matches!(
        store.chromosome_painter(
            ViewContext {
                scope: Scope::Public,
                ..author
            },
            person,
            72,
            8
        ),
        Err(Error::CapabilityDenied)
    ));
    Ok(())
}

#[test]
fn semantic_definition_pin_rejects_stale_and_incompatible_versions() -> Result<(), Error> {
    use atlas_store::semantic::DefinitionPin;
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let definition = ObjectRef::new(world.id);
    let record = |label: &str| Record {
        label: label.into(),
        valid: ValidInterval::default(),
        data: ObjectData::Definition {
            definition: Definition::Tag,
            description: "Governed vocabulary".into(),
        },
        source: None,
    };
    let command = |expected, change| Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Definition version validation".into(),
        scope: Scope::Author,
        changes: vec![change],
    };
    author.revision = store.apply(&command(
        author.revision,
        Change::Create(definition, record("Culture")),
    ))?;
    let old = author;
    let pin = DefinitionPin {
        version: store.get_object(definition, old)?.unwrap().reference(),
    };
    assert_eq!(
        store.validate_definition_pin(pin, old)?.record.label,
        "Culture"
    );
    let person = DefinitionPin {
        version: store.get_object(id(&world, 1), old)?.unwrap().reference(),
    };
    assert!(matches!(
        store.validate_definition_pin(person, old),
        Err(Error::WrongKind)
    ));
    assert!(matches!(
        store.validate_definition_pin(
            pin,
            ViewContext {
                scope: Scope::Public,
                ..old
            }
        ),
        Err(Error::CapabilityDenied)
    ));
    author.revision = store.apply(&command(
        author.revision,
        Change::Correct(definition, record("Cultural tradition")),
    ))?;
    assert!(matches!(
        store.validate_definition_pin(pin, author),
        Err(Error::InvalidInput("definition version pin"))
    ));
    assert_eq!(
        store.validate_definition_pin(pin, old)?.record.label,
        "Culture"
    );
    Ok(())
}

#[test]
fn canonical_concept_identity_is_searchable_and_historical() -> Result<(), Error> {
    use atlas_store::{network::IdentityKind, query::ObjectKindFilter};
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let before = author;
    let concept = store.create_identity(
        author,
        IdentityKind::Concept,
        "River language",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    assert!(store.get_object(concept, before)?.is_none());
    let version = store.get_object(concept, author)?.unwrap();
    assert_eq!(version.record.data, ObjectData::Concept);
    let spec = query(QueryFilter::Kind(ObjectKindFilter::Concept));
    assert!(store.execute_query(&spec, before)?.rows.is_empty());
    assert_eq!(store.execute_query(&spec, author)?.rows[0].object, concept);
    assert!(matches!(
        store.get_object(
            concept,
            ViewContext {
                scope: Scope::Public,
                ..author
            }
        ),
        Err(Error::CapabilityDenied)
    ));
    store.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn classifications_preserve_pins_and_branch_local_withdrawals() -> Result<(), Error> {
    use atlas_store::{
        network::IdentityKind,
        semantic::{ClassificationInput, ConceptProfile, DefinitionPin},
    };
    let dir = tempfile::tempdir()?;
    let mut store = Store::open(dir.path().join("source.sqlite"))?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let concept = store.create_identity(
        author,
        IdentityKind::Concept,
        "Estuary speech",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(2);
    let definition = ObjectRef::new(world.id);
    let record = |label: &str| Record {
        label: label.into(),
        valid: ValidInterval::default(),
        data: ObjectData::Definition {
            definition: Definition::Tag,
            description: "Language vocabulary".into(),
        },
        source: None,
    };
    let edit = |expected, change| Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Vocabulary revision".into(),
        scope: Scope::Author,
        changes: vec![change],
    };
    author.revision = store.apply(&edit(
        author.revision,
        Change::Create(definition, record("Language")),
    ))?;
    let pin = DefinitionPin {
        version: store.get_object(definition, author)?.unwrap().reference(),
    };
    let input = ClassificationInput {
        subject: concept,
        profile: ConceptProfile::Language,
        descriptor: pin,
        source: id(&world, 9),
        starts_at: WorldDay(13),
        ends_before: Some(WorldDay(20)),
    };
    assert!(matches!(
        store.record_classification(
            author,
            ClassificationInput {
                subject: id(&world, 1),
                ..input
            },
            Uuid::new_v4()
        ),
        Err(Error::WrongKind)
    ));
    assert!(matches!(
        store.record_classification(
            author,
            ClassificationInput {
                source: id(&world, 2),
                ..input
            },
            Uuid::new_v4()
        ),
        Err(Error::WrongKind)
    ));
    let before = author;
    let profile_query = query(QueryFilter::ClassifiedAs(ConceptProfile::Language));
    let descriptor_query = query(QueryFilter::ClassifiedBy(definition));
    assert!(store.execute_query(&profile_query, before)?.rows.is_empty());
    let assignment = store.record_classification(author, input, Uuid::new_v4())?;
    author.revision = AuthorRevision(4);
    assert_eq!(
        store.execute_query(&descriptor_query, author)?.rows[0].object,
        concept
    );
    let fact_ref = atlas_store::semantic::FactRef {
        world: world.id,
        family: atlas_store::semantic::FactFamily::Classification,
        id: assignment,
    };
    let fact = store.resolve_semantic_fact(fact_ref, author)?.unwrap();
    assert_eq!(
        fact.data,
        atlas_store::semantic::SemanticFactData::Classification(input)
    );
    for target in [concept, definition, id(&world, 9)] {
        assert!(
            store
                .semantic_fact_backlinks(target, author)?
                .contains(&fact)
        );
    }
    assert!(store.resolve_semantic_fact(fact_ref, before)?.is_none());
    assert_eq!(
        store.execute_query(&profile_query, author)?.rows[0].object,
        concept
    );
    let compound = query(QueryFilter::And(vec![
        profile_query.filter.clone(),
        QueryFilter::ExactLabel("Estuary speech".into()),
    ]));
    assert_eq!(
        store.execute_query(&compound, author)?.rows[0].object,
        concept
    );
    assert!(
        store
            .classifications(before, Some(concept), None)?
            .is_empty()
    );
    assert_eq!(
        store.classifications(author, Some(concept), None)?[0].id,
        assignment
    );
    assert!(matches!(
        store.record_classification(author, input, Uuid::new_v4()),
        Err(Error::InvalidInput("overlapping classification"))
    ));
    assert!(matches!(
        store.classifications(
            ViewContext {
                scope: Scope::Public,
                ..author
            },
            None,
            Some(ConceptProfile::Language)
        ),
        Err(Error::CapabilityDenied)
    ));
    assert!(
        store
            .classifications(
                ViewContext {
                    day: WorldDay(20),
                    ..author
                },
                Some(concept),
                None
            )?
            .is_empty()
    );
    author.revision = store.apply(&edit(
        author.revision,
        Change::Correct(definition, record("Written and spoken language")),
    ))?;
    assert_eq!(
        store.classifications(author, Some(concept), None)?[0].input,
        input
    );
    assert!(matches!(
        store.record_classification(
            author,
            ClassificationInput {
                profile: ConceptProfile::Culture,
                ..input
            },
            Uuid::new_v4()
        ),
        Err(Error::InvalidInput("definition version pin"))
    ));
    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: author.day,
        expected: author.revision,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Alternative classification history".into(),
        scope: Scope::Author,
    })?;
    let inherited = ViewContext {
        branch: fork.id,
        revision: fork.created_revision,
        ..author
    };
    assert_eq!(
        store.classifications(inherited, None, Some(ConceptProfile::Language))?[0].id,
        assignment
    );
    assert_eq!(
        store.execute_query(&profile_query, inherited)?.rows[0].object,
        concept
    );
    store.withdraw_classification(inherited, assignment, Uuid::new_v4())?;
    let after = ViewContext {
        revision: AuthorRevision(fork.created_revision.0 + 1),
        ..inherited
    };
    assert!(store.execute_query(&profile_query, after)?.rows.is_empty());
    assert!(
        store
            .execute_query(&descriptor_query, after)?
            .rows
            .is_empty()
    );
    assert!(store.resolve_semantic_fact(fact_ref, after)?.is_none());
    assert!(store.semantic_fact_backlinks(concept, after)?.is_empty());
    assert!(
        store
            .classifications(after, Some(concept), None)?
            .is_empty()
    );
    assert_eq!(
        store.classifications(inherited, Some(concept), None)?[0].id,
        assignment
    );
    assert_eq!(
        store.classifications(
            ViewContext {
                branch: world.main,
                ..after
            },
            Some(concept),
            None
        )?[0]
            .id,
        assignment
    );
    use atlas_store::{
        archive::restore_archive,
        layout::SavedQueryBinding,
        query::{ParameterBinding, ParameterValue, SaveNamedLabelQuery},
    };
    let mut main = ViewContext {
        branch: world.main,
        ..after
    };
    let saved = store.save_named_parameterized_query(SaveNamedLabelQuery {
        world_id: world.id,
        expected: main.revision,
        query_id: None,
        name: "Languages",
        spec: &profile_query,
        command_id: Uuid::new_v4(),
        scope: Scope::Author,
    })?;
    main.revision = saved.recorded_at;
    let binding = SavedQueryBinding {
        world: world.id,
        query_id: saved.id,
        definition_revision: saved.recorded_at.0,
        bindings: vec![ParameterBinding {
            name: "concept_profile_1".into(),
            value: ParameterValue::ConceptProfile {
                value: ConceptProfile::Language,
            },
        }],
    };
    assert_eq!(
        store
            .execute_query(&binding.resolve(&store, main)?, main)?
            .rows[0]
            .object,
        concept
    );
    let layout = LayoutDocument {
        schema_version: LAYOUT_SCHEMA_VERSION,
        entry: concept,
        root: LayoutElement {
            id: ElementId(Uuid::new_v4()),
            audience: ClaimAudience::Public,
            content: ElementContent::Stack(vec![LayoutElement {
                id: ElementId(Uuid::new_v4()),
                audience: ClaimAudience::Author,
                content: ElementContent::SavedQuery(binding),
            }]),
        },
    };
    main.revision = store.save_entry_layout(main, &layout, true, Uuid::new_v4())?;
    let public = ViewContext {
        scope: Scope::Public,
        ..main
    };
    assert_eq!(
        store.display_entry(public, concept)?.unwrap().content,
        DisplayContent::Stack(vec![])
    );
    let backup = dir.path().join("backup.sqlite");
    let restored = dir.path().join("restored.sqlite");
    store.backup_to(&backup, Scope::Author)?;
    restore_archive(&backup, &restored, Scope::Author)?;
    let reopened = Store::open(&restored)?;
    assert_eq!(
        reopened.classifications(main, Some(concept), None)?,
        store.classifications(main, Some(concept), None)?
    );
    assert_eq!(
        reopened.semantic_fact_backlinks(concept, main)?,
        store.semantic_fact_backlinks(concept, main)?
    );
    assert_eq!(
        reopened.display_entry(public, concept)?,
        store.display_entry(public, concept)?
    );
    assert!(reopened.resolve_semantic_fact(fact_ref, after)?.is_none());
    store.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn semantic_network_relations_require_curated_endpoint_profiles() -> Result<(), Error> {
    use atlas_store::{
        network::{IdentityKind, NetworkInput, NetworkKind},
        semantic::{ClassificationInput, ConceptProfile, DefinitionPin, FactFamily, FactRef},
    };
    let mut store = Store::open(":memory:")?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let culture = store.create_identity(
        author,
        IdentityKind::Concept,
        "Estuary culture",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(2);
    let language = store.create_identity(
        author,
        IdentityKind::Concept,
        "Estuary language",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(3);
    let descriptor = ObjectRef::new(world.id);
    author.revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: author.revision,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Curated vocabulary".into(),
        scope: Scope::Author,
        changes: vec![Change::Create(
            descriptor,
            Record {
                label: "Domain profile".into(),
                valid: ValidInterval::default(),
                data: ObjectData::Definition {
                    definition: Definition::Tag,
                    description: "Curated metadata".into(),
                },
                source: None,
            },
        )],
    })?;
    let pin = DefinitionPin {
        version: store.get_object(descriptor, author)?.unwrap().reference(),
    };
    let source = id(&world, 9);
    let relation = NetworkInput {
        subject: culture,
        target: language,
        source,
        kind: NetworkKind::CultureLanguage,
        role: "documented_language".into(),
        starts_at: 13,
        ends_before: None,
    };
    assert!(matches!(
        store.record_network_fact(author, &relation, Uuid::new_v4()),
        Err(Error::WrongKind)
    ));
    for (subject, profile) in [
        (culture, ConceptProfile::Culture),
        (language, ConceptProfile::Language),
    ] {
        store.record_classification(
            author,
            ClassificationInput {
                subject,
                profile,
                descriptor: pin,
                source,
                starts_at: WorldDay(13),
                ends_before: None,
            },
            Uuid::new_v4(),
        )?;
        author.revision = AuthorRevision(author.revision.0 + 1);
    }
    let wrong = NetworkInput {
        kind: NetworkKind::WorkingMaterial,
        role: "working_medium".into(),
        ..relation.clone()
    };
    assert!(matches!(
        store.record_network_fact(author, &wrong, Uuid::new_v4()),
        Err(Error::WrongKind)
    ));
    let wrong_role = NetworkInput {
        role: "membership".into(),
        ..relation.clone()
    };
    assert!(matches!(
        store.record_network_fact(author, &wrong_role, Uuid::new_v4()),
        Err(Error::InvalidInput("semantic relation role"))
    ));
    let before = author;
    let fact = store.record_network_fact(author, &relation, Uuid::new_v4())?;
    author.revision = AuthorRevision(author.revision.0 + 1);
    let reference = FactRef {
        world: world.id,
        family: FactFamily::Network,
        id: fact,
    };
    assert!(store.resolve_semantic_fact(reference, before)?.is_none());
    assert_eq!(store.network_facts(author, culture)?[0].input, relation);
    assert!(store.resolve_semantic_fact(reference, author)?.is_some());
    store.verify_foreign_keys(Scope::Author)?;
    Ok(())
}

#[test]
fn site_area_preserves_unknown_zero_units_corrections_and_archive() -> Result<(), Error> {
    use atlas_store::{
        archive::restore_archive,
        network::IdentityKind,
        semantic::{AreaReading, AreaTest, AreaUnit, DefinitionPin, FactFamily, FactRef, SiteAreaInput},
    };
    let dir = tempfile::tempdir()?;
    let mut store = Store::open(dir.path().join("source.sqlite"))?;
    let world = seed(&mut store)?;
    let mut author = view(&world, 13);
    let place = store.create_identity(
        author,
        IdentityKind::Place,
        "Estuary workshop",
        Uuid::new_v4(),
    )?;
    author.revision = AuthorRevision(2);
    let descriptor = ObjectRef::new(world.id);
    author.revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: author.revision,
        id: Uuid::new_v4(),
        author: "Fixture".into(),
        rationale: "Area measurement vocabulary".into(),
        scope: Scope::Author,
        changes: vec![Change::Create(
            descriptor,
            Record {
                label: "Recorded site area".into(),
                valid: ValidInterval::default(),
                data: ObjectData::Definition {
                    definition: Definition::Tag,
                    description: "Survey area".into(),
                },
                source: None,
            },
        )],
    })?;
    let pin = DefinitionPin {
        version: store.get_object(descriptor, author)?.unwrap().reference(),
    };
    let input = SiteAreaInput {
        place,
        descriptor: pin,
        source: id(&world, 9),
        reading: AreaReading::Known {
            amount: 0,
            unit: AreaUnit::Hectare,
        },
        starts_at: WorldDay(13),
        ends_before: Some(WorldDay(20)),
    };
    let unknown = query(QueryFilter::SiteArea(AreaTest::Unknown));
    let zero = query(QueryFilter::SiteArea(AreaTest::AtLeast {
        amount: 0,
        unit: AreaUnit::Hectare,
    }));
    assert!(store.site_area(author, place)?.is_none());
    assert!(store.execute_query(&unknown, author)?.rows.is_empty());
    assert!(store.execute_query(&zero, author)?.rows.is_empty());
    assert!(matches!(
        store.record_site_area(
            author,
            SiteAreaInput {
                place: id(&world, 1),
                ..input
            },
            None,
            Uuid::new_v4()
        ),
        Err(Error::WrongKind)
    ));
    assert!(matches!(
        store.record_site_area(
            author,
            SiteAreaInput {
                reading: AreaReading::Known {
                    amount: -1,
                    unit: AreaUnit::Hectare,
                },
                ..input
            },
            None,
            Uuid::new_v4()
        ),
        Err(Error::InvalidInput("site area amount"))
    ));
    let first = store.record_site_area(author, input, None, Uuid::new_v4())?;
    author.revision = AuthorRevision(4);
    assert_eq!(store.site_area(author, place)?.unwrap().input, input);
    assert_eq!(store.execute_query(&zero, author)?.rows[0].object, place);
    assert!(store.execute_query(&unknown, author)?.rows.is_empty());
    assert!(matches!(
        store.record_site_area(author, input, None, Uuid::new_v4()),
        Err(Error::InvalidInput("overlapping site area"))
    ));
    let prior = author;
    let corrected = SiteAreaInput {
        reading: AreaReading::Known {
            amount: 120000,
            unit: AreaUnit::SquareMetre,
        },
        ..input
    };
    let second = store.record_site_area(author, corrected, Some(first), Uuid::new_v4())?;
    author.revision = AuthorRevision(5);
    assert_eq!(store.site_area(prior, place)?.unwrap().input, input);
    let fact = store.site_area(author, place)?.unwrap();
    assert_eq!(fact.input, corrected);
    assert_eq!(fact.corrects, Some(first));
    assert!(store.execute_query(&zero, author)?.rows.is_empty());
    let squared = query(QueryFilter::SiteArea(AreaTest::AtLeast {
        amount: 120000,
        unit: AreaUnit::SquareMetre,
    }));
    assert_eq!(
        store.execute_query(&squared, author)?.rows[0].object,
        place
    );
    let reference = FactRef {
        world: world.id,
        family: FactFamily::SiteArea,
        id: second,
    };
    assert!(store.resolve_semantic_fact(reference, prior)?.is_none());
    assert!(
        store
            .semantic_fact_backlinks(descriptor, author)?
            .iter()
            .any(|fact| fact.reference == reference)
    );
    let next = SiteAreaInput {
        reading: AreaReading::Unknown,
        starts_at: WorldDay(20),
        ends_before: None,
        ..input
    };
    store.record_site_area(author, next, None, Uuid::new_v4())?;
    author.revision = AuthorRevision(6);
    let later = ViewContext {
        day: WorldDay(20),
        ..author
    };
    assert_eq!(store.site_area(later, place)?.unwrap().input, next);
    assert_eq!(store.execute_query(&unknown, later)?.rows[0].object, place);
    assert!(store.execute_query(&zero, later)?.rows.is_empty());
    assert!(matches!(
        store.site_area(
            ViewContext {
                scope: Scope::Public,
                ..later
            },
            place
        ),
        Err(Error::CapabilityDenied)
    ));
    let backup = dir.path().join("backup.sqlite");
    let restored = dir.path().join("restored.sqlite");
    store.backup_to(&backup, Scope::Author)?;
    restore_archive(&backup, &restored, Scope::Author)?;
    let reopened = Store::open(&restored)?;
    assert_eq!(
        reopened.site_area(prior, place)?,
        store.site_area(prior, place)?
    );
    assert_eq!(
        reopened.site_area(later, place)?,
        store.site_area(later, place)?
    );
    Ok(())
}
