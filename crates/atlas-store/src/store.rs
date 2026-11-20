use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use uuid::Uuid;

use crate::*;

const APPLICATION_ID: i64 = 0x4357_4c44; // CWLD
const SCHEMA_VERSION: i64 = 2;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: i64 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
        let application_id: i64 =
            tx.pragma_query_value(None, "application_id", |row| row.get(0))?;
        match version {
            version if version > SCHEMA_VERSION => return Err(Error::SchemaTooNew(version)),
            0 if application_id == 0 => {
                let empty: bool = tx.query_row(
                    "SELECT NOT EXISTS(SELECT 1 FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%')",
                    [],
                    |row| row.get(0),
                )?;
                if !empty {
                    return Err(Error::WrongDatabase);
                }
                tx.execute_batch(include_str!("../migrations/0001.sql"))?;
                tx.pragma_update(None, "application_id", APPLICATION_ID)?;
            }
            1..=SCHEMA_VERSION if application_id == APPLICATION_ID => {}
            _ => return Err(Error::WrongDatabase),
        }
        if version < 2 {
            tx.execute_batch(include_str!("../migrations/0002.sql"))?;
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        tx.commit()?;
        Ok(Self { conn })
    }

    pub fn create_world(&mut self, name: &str, scope: Scope) -> Result<World, Error> {
        require_author(scope)?;
        validate_text(name, "world name")?;
        let id = WorldId(Uuid::new_v4());
        let main = BranchId(Uuid::new_v4());
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO worlds (id, name) VALUES (?1, ?2)",
            params![id.0.as_bytes(), name],
        )?;
        tx.execute(
            "INSERT INTO branches (world_id, id, is_main) VALUES (?1, ?2, 1)",
            params![id.0.as_bytes(), main.0.as_bytes()],
        )?;
        tx.commit()?;
        Ok(World {
            id,
            name: name.into(),
            main,
            revision: AuthorRevision(0),
        })
    }

    pub fn world(&self, world: WorldId, scope: Scope) -> Result<World, Error> {
        require_author(scope)?;
        load_world(&self.conn, world)
    }

    pub fn apply(&mut self, command: &Command) -> Result<AuthorRevision, Error> {
        require_author(command.scope)?;
        validate_text(&command.author, "command author")?;
        validate_text(&command.rationale, "command rationale")?;
        if command.changes.is_empty() {
            return Err(Error::InvalidInput("empty command"));
        }

        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = load_world(&tx, command.world)?;
        if command.branch != world.main {
            return Err(Error::NotFound);
        }
        if let Some(revision) = tx
            .query_row(
                "SELECT revision FROM author_revisions WHERE world_id = ?1 AND command_id = ?2",
                params![world.id.0.as_bytes(), command.id.as_bytes()],
                |row| row.get(0),
            )
            .optional()?
        {
            return Err(Error::DuplicateCommand(AuthorRevision(revision)));
        }
        if command.expected != world.revision {
            return Err(Error::RevisionConflict {
                expected: command.expected,
                actual: world.revision,
            });
        }
        let revision = world
            .revision
            .0
            .checked_add(1)
            .ok_or(Error::InvalidRevision)?;
        tx.execute(
            "INSERT INTO author_revisions (world_id, revision, branch_id, command_id, author, rationale)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![world.id.0.as_bytes(), revision, command.branch.0.as_bytes(), command.id.as_bytes(),
                command.author, command.rationale],
        )?;
        let mut changed = HashSet::new();
        for change in &command.changes {
            let (object, record) = match change {
                Change::Create(object, record)
                | Change::Correct(object, record)
                | Change::RecordState(object, record) => (*object, record),
            };
            if object.world != world.id {
                return Err(Error::WrongWorld);
            }
            if !changed.insert(object.id) {
                return Err(Error::RepeatedObject);
            }
            let branch = command.branch;
            let state_revision = match change {
                Change::Create(..) => revision,
                Change::Correct(..) => {
                    let (state, _) =
                        latest_state(&tx, object, branch, None)?.ok_or(Error::NotFound)?;
                    if let Some((_, previous)) = latest_state(&tx, object, branch, Some(state))?
                        && !matches!(
                            (previous.record.valid.end_exclusive, record.valid.start),
                            (Some(end), Some(start)) if end <= start
                        )
                    {
                        return Err(Error::InvalidInterval);
                    }
                    state
                }
                Change::RecordState(..) => {
                    let start = record.valid.start.ok_or(Error::InvalidInterval)?;
                    let (state, mut previous) =
                        latest_state(&tx, object, branch, None)?.ok_or(Error::NotFound)?;
                    if previous.record.valid.start.is_some_and(|day| day >= start) {
                        return Err(Error::InvalidInterval);
                    }
                    if previous
                        .record
                        .valid
                        .end_exclusive
                        .is_none_or(|day| day > start)
                    {
                        previous.record.valid.end_exclusive = Some(start);
                        write_version(
                            &tx,
                            branch,
                            revision,
                            state,
                            object,
                            &previous.record,
                            false,
                        )?;
                    }
                    revision
                }
            };
            write_version(
                &tx,
                branch,
                revision,
                state_revision,
                object,
                record,
                matches!(change, Change::Create(..)),
            )?;
            tx.execute(
                "INSERT INTO invalidations (world_id, revision, object_id) VALUES (?1, ?2, ?3)",
                params![object.world.0.as_bytes(), revision, object.id.0.as_bytes()],
            )?;
        }
        tx.commit()?;
        Ok(AuthorRevision(revision))
    }

    pub fn get_object(
        &self,
        object: ObjectRef,
        view: ViewContext,
    ) -> Result<Option<Version>, Error> {
        require_author(view.scope)?;
        if object.world != view.world {
            return Err(Error::WrongWorld);
        }
        // Keep the context check and object query on the same read snapshot.
        let tx = self.conn.unchecked_transaction()?;
        let world = load_world(&tx, view.world)?;
        if view.branch != world.main {
            return Err(Error::NotFound);
        }
        if view.revision < AuthorRevision(0) || view.revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        let version = tx.query_row(
            "SELECT revision, kind, label, valid_start, valid_end, event_day, citation, source_id,
                    earliest_day, latest_day, date_label
             FROM object_versions AS v
             WHERE world_id = ?1 AND object_id = ?2 AND branch_id = ?3 AND revision <= ?4
               AND (valid_start IS NULL OR valid_start <= ?5)
               AND (valid_end IS NULL OR valid_end > ?5)
               AND NOT EXISTS (
                   SELECT 1 FROM object_versions AS newer
                   WHERE newer.world_id = v.world_id AND newer.object_id = v.object_id
                     AND newer.branch_id = v.branch_id AND newer.state_revision = v.state_revision
                     AND newer.revision > v.revision AND newer.revision <= ?4
               )
             ORDER BY state_revision DESC LIMIT 1",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), view.branch.0.as_bytes(), view.revision.0, view.day.0],
            |row| read_version(row, object),
        ).optional()?;
        tx.commit()?;
        Ok(version)
    }

    pub fn verify_foreign_keys(&self, scope: Scope) -> Result<(), Error> {
        require_author(scope)?;
        let violation = self
            .conn
            .query_row("PRAGMA foreign_key_check", [], |row| row.get(0))
            .optional()?;
        match violation {
            Some(table) => Err(Error::Integrity(table)),
            None => Ok(()),
        }
    }

    pub fn backup_to(&self, path: impl AsRef<Path>, scope: Scope) -> Result<(), Error> {
        require_author(scope)?;
        let path = std::path::absolute(path)?;
        let filename = path
            .to_str()
            .ok_or_else(|| Error::InvalidBackupPath(path.to_path_buf()))?;
        // Claim the destination without overwriting a prior backup.
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        if let Err(error) = self.conn.execute("VACUUM INTO ?1", [filename]) {
            fs::remove_file(&path)?;
            return Err(error.into());
        }
        Ok(())
    }
}

