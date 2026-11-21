//! Reproducible disk-backed focused-read workload. Values are wall-clock time.
use atlas_store::{
    conflict::{CausalLink, LinkStatus},
    genealogy::{Parentage, ParentageKind},
    query::{QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec},
    *,
};
use std::time::Instant;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "50000".into())
        .parse::<usize>()?;
    if !(10..=50_000).contains(&count) {
        return Err("choose 10 to 50000 records".into());
    }
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("bench.sqlite");
    let mut store = Store::open(&path)?;
    let world = store.create_world("Focused world", Scope::Author)?;
    let object = |i: usize| ObjectRef {
        world: world.id,
        id: ObjectId(Uuid::from_u128(i as u128 + 1)),
    };
    let mut revision = world.revision;
    let write_start = Instant::now();
    for start in (0..count).step_by(1000) {
        let changes = (start..count.min(start + 1000))
            .map(|i| {
                let data = match i {
                    0 | 1 => ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(i as i64)),
                    },
                    2 => ObjectData::Source {
                        citation: "Gazette".into(),
                    },
                    _ => ObjectData::Person,
                };
                Change::Create(
                    object(i),
                    Record {
                        label: format!("Resident {i}"),
                        data,
                        valid: ValidInterval::default(),
                        source: None,
                    },
                )
            })
            .collect();
        revision = store.apply(&Command {
            world: world.id,
            branch: world.main,
            expected: revision,
            id: Uuid::new_v4(),
            scope: Scope::Author,
            author: "Benchmark".into(),
            rationale: "Fixture batch".into(),
            changes,
        })?;
    }
    println!(
        "records={count} write_ms={} bytes={}",
        write_start.elapsed().as_millis(),
        std::fs::metadata(&path)?.len()
    );
    drop(store);
    let store = Store::open(&path)?;
    let view = ViewContext {
        world: world.id,
        branch: world.main,
        revision,
        day: WorldDay(10),
        scope: Scope::Author,
    };
    let query = QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter: QueryFilter::FullTextPhrase(format!("Resident {}", count - 1)),
        limit: 10,
    };
    for sample in 1..=5 {
        let start = Instant::now();
        let result = store.execute_query(&query, view)?;
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].object, object(count - 1));
        let query_us = start.elapsed().as_micros();
        let start = Instant::now();
        let pedigree = store.pedigree_at(
            view,
            &[Parentage {
                child: object(count - 1),
                parent: object(3),
                kind: ParentageKind::Legal,
                valid: ValidInterval::default(),
                accepted: true,
            }],
        )?;
        assert_eq!(
            pedigree.parents(object(count - 1), ParentageKind::Legal)?,
            vec![object(3)]
        );
        let family_us = start.elapsed().as_micros();
        let start = Instant::now();
        let impact = store.confirmed_impact(
            view,
            object(0),
            &[CausalLink {
                cause: object(0),
                consequence: object(1),
                source: object(2),
                status: LinkStatus::Confirmed,
            }],
            2,
            10,
        )?;
        assert_eq!(impact.events, vec![object(1)]);
        println!(
            "sample={sample} query_us={query_us} family_us={family_us} impact_us={}",
            start.elapsed().as_micros()
        );
    }
    Ok(())
}
