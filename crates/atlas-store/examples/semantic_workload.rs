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
        let result = store.execute_query(&spec(ConceptProfile::Language), view);
        if count <= 2000 {
            let result = result?;
            assert_eq!(result.rows.len(), 500);
            assert!(result.truncated);
        } else {
            assert!(matches!(result, Err(Error::QueryBudgetExceeded)));
        }
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
    // Verify actual pagination on the bounded workload; dense large queries
    // must reject instead of silently disguising budget exhaustion as absence.
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
        assert!(matches!(
            store.execute_query_page(&spec, view, None),
            Err(Error::QueryBudgetExceeded)
        ));
        println!("dense_pages=budget_rejected");
    }
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
