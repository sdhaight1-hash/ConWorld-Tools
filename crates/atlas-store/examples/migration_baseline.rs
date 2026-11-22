//! Compile this fixture generator against the pinned schema-36 source in CI.
use atlas_store::{
    genealogy::{ParentageKind, RecordParentage},
    network::{NetworkInput, NetworkKind},
    *,
};
use std::path::PathBuf;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(std::env::args_os().nth(1).ok_or("fixture path required")?);
    if path.exists() || path.with_extension("json").exists() {
        return Err("fixture output already exists".into());
    }
    let mut store = Store::open(&path)?;
    let world = store.create_world("Released schema-36 fixture", Scope::Author)?;
    let object = |number| ObjectRef {
        world: world.id,
        id: ObjectId(Uuid::from_u128(number)),
    };
    let record = |label: &str, data, source| Record {
        label: label.into(),
        data,
        source,
        valid: ValidInterval::default(),
    };
    let command = |expected, changes| Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Legacy baseline".into(),
        rationale: "Released database fixture".into(),
        scope: Scope::Author,
        changes,
    };
    let first = store.apply(&command(
        world.revision,
        vec![
            Change::Create(
                object(1),
                record(
                    "Register",
                    ObjectData::Source {
                        citation: "Original folio 7".into(),
                    },
                    None,
                ),
            ),
            Change::Create(
                object(2),
                record("Alice Vale", ObjectData::Person, Some(object(1))),
            ),
            Change::Create(object(3), record("Beatrice Vale", ObjectData::Person, None)),
            Change::Create(
                object(4),
                record("House Vale", ObjectData::FamilyHouse, None),
            ),
        ],
    ))?;
    let mut view = ViewContext {
        world: world.id,
        branch: world.main,
        revision: first,
        day: WorldDay(13),
        scope: Scope::Author,
    };
    let network = store.record_network_fact(
        view,
        &NetworkInput {
            subject: object(3),
            target: object(4),
            source: object(1),
            kind: NetworkKind::Membership,
            role: "member".into(),
            starts_at: 10,
            ends_before: None,
        },
        Uuid::new_v4(),
    )?;
    view.revision.0 += 1;
    let parentage = store.record_parentage(
        view,
        RecordParentage {
            parent: object(2),
            child: object(3),
            kind: ParentageKind::Biological,
            starts_at: WorldDay(0),
            ends_before: None,
            command_id: Uuid::new_v4(),
            rationale: "Original register",
        },
    )?;
    view.revision.0 += 1;
    let revision = store.apply(&command(
        view.revision,
        vec![Change::Correct(
            object(2),
            record("Alice Thorne", ObjectData::Person, Some(object(1))),
        )],
    ))?;
    store.verify_foreign_keys(Scope::Author)?;
    drop(store);
    let conn =
        rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let schema: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    std::fs::write(
        path.with_extension("json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema": schema,
            "world": world.id,
            "branch": world.main,
            "first": first.0,
            "revision": revision.0,
            "network": network,
            "parentage": parentage
        }))?,
    )?;
    println!("generated_schema={schema} revision={}", revision.0);
    Ok(())
}
