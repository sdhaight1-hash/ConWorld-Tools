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
    let persisted = std::env::args().nth(2).as_deref() == Some("--persisted");
    let directory = tempfile::tempdir()?;
    let database = if persisted {
        directory.path().join("semantic.sqlite")
    } else {
        std::path::PathBuf::from(":memory:")
    };
    println!(
        "storage={} entities={count}",
        if persisted { "disk" } else { "memory" }
    );
    let mut store = Store::open(&database)?;
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
        let second = store.execute_query_page_controlled(
            &spec,
            view,
            first.next_cursor.as_ref(),
            &control,
        )?;
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
    let second = store.execute_query_page_controlled(
        &composite,
        view,
        first.next_cursor.as_ref(),
        &control,
    )?;
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
    let boolean = QuerySpec {
        filter: QueryFilter::Or(vec![
            QueryFilter::ClassifiedAs(ConceptProfile::Culture),
            QueryFilter::And(vec![
                QueryFilter::ClassifiedAs(ConceptProfile::Language),
                QueryFilter::Not(Box::new(QueryFilter::ClassifiedAs(ConceptProfile::Culture))),
            ]),
        ]),
        ..spec.clone()
    };
    let started = Instant::now();
    let control = QueryControl::new(Duration::from_secs(30))?;
    let first = store.execute_query_page_controlled(&boolean, view, None, &control)?;
    let second = store.execute_query_page_controlled(
        &boolean,
        view,
        first.next_cursor.as_ref(),
        &control,
    )?;
    assert_eq!(first.rows.len(), 500);
    assert_eq!(second.rows.len(), 500);
    assert_eq!(second.next_cursor.is_some(), count > 1000);
    let mut ids = std::collections::HashSet::new();
    for row in first.rows.iter().chain(&second.rows) {
        assert!(ids.insert(row.object));
    }
    assert_eq!(
        store
            .execute_query_controlled(&boolean, view, &control)?
            .rows,
        first.rows
    );
    println!(
        "boolean_first_two_pages=2 distinct={} elapsed_ms={}",
        ids.len(),
        started.elapsed().as_millis()
    );
    // A dense false arm must not consume the 2,000-candidate budget. The
    // remaining sparse OR arm must produce the same identities in both orders.
    let missing = QueryFilter::ClassifiedBy(ObjectRef::new(world.id));
    let dense = QueryFilter::ClassifiedAs(ConceptProfile::Language);
    let sparse = QueryFilter::ClassifiedAs(ConceptProfile::Culture);
    let expected = store.execute_query(
        &QuerySpec {
            filter: sparse.clone(),
            ..spec.clone()
        },
        view,
    )?;
    for children in [
        vec![dense.clone(), missing.clone()],
        vec![missing.clone(), dense.clone()],
    ] {
        let filter = QueryFilter::And(children);
        let empty = QuerySpec {
            filter: filter.clone(),
            ..spec.clone()
        };
        let control = QueryControl::new(Duration::from_secs(30))?;
        let result = store.execute_query_controlled(&empty, view, &control)?;
        assert!(result.rows.is_empty());
        assert!(!result.truncated);
        let page = store.execute_query_page_controlled(&empty, view, None, &control)?;
        assert!(page.rows.is_empty());
        assert!(page.next_cursor.is_none());
        for arms in [
            vec![filter.clone(), sparse.clone()],
            vec![sparse.clone(), filter.clone()],
        ] {
            let union = QuerySpec {
                filter: QueryFilter::Or(arms),
                ..spec.clone()
            };
            let control = QueryControl::new(Duration::from_secs(30))?;
            let actual = store.execute_query_controlled(&union, view, &control)?;
            let page = store.execute_query_page_controlled(&union, view, None, &control)?;
            assert_eq!(actual.rows, expected.rows);
            assert_eq!(actual.truncated, expected.truncated);
            assert_eq!(page.rows, expected.rows);
            assert!(page.next_cursor.is_none());
        }
    }
    println!("absent_descriptor_operand_permutations=verified");
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
    // Windows equivalent of the Linux VmHWM sample above. The workload
    // process is still alive; PowerShell only reads its peak resident bytes.
    // This is outside all timed query sections and requires no crate or service.
    #[cfg(target_os = "windows")]
    {
        let command = format!("(Get-Process -Id {}).PeakWorkingSet64", std::process::id());
        let output = std::process::Command::new("powershell.exe")
            .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"])
            .arg(command)
            .output()?;
        if !output.status.success() {
            return Err("Windows peak working set measurement failed".into());
        }
        let peak: u64 = std::str::from_utf8(&output.stdout)?.trim().parse()?;
        if peak == 0 {
            return Err("Windows peak working set measurement is zero".into());
        }
        println!("windows_peak_working_set_bytes={peak}");
    }
    store.verify_foreign_keys(Scope::Author)?;
    if persisted {
        let started = Instant::now();
        let backup = directory.path().join("backup.sqlite");
        let restored = directory.path().join("restored.sqlite");
        store.backup_to(&backup, Scope::Author)?;
        let manifest = atlas_store::archive::restore_archive(&backup, &restored, Scope::Author)?;
        let reopened = Store::open(&restored)?;
        assert_eq!(
            reopened.resolve_semantic_fact(fact, view)?,
            store.resolve_semantic_fact(fact, view)?
        );
        let expected = store.execute_query(&composite, view)?;
        let actual = reopened.execute_query(&composite, view)?;
        assert_eq!(actual.rows, expected.rows);
        assert_eq!(actual.truncated, expected.truncated);
        println!(
            "archive_replay_schema={} database_bytes={} backup_bytes={} elapsed_ms={}",
            manifest.schema,
            std::fs::metadata(&database)?.len(),
            std::fs::metadata(&backup)?.len(),
            started.elapsed().as_millis()
        );
    }
    Ok(())
}