fn require_author(scope: Scope) -> Result<(), Error> {
    if scope != Scope::Author {
        return Err(Error::CapabilityDenied);
    }
    Ok(())
}

fn validate_text(value: &str, field: &'static str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::InvalidInput(field));
    }
    Ok(())
}

fn load_world(conn: &Connection, id: WorldId) -> Result<World, Error> {
    conn.query_row(
        "SELECT w.name, b.id, COALESCE(
             (SELECT MAX(revision) FROM author_revisions WHERE world_id = w.id), 0)
         FROM worlds w JOIN branches b ON b.world_id = w.id AND b.is_main = 1
         WHERE w.id = ?1",
        [id.0.as_bytes()],
        |row| {
            Ok(World {
                id,
                name: row.get(0)?,
                main: BranchId(Uuid::from_bytes(row.get(1)?)),
                revision: AuthorRevision(row.get(2)?),
            })
        },
    )
    .optional()?
    .ok_or(Error::NotFound)
}

fn kind_of(conn: &Connection, object: ObjectRef) -> Result<Option<String>, Error> {
    Ok(conn
        .query_row(
            "SELECT kind FROM objects WHERE world_id = ?1 AND id = ?2",
            params![object.world.0.as_bytes(), object.id.0.as_bytes()],
            |row| row.get(0),
        )
        .optional()?)
}

fn latest_state(
    conn: &Connection,
    object: ObjectRef,
    branch: BranchId,
    before: Option<i64>,
) -> Result<Option<(i64, Version)>, Error> {
    Ok(conn
        .query_row(
            "SELECT revision, kind, label, valid_start, valid_end, event_day, citation, source_id,
                earliest_day, latest_day, date_label, state_revision
         FROM object_versions
         WHERE world_id = ?1 AND object_id = ?2 AND branch_id = ?3
           AND (?4 IS NULL OR state_revision < ?4)
         ORDER BY state_revision DESC, revision DESC LIMIT 1",
            params![
                object.world.0.as_bytes(),
                object.id.0.as_bytes(),
                branch.0.as_bytes(),
                before
            ],
            |row| Ok((row.get(11)?, read_version(row, object)?)),
        )
        .optional()?)
}

