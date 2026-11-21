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
