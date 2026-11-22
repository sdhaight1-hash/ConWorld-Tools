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
    let opened = Instant::now();
    let mut store = Store::open(&path)?;
    println!("reopen_ms={}", opened.elapsed().as_millis());
    let mut view = ViewContext {
        world: world.id,
        branch: world.main,
        revision,
        day: WorldDay(10),
        scope: Scope::Author,
    };
    let house = store.create_identity(view, atlas_store::network::IdentityKind::FamilyHouse, "Benchmark house", Uuid::new_v4())?;
    view.revision = AuthorRevision(view.revision.0+1);
    store.record_network_fact(view, &atlas_store::network::NetworkInput {
        subject: object(count-1), target: house, source: object(2),
        kind: atlas_store::network::NetworkKind::Membership, role: "member".into(), starts_at: 0, ends_before: None,
    }, Uuid::new_v4())?;
    view.revision = AuthorRevision(view.revision.0+1);
    store.record_parentage(view, atlas_store::genealogy::RecordParentage {
        parent: object(3), child: object(count-1), kind: ParentageKind::Biological,
        starts_at: WorldDay(0), ends_before: None, rationale: "Fixture", command_id: Uuid::new_v4(),
    })?;
    view.revision = AuthorRevision(view.revision.0+1);
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
        let network_start = Instant::now();
        assert_eq!(store.network_facts(view, house)?.len(), 1);
        let network_us=network_start.elapsed().as_micros();
        let founder_start=Instant::now();
        assert_eq!(store.founder_genome(view,object(count-1),72)?.model,atlas_store::inheritance::FOUNDER_MODEL_VERSION);
        let founder_us=founder_start.elapsed().as_micros();
        println!("sample={sample} network_us={network_us} founder_us={founder_us}");
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
    let start=Instant::now();
    let backup=dir.path().join("backup.sqlite");
    store.backup_to(&backup,Scope::Author)?;
    println!("backup_ms={} backup_bytes={}",start.elapsed().as_millis(),std::fs::metadata(&backup)?.len());
    let start=Instant::now();
    let restore=dir.path().join("restored.sqlite");
    atlas_store::archive::restore_archive(&backup,&restore,Scope::Author)?;
    let restored=Store::open(&restore)?;
    assert_eq!(restored.network_facts(view,house)?.len(),1);
    assert_eq!(restored.founder_genome(view,object(count-1),72)?,store.founder_genome(view,object(count-1),72)?);
    println!("restore_verify_ms={}",start.elapsed().as_millis());
    Ok(())
}