fn write_version(
    tx: &Transaction<'_>,
    branch: BranchId,
    revision: i64,
    state_revision: i64,
    object: ObjectRef,
    record: &Record,
    create: bool,
) -> Result<(), Error> {
    validate_text(&record.label, "object label")?;
    if matches!((record.valid.start, record.valid.end_exclusive), (Some(start), Some(end)) if start >= end)
    {
        return Err(Error::InvalidInterval);
    }
    let (kind, occurrence, citation) = match &record.data {
        ObjectData::Person => ("person", &EventDate::Unknown, None),
        ObjectData::HistoricalEvent { occurrence } => ("historical_event", occurrence, None),
        ObjectData::Source { citation } => {
            validate_text(citation, "source citation")?;
            ("source", &EventDate::Unknown, Some(citation.as_str()))
        }
    };
    let (event_day, earliest, latest, date_label) = match occurrence {
        EventDate::Exact(day) => (Some(day.0), None, None, None),
        EventDate::Unknown => (None, None, None, None),
        EventDate::Uncertain {
            earliest,
            latest,
            label,
        } => {
            validate_text(label, "uncertain date label")?;
            if (earliest.is_none() && latest.is_none())
                || matches!((earliest, latest), (Some(start), Some(end)) if start > end)
            {
                return Err(Error::InvalidOccurrence);
            }
            (
                None,
                earliest.map(|day| day.0),
                latest.map(|day| day.0),
                Some(label.as_str()),
            )
        }
    };
    if let Some(source) = record.source {
        if source.world != object.world {
            return Err(Error::WrongWorld);
        }
        if source == object {
            return Err(Error::InvalidInput("self-citation"));
        }
        match kind_of(tx, source)?.as_deref() {
            Some("source") => {}
            Some(_) => return Err(Error::WrongKind),
            None => return Err(Error::NotFound),
        }
    }
    match (create, kind_of(tx, object)?.as_deref()) {
        (true, None) => {
            tx.execute(
                "INSERT INTO objects (world_id, id, kind) VALUES (?1, ?2, ?3)",
                params![object.world.0.as_bytes(), object.id.0.as_bytes(), kind],
            )?;
        }
        (true, Some(_)) => return Err(Error::ObjectExists),
        (false, None) => return Err(Error::NotFound),
        (false, Some(existing)) if existing != kind => return Err(Error::WrongKind),
        (false, Some(_)) => {}
    }
    tx.execute(
        "INSERT INTO object_versions (
             world_id, object_id, branch_id, revision, kind, label,
             valid_start, valid_end, event_day, citation, source_id,
             earliest_day, latest_day, date_label, state_revision
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            object.world.0.as_bytes(),
            object.id.0.as_bytes(),
            branch.0.as_bytes(),
            revision,
            kind,
            record.label,
            record.valid.start.map(|day| day.0),
            record.valid.end_exclusive.map(|day| day.0),
            event_day,
            citation,
            record.source.map(|source| *source.id.0.as_bytes()),
            earliest,
            latest,
            date_label,
            state_revision,
        ],
    )?;
    Ok(())
}

fn read_version(row: &rusqlite::Row<'_>, object: ObjectRef) -> rusqlite::Result<Version> {
    let data = match row.get::<_, String>(1)?.as_str() {
        "person" => ObjectData::Person,
        "historical_event" => {
            let occurrence = match (
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<String>>(10)?,
            ) {
                (Some(day), None) => EventDate::Exact(WorldDay(day)),
                (None, Some(label)) => EventDate::Uncertain {
                    earliest: row.get::<_, Option<i64>>(8)?.map(WorldDay),
                    latest: row.get::<_, Option<i64>>(9)?.map(WorldDay),
                    label,
                },
                (None, None) => EventDate::Unknown,
                _ => return Err(rusqlite::Error::InvalidQuery),
            };
            ObjectData::HistoricalEvent { occurrence }
        }
        "source" => ObjectData::Source {
            citation: row.get(6)?,
        },
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(Version {
        object,
        recorded_at: AuthorRevision(row.get(0)?),
        record: Record {
            label: row.get(2)?,
            valid: ValidInterval {
                start: row.get::<_, Option<i64>>(3)?.map(WorldDay),
                end_exclusive: row.get::<_, Option<i64>>(4)?.map(WorldDay),
            },
            data,
            source: row.get::<_, Option<[u8; 16]>>(7)?.map(|bytes| ObjectRef {
                world: object.world,
                id: ObjectId(Uuid::from_bytes(bytes)),
            }),
        },
    })
}
