//! Native semantic store workloads: sparse and dense author queries on real facts.
use atlas_store::{
    query::{QUERY_SCHEMA_VERSION, QueryControl, QueryFilter, QuerySpec},
    semantic::{ClassificationInput, ConceptProfile, DefinitionPin, FactFamily, FactRef},
    *,
};
use std::time::{Duration, Instant};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1000".into())
        .parse::<usize>()?;
    if !matches!(count, 1000 | 10000 | 50000) {
        return Err("choose 1000, 10000 or 50000".into());
    }
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Semantic workload", Scope::Author)?;
    let mut view = ViewContext {
        world: world.id,
        branch: world.main,
        revision: world.revision,
        day: WorldDay(10),
        scope: Scope::Author,
    };
    let object = |index: usize| ObjectRef {
        world: world.id,
        id: ObjectId(Uuid::from_u128(index as u128 + 1)),
    };
    let source = object(count);
    let descriptor = object(count + 1);
    let create = |object, label, data| {
        Change::Create(
            object,
            Record {
                label,
                data,
                valid: ValidInterval::default(),
                source: None,
            },
        )
    };
    let started = Instant::now();
    view.revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: view.revision,
        id: Uuid::new_v4(),
        author: "Benchmark".into(),
        rationale: "Semantic workload vocabulary".into(),
        scope: Scope::Author,
        changes: vec![
            create(
                source,
                "Survey register".into(),
                ObjectData::Source {
                    citation: "Fixture register".into(),
                },
            ),
            create(
                descriptor,
                "Semantic descriptor".into(),
                ObjectData::Definition {
                    definition: Definition::Tag,
                    description: "Benchmark descriptor".into(),
                },
            ),
        ],
    })?;
    let pin = DefinitionPin {
        version: store
            .get_object(descriptor, view)?
            .ok_or("missing descriptor")?
            .reference(),
    };
    for start in (0..count).step_by(1000) {
        view.revision = store.apply(&Command {
            world: world.id,
            branch: world.main,
            expected: view.revision,
            id: Uuid::new_v4(),
            author: "Benchmark".into(),
            rationale: "Concept batch".into(),
            scope: Scope::Author,
            changes: (start..count.min(start + 1000))
                .map(|i| create(object(i), format!("Concept {i}"), ObjectData::Concept))
                .collect(),
        })?;
    }
    let mut last = None;
    // Sparse (1%) and dense (99%) profiles in the same world.
    for i in 0..count {
        let profile = if i % 100 == 0 {
            ConceptProfile::Culture
        } else {
            ConceptProfile::Language
        };
        last = Some(store.record_classification(
            view,
            ClassificationInput {
                subject: object(i),
                profile,
                descriptor: pin,
                source,
                starts_at: view.day,
                ends_before: None,
            },
            Uuid::new_v4(),
        )?);
        view.revision = AuthorRevision(view.revision.0 + 1);
        if (i + 1) % 1000 == 0 {
            println!(
                "entities={count} classified={} elapsed_ms={}",
                i + 1,
                started.elapsed().as_millis()
            );
        }
    }
    let fact = FactRef {
        world: world.id,
        family: FactFamily::Classification,
        id: last.ok_or("missing fact")?,
    };
    println!(
        "entities={count} classifications={count} setup_ms={}",
        started.elapsed().as_millis()
    );
    let spec = |profile| QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter: QueryFilter::ClassifiedAs(profile),
        limit: 500,
    };
    for sample in 1..=5 {
        let sparse = Instant::now();
        let result = store.execute_query(&spec(ConceptProfile::Culture), view)?;
        assert_eq!(result.rows.len(), count / 100);
        assert!(!result.truncated);
        let sparse_us = sparse.elapsed().as_micros();
        let dense = Instant::now();
        let result = store.execute_query(&spec(ConceptProfile::Language), view)?;
        assert_eq!(result.rows.len(), 500);
        assert!(result.truncated);
        let dense_us = dense.elapsed().as_micros();
        let point = Instant::now();
        assert!(store.resolve_semantic_fact(fact, view)?.is_some());
        let point_us = point.elapsed().as_micros();
        let links = Instant::now();
        assert_eq!(
            store
                .semantic_fact_backlinks(object(count - 1), view)?
                .len(),
            1
        );
        println!(
            "sample={sample} sparse_us={sparse_us} dense_us={dense_us} point_us={point_us} backlinks_us={}",
            links.elapsed().as_micros()
        );
    }
    // One-shot and paged semantic queries share a bounded seek path.
    let spec = spec(ConceptProfile::Language);
    if count == 1000 {
        let first = store.execute_query_page(&spec, view, None)?;
        assert_eq!(first.rows.len(), 500);
        let second = store.execute_query_page(&spec, view, first.next_cursor.as_ref())?;
        assert_eq!(second.rows.len(), 490);
        assert!(second.next_cursor.is_none());
        let mut ids = std::collections::HashSet::new();
        for row in first.rows.iter().chain(&second.rows) {
            assert!(ids.insert(row.object));
        }
        println!("dense_pages=2 distinct={}", ids.len());
    } else {
        let started = Instant::now();
        let control = QueryControl::new(Duration::from_secs(30))?;
        let first = store.execute_query_page_controlled(&spec, view, None, &control)?;
        assert_eq!(first.rows.len(), 500);
        assert!(first.next_cursor.is_some());
        let control = QueryControl::new(Duration::from_secs(30))?;
        let second =
            store.execute_query_page_controlled(&spec, view, first.next_cursor.as_ref(), &control)?;
        assert_eq!(second.rows.len(), 500);
        assert!(second.next_cursor.is_some());
        let mut ids = std::collections::HashSet::new();
        for row in first.rows.iter().chain(&second.rows) {
            assert!(ids.insert(row.object));
        }
        println!(
            "dense_first_two_pages=2 distinct={} elapsed_ms={}",
            ids.len(),
            started.elapsed().as_millis()
        );
    }
    let descriptor_spec = QuerySpec {
        filter: QueryFilter::ClassifiedBy(descriptor),
        ..spec.clone()
    };
    let started = Instant::now();
    let mut cursor = None;
    let mut seen = std::collections::HashSet::new();
    for _ in 0..2 {
        let control = QueryControl::new(Duration::from_secs(30))?;
        let page = store.execute_query_page_controlled(
            &descriptor_spec,
            view,
            cursor.as_ref(),
            &control,
        )?;
        assert_eq!(page.rows.len(), 500);
        for row in page.rows {
            assert!(seen.insert(row.object));
        }
        cursor = page.next_cursor;
    }
    assert_eq!(seen.len(), 1000);
    assert_eq!(cursor.is_some(), count > 1000);
    println!(
        "descriptor_first_two_pages=2 distinct={} elapsed_ms={}",
        seen.len(),
        started.elapsed().as_millis()
    );
    let composite = QuerySpec {
        filter: QueryFilter::And(vec![
            QueryFilter::ClassifiedBy(descriptor),
            QueryFilter::ClassifiedAs(ConceptProfile::Language),
            QueryFilter::LabelContains("Concept ".into()),
        ]),
        ..spec.clone()
    };
    let control = QueryControl::new(Duration::from_secs(30))?;
    let started = Instant::now();
    let first = store.execute_query_page_controlled(&composite, view, None, &control)?;
    let direct = store.execute_query_controlled(&composite, view, &control)?;
    assert_eq!(first.rows, direct.rows);
    assert!(direct.truncated);
    let control = QueryControl::new(Duration::from_secs(30))?;
    let second =
        store.execute_query_page_controlled(&composite, view, first.next_cursor.as_ref(), &control)?;
    assert_eq!(first.rows.len(), 500);
    assert_eq!(second.rows.len(), if count == 1000 { 490 } else { 500 });
    let mut ids = std::collections::HashSet::new();
    for row in first.rows.iter().chain(&second.rows) {
        assert!(ids.insert(row.object));
    }
    println!(
        "composite_first_two_pages=2 distinct={} elapsed_ms={}",
        ids.len(),
        started.elapsed().as_millis()
    );
    let control = QueryControl::new(Duration::from_secs(30))?;
    control.cancel();
    assert!(matches!(
        store.execute_query_controlled(&spec, view, &control),
        Err(Error::QueryCancelled)
    ));
    let deadline = QueryControl::new(Duration::ZERO)?;
    assert!(matches!(
        store.execute_query_controlled(&spec, view, &deadline),
        Err(Error::QueryDeadlineExceeded)
    ));
    println!("cancellation=verified deadline=verified");
    if let Ok(status) = std::fs::read_to_string("/proc/self/status")
        && let Some(peak) = status.lines().find(|line| line.starts_with("VmHWM:"))
    {
        println!("{peak}");
    }
    store.verify_foreign_keys(Scope::Author)?;
    Ok(())
}
