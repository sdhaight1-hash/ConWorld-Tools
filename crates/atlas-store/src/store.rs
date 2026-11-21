use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use uuid::Uuid;

use crate::*;

const APPLICATION_ID: i64 = 0x4357_4c44; // CWLD
const SCHEMA_VERSION: i64 = 6;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        // SQLite requires an unchecked connection during the registry rebuild.
        // Check the new schema before committing and enable enforcement before returning the store.
        conn.pragma_update(None, "foreign_keys", false)?;
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
        }
        if version < 3 {
            tx.execute_batch(include_str!("../migrations/0003.sql"))?;
        }
        if version < 4 {
            tx.execute_batch(include_str!("../migrations/0004.sql"))?;
        }
        if version < 5 {
            tx.execute_batch(include_str!("../migrations/0005.sql"))?;
        }
        if version < 6 {
            tx.execute_batch(include_str!("../migrations/0006.sql"))?;
            verify_foreign_keys(&tx)?;
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        tx.commit()?;
        conn.pragma_update(None, "foreign_keys", true)?;
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

    pub fn register_calendar(
        &mut self,
        command: &CalendarCommand,
    ) -> Result<AuthorRevision, Error> {
        require_author(command.scope)?;
        command.rules.validate()?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = load_world(&tx, command.world)?;
        let revision = append_revision(
            &tx,
            &world,
            world.main,
            command.expected,
            command.command_id,
            (&command.author, &command.rationale),
        )?;
        let rules = &command.rules;
        let (leap_every, leap_month) = rules
            .leap
            .map_or((None, None), |leap| (Some(leap.every), Some(leap.month)));
        tx.execute(
            "INSERT INTO calendar_rules (world_id, calendar_id, revision, name, epoch_day, leap_every, leap_month)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![world.id.0.as_bytes(), command.calendar.0.as_bytes(), revision, rules.name, rules.epoch.0, leap_every, leap_month],
        )?;
        {
            let mut insert = tx.prepare(
                "INSERT INTO calendar_months (world_id, calendar_id, revision, month, length)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for (index, length) in rules.month_lengths.iter().enumerate() {
                insert.execute(params![
                    world.id.0.as_bytes(),
                    command.calendar.0.as_bytes(),
                    revision,
                    index + 1,
                    length
                ])?;
            }
        }
        for (index, era) in rules.eras.iter().enumerate() {
            tx.execute(
                "INSERT INTO calendar_eras (world_id, calendar_id, revision, name, position, year_one, descending)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![world.id.0.as_bytes(), command.calendar.0.as_bytes(), revision, era.name, index + 1, era.year_one, era.descending],
            )?;
        }
        tx.commit()?;
        Ok(AuthorRevision(revision))
    }

    pub fn calendar(
        &self,
        world: WorldId,
        id: CalendarId,
        revision: AuthorRevision,
        scope: Scope,
    ) -> Result<CalendarVersion, Error> {
        require_author(scope)?;
        let tx = self.conn.unchecked_transaction()?;
        let world = load_world(&tx, world)?;
        if revision < AuthorRevision(0) || revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        let version = read_calendar(&tx, world.id, id, revision)?;
        tx.commit()?;
        Ok(version)
    }

    pub fn branch_fork(
        &self,
        world: WorldId,
        branch: BranchId,
        revision: AuthorRevision,
        scope: Scope,
    ) -> Result<Option<Fork>, Error> {
        require_author(scope)?;
        let tx = self.conn.unchecked_transaction()?;
        let world = load_world(&tx, world)?;
        if revision < AuthorRevision(0) || revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        let fork = load_fork(&tx, world.id, branch, revision)?;
        tx.commit()?;
        Ok(fork)
    }

    pub fn fork(&mut self, command: &ForkCommand) -> Result<Fork, Error> {
        self.fork_from(command, None)
    }

    /// The supplied day must agree with the parent's exact event occurrence.
    pub fn fork_at_event(
        &mut self,
        command: &ForkCommand,
        event: ObjectRef,
    ) -> Result<Fork, Error> {
        if event.world != command.world {
            return Err(Error::WrongWorld);
        }
        self.fork_from(command, Some(event))
    }

    fn fork_from(
        &mut self,
        command: &ForkCommand,
        event: Option<ObjectRef>,
    ) -> Result<Fork, Error> {
        require_author(command.scope)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = load_world(&tx, command.world)?;
        if let Some(parent) = load_fork(&tx, world.id, command.parent, world.revision)?
            && command.day < parent.day
        {
            return Err(Error::BeforeFork);
        }
        let revision = append_revision(
            &tx,
            &world,
            command.parent,
            command.expected,
            command.id,
            (&command.author, &command.rationale),
        )?;
        let anchor = if let Some(event) = event {
            let origin = resolve_object(&tx, event, command.parent, world.revision, command.day)?
                .ok_or(Error::NotFound)?;
            match &origin.record.data {
                ObjectData::HistoricalEvent { occurrence }
                    if occurrence.exact_day() == Some(command.day) => {}
                ObjectData::HistoricalEvent { .. } => return Err(Error::InvalidOccurrence),
                _ => return Err(Error::WrongKind),
            }
            Some(origin.reference())
        } else {
            None
        };
        let child = BranchId(Uuid::new_v4());
        tx.execute(
            "INSERT INTO branches (world_id, id, is_main) VALUES (?1, ?2, 0)",
            params![world.id.0.as_bytes(), child.0.as_bytes()],
        )?;
        tx.execute(
            "INSERT INTO branch_forks (world_id, branch_id, parent_id, fork_day, inherited_revision, created_revision)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![world.id.0.as_bytes(), child.0.as_bytes(), command.parent.0.as_bytes(), command.day.0, world.revision.0, revision],
        )?;
        if let Some(event) = anchor {
            tx.execute(
                "INSERT INTO fork_anchors (world_id, child_branch, event_id, event_branch, event_state_revision, event_revision)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![world.id.0.as_bytes(), child.0.as_bytes(), event.object.id.0.as_bytes(), event.branch.0.as_bytes(), event.state_revision.0, event.recorded_at.0],
            )?;
        }
        tx.commit()?;
        Ok(Fork {
            id: child,
            parent: command.parent,
            day: command.day,
            inherited_revision: world.revision,
            created_revision: AuthorRevision(revision),
            anchor,
        })
    }

    pub fn apply(&mut self, command: &Command) -> Result<AuthorRevision, Error> {
        require_author(command.scope)?;
        if command.changes.is_empty() {
            return Err(Error::InvalidInput("empty command"));
        }

        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = load_world(&tx, command.world)?;
        let branch = command.branch;
        let fork = load_fork(&tx, world.id, branch, world.revision)?;
        let revision = append_revision(
            &tx,
            &world,
            branch,
            command.expected,
            command.id,
            (&command.author, &command.rationale),
        )?;
        let mut changed = HashSet::new();
        for change in &command.changes {
            let (object, record) = match change {
                Change::Create(object, record)
                | Change::Correct(object, record)
                | Change::RecordState(object, record) => (*object, record),
                Change::EventEffect { target, record, .. } => (*target, record),
            };
            if object.world != world.id {
                return Err(Error::WrongWorld);
            }
            if !changed.insert(object.id) {
                return Err(Error::RepeatedObject);
            }
            let effect = match change {
                Change::EventEffect { event, .. } => {
                    if event.world != object.world {
                        return Err(Error::WrongWorld);
                    }
                    if *event == object || record.source.is_none() {
                        return Err(Error::InvalidInput(
                            "event effect requires a distinct target and a source",
                        ));
                    }
                    let day = record.valid.start.ok_or(Error::InvalidInterval)?;
                    let origin =
                        resolve_object(&tx, *event, branch, AuthorRevision(revision), day)?
                            .ok_or(Error::NotFound)?;
                    match origin.record.data {
                        ObjectData::HistoricalEvent { ref occurrence }
                            if occurrence.exact_day().is_some_and(|on| on <= day) => {}
                        ObjectData::HistoricalEvent { .. } => return Err(Error::InvalidOccurrence),
                        _ => return Err(Error::WrongKind),
                    }
                    let before = resolve_object(&tx, object, branch, world.revision, day)?
                        .ok_or(Error::NotFound)?;
                    Some((origin.reference(), before.reference(), day))
                }
                _ => None,
            };
            apply_change(&tx, branch, fork, revision, change, object, record)?;
            if let Some((event, before, day)) = effect {
                tx.execute(
                    "INSERT INTO historical_effects (world_id, target_id, branch_id, revision,
                         event_id, event_branch, event_state_revision, event_revision,
                         before_branch, before_state_revision, before_revision, day)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        object.world.0.as_bytes(),
                        object.id.0.as_bytes(),
                        branch.0.as_bytes(),
                        revision,
                        event.object.id.0.as_bytes(),
                        event.branch.0.as_bytes(),
                        event.state_revision.0,
                        event.recorded_at.0,
                        before.branch.0.as_bytes(),
                        before.state_revision.0,
                        before.recorded_at.0,
                        day.0
                    ],
                )?;
            }
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
        if view.revision < AuthorRevision(0) || view.revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        let version = resolve_object(&tx, object, view.branch, view.revision, view.day)?;
        tx.commit()?;
        Ok(version)
    }

    pub fn effect_on(
        &self,
        target: ObjectRef,
        view: ViewContext,
    ) -> Result<Option<HistoricalEffect>, Error> {
        let Some(after) = self
            .get_object(target, view)?
            .map(|version| version.reference())
        else {
            return Ok(None);
        };
        // Corrections retain the provenance of the state they update.
        Ok(self.conn.query_row(
            "SELECT event_id, event_branch, event_state_revision, event_revision,
                    before_branch, before_state_revision, before_revision, day
             FROM historical_effects WHERE world_id = ?1 AND target_id = ?2 AND branch_id = ?3 AND revision = ?4",
            params![target.world.0.as_bytes(), target.id.0.as_bytes(), after.branch.0.as_bytes(), after.state_revision.0],
            |row| Ok(HistoricalEffect {
                event: VersionRef {
                    object: ObjectRef { world: target.world, id: ObjectId(Uuid::from_bytes(row.get(0)?)) },
                    branch: BranchId(Uuid::from_bytes(row.get(1)?)),
                    state_revision: AuthorRevision(row.get(2)?),
                    recorded_at: AuthorRevision(row.get(3)?),
                },
                before: VersionRef {
                    object: target, branch: BranchId(Uuid::from_bytes(row.get(4)?)),
                    state_revision: AuthorRevision(row.get(5)?),
                    recorded_at: AuthorRevision(row.get(6)?),
                },
                after: VersionRef { recorded_at: after.state_revision, ..after },
                day: WorldDay(row.get(7)?),
            }),
        ).optional()?)
    }

    pub fn verify_foreign_keys(&self, scope: Scope) -> Result<(), Error> {
        require_author(scope)?;
        verify_foreign_keys(&self.conn)
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

fn read_calendar(
    conn: &Connection,
    world: WorldId,
    id: CalendarId,
    revision: AuthorRevision,
) -> Result<CalendarVersion, Error> {
    let (registered_at, mut rules) = conn
        .query_row(
            "SELECT revision, name, epoch_day, leap_every, leap_month FROM calendar_rules
             WHERE world_id = ?1 AND calendar_id = ?2 AND revision <= ?3
             ORDER BY revision DESC LIMIT 1",
            params![world.0.as_bytes(), id.0.as_bytes(), revision.0],
            |row| {
                let leap = match (row.get(3)?, row.get(4)?) {
                    (Some(every), Some(month)) => Some(LeapCycle { every, month }),
                    (None, None) => None,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                Ok((
                    AuthorRevision(row.get(0)?),
                    CalendarRules {
                        name: row.get(1)?,
                        epoch: WorldDay(row.get(2)?),
                        month_lengths: Vec::new(),
                        leap,
                        eras: Vec::new(),
                    },
                ))
            },
        )
        .optional()?
        .ok_or(Error::NotFound)?;
    {
        let mut months = conn.prepare(
            "SELECT month, length FROM calendar_months
                 WHERE world_id = ?1 AND calendar_id = ?2 AND revision = ?3 ORDER BY month",
        )?;
        let mut rows = months.query(params![
            world.0.as_bytes(),
            id.0.as_bytes(),
            registered_at.0
        ])?;
        while let Some(row) = rows.next()? {
            if row.get::<_, usize>(0)? != rules.month_lengths.len() + 1 {
                return Err(Error::InvalidCalendar);
            }
            rules.month_lengths.push(row.get(1)?);
        }
    }
    let mut eras = conn.prepare(
        "SELECT name, year_one, descending FROM calendar_eras
         WHERE world_id = ?1 AND calendar_id = ?2 AND revision = ?3 ORDER BY position",
    )?;
    rules.eras = eras
        .query_map(
            params![world.0.as_bytes(), id.0.as_bytes(), registered_at.0],
            |row| {
                Ok(CalendarEra {
                    name: row.get(0)?,
                    year_one: row.get(1)?,
                    descending: row.get(2)?,
                })
            },
        )?
        .collect::<rusqlite::Result<_>>()?;
    rules.validate()?;
    Ok(CalendarVersion {
        id,
        registered_at,
        rules,
    })
}

fn append_revision(
    tx: &Transaction<'_>,
    world: &World,
    branch: BranchId,
    expected: AuthorRevision,
    id: Uuid,
    details: (&str, &str),
) -> Result<i64, Error> {
    validate_text(details.0, "command author")?;
    validate_text(details.1, "command rationale")?;
    if let Some(revision) = tx
        .query_row(
            "SELECT revision FROM author_revisions WHERE world_id = ?1 AND command_id = ?2",
            params![world.id.0.as_bytes(), id.as_bytes()],
            |row| row.get(0),
        )
        .optional()?
    {
        return Err(Error::DuplicateCommand(AuthorRevision(revision)));
    }
    if expected != world.revision {
        return Err(Error::RevisionConflict {
            expected,
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
        params![world.id.0.as_bytes(), revision, branch.0.as_bytes(), id.as_bytes(), details.0, details.1],
    )?;
    Ok(revision)
}

fn load_fork(
    conn: &Connection,
    world: WorldId,
    branch: BranchId,
    at: AuthorRevision,
) -> Result<Option<Fork>, Error> {
    conn.query_row(
        "SELECT f.parent_id, f.fork_day, f.inherited_revision, f.created_revision,
                a.event_id, a.event_branch, a.event_state_revision, a.event_revision
         FROM branches b LEFT JOIN branch_forks f ON f.world_id = b.world_id AND f.branch_id = b.id
         LEFT JOIN fork_anchors a ON a.world_id = f.world_id AND a.child_branch = f.branch_id
         WHERE b.world_id = ?1 AND b.id = ?2 AND (b.is_main = 1 OR f.created_revision <= ?3)",
        params![world.0.as_bytes(), branch.0.as_bytes(), at.0],
        |row| match row.get::<_, Option<[u8; 16]>>(0)? {
            Some(parent) => Ok(Some(Fork {
                id: branch,
                parent: BranchId(Uuid::from_bytes(parent)),
                day: WorldDay(row.get(1)?),
                inherited_revision: AuthorRevision(row.get(2)?),
                created_revision: AuthorRevision(row.get(3)?),
                anchor: match row.get::<_, Option<[u8; 16]>>(4)? {
                    Some(event) => Some(VersionRef {
                        object: ObjectRef {
                            world,
                            id: ObjectId(Uuid::from_bytes(event)),
                        },
                        branch: BranchId(Uuid::from_bytes(row.get(5)?)),
                        state_revision: AuthorRevision(row.get(6)?),
                        recorded_at: AuthorRevision(row.get(7)?),
                    }),
                    None => None,
                },
            })),
            None => Ok(None),
        },
    )
    .optional()?
    .ok_or(Error::NotFound)
}

fn override_start(
    conn: &Connection,
    object: ObjectRef,
    branch: BranchId,
    at: AuthorRevision,
) -> Result<Option<WorldDay>, Error> {
    Ok(conn
        .query_row(
            "SELECT starts_at FROM branch_overrides
         WHERE world_id = ?1 AND branch_id = ?2 AND object_id = ?3 AND revision <= ?4",
            params![
                object.world.0.as_bytes(),
                branch.0.as_bytes(),
                object.id.0.as_bytes(),
                at.0
            ],
            |row| row.get(0).map(WorldDay),
        )
        .optional()?)
}

fn resolve_object(
    conn: &Connection,
    object: ObjectRef,
    mut branch: BranchId,
    mut revision: AuthorRevision,
    requested_day: WorldDay,
) -> Result<Option<Version>, Error> {
    let mut day = requested_day;
    let mut cutoff = None;
    while let Some(fork) = load_fork(conn, object.world, branch, revision)? {
        if day >= fork.day
            && override_start(conn, object, branch, revision)?.is_some_and(|start| start <= day)
        {
            break;
        }
        cutoff = Some(cutoff.map_or(fork.day, |bound: WorldDay| bound.min(fork.day)));
        day = day.min(fork.day);
        branch = fork.parent;
        revision = fork.inherited_revision;
    }
    let version = local_object_at(conn, object, branch, revision, day)?;
    Ok(version.filter(|version| {
        version
            .record
            .valid
            .end_exclusive
            .is_none_or(|end| requested_day < end)
            && cutoff.is_none_or(|bound| match &version.record.data {
                ObjectData::HistoricalEvent { occurrence } => match occurrence {
                    EventDate::Exact(day) | EventDate::CalendarExact { day, .. } => *day <= bound,
                    EventDate::Uncertain { latest, .. } => latest.is_some_and(|day| day <= bound),
                    EventDate::Unknown => false,
                },
                _ => true,
            })
    }))
}

fn local_object_at(
    conn: &Connection,
    object: ObjectRef,
    branch: BranchId,
    revision: AuthorRevision,
    day: WorldDay,
) -> Result<Option<Version>, Error> {
    Ok(conn
        .query_row(
            "SELECT revision, kind, label, valid_start, valid_end, event_day, citation, source_id,
                    earliest_day, latest_day, date_label, state_revision
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
            params![
                object.world.0.as_bytes(),
                object.id.0.as_bytes(),
                branch.0.as_bytes(),
                revision.0,
                day.0
            ],
            |row| read_version(conn, row, object, branch),
        )
        .optional()?)
}

fn written_on_lineage(
    conn: &Connection,
    source: ObjectRef,
    mut branch: BranchId,
    mut revision: AuthorRevision,
) -> Result<bool, Error> {
    loop {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM object_versions
             WHERE world_id = ?1 AND object_id = ?2 AND branch_id = ?3 AND revision <= ?4)",
            params![
                source.world.0.as_bytes(),
                source.id.0.as_bytes(),
                branch.0.as_bytes(),
                revision.0
            ],
            |row| row.get(0),
        )?;
        if exists {
            return Ok(true);
        }
        let Some(fork) = load_fork(conn, source.world, branch, revision)? else {
            return Ok(false);
        };
        branch = fork.parent;
        revision = fork.inherited_revision;
    }
}

fn apply_change(
    tx: &Transaction<'_>,
    branch: BranchId,
    fork: Option<Fork>,
    revision: i64,
    change: &Change,
    object: ObjectRef,
    record: &Record,
) -> Result<(), Error> {
    let prior_revision = AuthorRevision(revision - 1);
    let existing = latest_state(tx, object, branch, None)?;
    let mut new_override = None;
    let mut local_boundary = None;
    if let Some(fork) = fork {
        let current = override_start(tx, object, branch, prior_revision)?;
        let boundary = current.unwrap_or(fork.day);
        local_boundary = Some(boundary);
        if record.valid.start.is_some_and(|day| day < boundary)
            || record
                .valid
                .end_exclusive
                .is_some_and(|day| day <= boundary)
        {
            return Err(Error::BeforeFork);
        }
        let inherited = if current.is_none() && !matches!(change, Change::Create(..)) {
            Some(
                resolve_object(tx, object, branch, prior_revision, fork.day)?
                    .ok_or(Error::NotFound)?,
            )
        } else {
            None
        };
        if let ObjectData::HistoricalEvent { occurrence } = &record.data {
            let previous = existing
                .as_ref()
                .map(|(_, version)| version)
                .or(inherited.as_ref());
            let occurrence_unchanged =
                previous.is_some_and(|version| version.record.data == record.data);
            let may_predate_fork = match occurrence {
                EventDate::Exact(day) | EventDate::CalendarExact { day, .. } => *day < fork.day,
                EventDate::Uncertain { earliest, .. } => earliest.is_none_or(|day| day < fork.day),
                EventDate::Unknown => false,
            };
            if may_predate_fork && !occurrence_unchanged {
                return Err(Error::BeforeFork);
            }
        }
        if current.is_none() {
            new_override = Some(match change {
                Change::RecordState(..) | Change::EventEffect { .. } => {
                    record.valid.start.ok_or(Error::InvalidInterval)?
                }
                _ => fork.day,
            });
        }
    }
    let state_revision = match (change, existing) {
        (Change::Create(..), _) => revision,
        (Change::Correct(..), Some((state, _))) => {
            if let Some((_, previous)) = latest_state(tx, object, branch, Some(state))?
                && !matches!(
                    (previous.record.valid.end_exclusive, record.valid.start),
                    (Some(end), Some(start)) if end <= start
                )
            {
                return Err(Error::InvalidInterval);
            }
            state
        }
        (Change::RecordState(..) | Change::EventEffect { .. }, Some((state, mut previous))) => {
            let start = record.valid.start.ok_or(Error::InvalidInterval)?;
            if previous
                .record
                .valid
                .start
                .or(local_boundary)
                .is_some_and(|day| day >= start)
            {
                return Err(Error::InvalidInterval);
            }
            if previous
                .record
                .valid
                .end_exclusive
                .is_none_or(|day| day > start)
            {
                previous.record.valid.end_exclusive = Some(start);
                write_version(tx, branch, revision, state, object, &previous.record, false)?;
            }
            revision
        }
        (_, None) if new_override.is_some() => revision,
        (_, None) => return Err(Error::NotFound),
    };
    write_version(
        tx,
        branch,
        revision,
        state_revision,
        object,
        record,
        matches!(change, Change::Create(..)),
    )?;
    if let Some(start) = new_override {
        tx.execute(
            "INSERT INTO branch_overrides (world_id, branch_id, object_id, revision, starts_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                object.world.0.as_bytes(),
                branch.0.as_bytes(),
                object.id.0.as_bytes(),
                revision,
                start.0
            ],
        )?;
    }
    Ok(())
}

fn verify_foreign_keys(conn: &Connection) -> Result<(), Error> {
    let violation = conn
        .query_row("PRAGMA foreign_key_check", [], |row| row.get(0))
        .optional()?;
    match violation {
        Some(table) => Err(Error::Integrity(table)),
        None => Ok(()),
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
            |row| Ok((row.get(11)?, read_version(conn, row, object, branch)?)),
        )
        .optional()?)
}

fn require_visible(
    conn: &Connection,
    world: WorldId,
    reference: ObjectRef,
    branch: BranchId,
    revision: i64,
    day: WorldDay,
) -> Result<Version, Error> {
    if reference.world != world { return Err(Error::WrongWorld); }
    resolve_object(conn, reference, branch, AuthorRevision(revision), day)?.ok_or(Error::NotFound)
}

fn require_person(
    conn: &Connection, world: WorldId, reference: ObjectRef,
    branch: BranchId, revision: i64, day: WorldDay,
) -> Result<(), Error> {
    match require_visible(conn, world, reference, branch, revision, day)?.record.data {
        ObjectData::Person => Ok(()),
        _ => Err(Error::WrongKind),
    }
}

fn require_event(
    conn: &Connection, world: WorldId, reference: ObjectRef,
    branch: BranchId, revision: i64, day: WorldDay,
) -> Result<(), Error> {
    match require_visible(conn, world, reference, branch, revision, day)?.record.data {
        ObjectData::HistoricalEvent { occurrence }
            if occurrence.exact_day().is_some_and(|at| at <= day) => Ok(()),
        ObjectData::HistoricalEvent { .. } => Err(Error::InvalidOccurrence),
        _ => Err(Error::WrongKind),
    }
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
        ObjectData::Relationship { subject, target, kind, trust, tags } => {
            let day = record.valid.start.ok_or(Error::InvalidInterval)?;
            if subject == target { return Err(Error::InvalidInput("self-relationship")); }
            validate_text(kind, "relationship kind")?;
            if trust.is_some_and(|value| value > 100) {
                return Err(Error::InvalidInput("trust must be between 0 and 100"));
            }
            let mut seen = HashSet::new();
            for tag in tags {
                validate_text(tag, "relationship tag")?;
                if !seen.insert(tag.trim().to_lowercase()) {
                    return Err(Error::InvalidInput("duplicate relationship tag"));
                }
            }
            require_person(tx, object.world, *subject, branch, revision, day)?;
            require_person(tx, object.world, *target, branch, revision, day)?;
            ("relationship", &EventDate::Unknown, None)
        }
        ObjectData::EventInvolvement { event, participant, .. } => {
            let day = record.valid.start.ok_or(Error::InvalidInterval)?;
            require_event(tx, object.world, *event, branch, revision, day)?;
            require_person(tx, object.world, *participant, branch, revision, day)?;
            ("event_involvement", &EventDate::Unknown, None)
        }
        ObjectData::ActorAction { event, actor, target, action, intent, .. } => {
            let day = record.valid.start.ok_or(Error::InvalidInterval)?;
            validate_text(action, "action kind")?;
            validate_text(intent, "action intention")?;
            require_event(tx, object.world, *event, branch, revision, day)?;
            require_person(tx, object.world, *actor, branch, revision, day)?;
            if let Some(target) = target {
                require_visible(tx, object.world, *target, branch, revision, day)?;
            }
            ("actor_action", &EventDate::Unknown, None)
        },
        ObjectData::Claim {
            subject,
            predicate,
            statement,
        } => {
            validate_text(predicate, "claim predicate")?;
            validate_text(statement, "claim statement")?;
            if subject.world != object.world {
                return Err(Error::WrongWorld);
            }
            if *subject == object || record.source.is_none() {
                return Err(Error::InvalidInput(
                    "claim requires a distinct subject and an asserting source",
                ));
            }
            if !written_on_lineage(tx, *subject, branch, AuthorRevision(revision))? {
                return Err(Error::NotFound);
            }
            ("claim", &EventDate::Unknown, None)
        }
        ObjectData::HistoricalEvent { occurrence } => ("historical_event", occurrence, None),
        ObjectData::Source { citation } => {
            validate_text(citation, "source citation")?;
            ("source", &EventDate::Unknown, Some(citation.as_str()))
        }
    };
    let (event_day, earliest, latest, date_label) = match occurrence {
        EventDate::Exact(day) | EventDate::CalendarExact { day, .. } => {
            (Some(day.0), None, None, None)
        }
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
    if let EventDate::CalendarExact {
        day,
        calendar,
        calendar_revision,
        date,
        label,
    } = occurrence
    {
        validate_text(label, "exact calendar date label")?;
        if calendar_revision.0 >= revision {
            return Err(Error::InvalidRevision);
        }
        let version = read_calendar(tx, object.world, *calendar, *calendar_revision)?;
        if version.registered_at != *calendar_revision
            || version
                .rules
                .normalize(CalendarDate::Day(*date), "")?
                .exact_day()
                != Some(*day)
        {
            return Err(Error::InvalidCalendarDate);
        }
    }
    if let Some(source) = record.source {
        if source.world != object.world {
            return Err(Error::WrongWorld);
        }
        if source == object {
            return Err(Error::InvalidInput("self-citation"));
        }
        if !written_on_lineage(tx, source, branch, AuthorRevision(revision))? {
            return Err(Error::NotFound);
        }
        if kind_of(tx, source)?.as_deref() != Some("source") {
            return Err(Error::WrongKind);
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
    if let ObjectData::Relationship { subject, target, kind, trust, tags } = &record.data {
        tx.execute(
            "INSERT INTO relationship_versions
             (world_id, relationship_id, branch_id, revision, state_revision, subject_id, target_id, relation_kind, trust)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(), revision,
                state_revision, subject.id.0.as_bytes(), target.id.0.as_bytes(), kind, trust.map(i64::from)],
        )?;
        for tag in tags {
            tx.execute(
                "INSERT INTO relationship_tags
                 (world_id, relationship_id, branch_id, state_revision, revision, tag)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(),
                    state_revision, revision, tag.trim()],
            )?;
        }
    }
    if let ObjectData::EventInvolvement { event, participant, role } = &record.data {
        let role = match role {
            EventRole::Actor => "actor",
            EventRole::Witness => "witness",
            EventRole::Victim => "victim",
            EventRole::Stakeholder => "stakeholder",
            EventRole::Sponsor => "sponsor",
        };
        tx.execute(
            "INSERT INTO event_involvement_versions
             (world_id, involvement_id, branch_id, state_revision, revision, event_id, participant_id, role)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(),
                state_revision, revision, event.id.0.as_bytes(), participant.id.0.as_bytes(), role],
        )?;
    }
    if let ObjectData::ActorAction { event, actor, target, action, intent, outcome } = &record.data {
        let outcome = match outcome {
            ActionOutcome::Attempted => "attempted",
            ActionOutcome::Succeeded => "succeeded",
            ActionOutcome::Failed => "failed",
        };
        tx.execute(
            "INSERT INTO actor_action_versions
             (world_id, action_id, branch_id, state_revision, revision, event_id, actor_id, target_id, action_kind, intent, outcome)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(),
                state_revision, revision, event.id.0.as_bytes(), actor.id.0.as_bytes(),
                target.map(|refid| *refid.id.0.as_bytes()), action, intent, outcome],
        )?;
    }
    if let ObjectData::Claim {
        subject,
        predicate,
        statement,
    } = &record.data
    {
        tx.execute(
            "INSERT INTO claim_versions (world_id, claim_id, branch_id, revision, state_revision, subject_id, predicate, statement)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(),
                revision, state_revision, subject.id.0.as_bytes(), predicate, statement],
        )?;
    }
    if let EventDate::CalendarExact {
        calendar,
        calendar_revision,
        date,
        label,
        ..
    } = occurrence
    {
        tx.execute(
            "INSERT INTO event_calendar_dates (world_id, event_id, branch_id, revision, state_revision,
                 calendar_id, calendar_revision, year, month, day, original_label)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(), revision,
                state_revision, calendar.0.as_bytes(), calendar_revision.0, date.year, date.month, date.day, label],
        )?;
    }
    Ok(())
}

fn read_version(
    conn: &Connection,
    row: &rusqlite::Row<'_>,
    object: ObjectRef,
    branch: BranchId,
) -> rusqlite::Result<Version> {
    let data = match row.get::<_, String>(1)?.as_str() {
        "person" => ObjectData::Person,
        "relationship" => {
            let (subject, target, kind, trust): ([u8; 16], [u8; 16], String, Option<u8>) = conn.query_row(
                "SELECT subject_id, target_id, relation_kind, trust FROM relationship_versions
                 WHERE world_id = ?1 AND relationship_id = ?2 AND branch_id = ?3 AND revision = ?4 AND state_revision = ?5",
                params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(),
                    row.get::<_, i64>(0)?, row.get::<_, i64>(11)?],
                |relation| Ok((relation.get(0)?, relation.get(1)?, relation.get(2)?, relation.get(3)?)),
            )?;
            let mut stmt = conn.prepare(
                "SELECT tag FROM relationship_tags WHERE world_id = ?1 AND relationship_id = ?2
                 AND branch_id = ?3 AND revision = ?4 AND state_revision = ?5 ORDER BY tag",
            )?;
            let tags = stmt.query_map(params![object.world.0.as_bytes(), object.id.0.as_bytes(),
                branch.0.as_bytes(), row.get::<_, i64>(0)?, row.get::<_, i64>(11)?], |tag| tag.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;
            ObjectData::Relationship {
                subject: ObjectRef { world: object.world, id: ObjectId(Uuid::from_bytes(subject)) },
                target: ObjectRef { world: object.world, id: ObjectId(Uuid::from_bytes(target)) },
                kind, trust, tags,
            }
        }
        "event_involvement" => conn.query_row(
            "SELECT event_id, participant_id, role FROM event_involvement_versions
             WHERE world_id = ?1 AND involvement_id = ?2 AND branch_id = ?3 AND revision = ?4 AND state_revision = ?5",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(),
                row.get::<_, i64>(0)?, row.get::<_, i64>(11)?],
            |entry| {
                let role = match entry.get::<_, String>(2)?.as_str() {
                    "actor" => EventRole::Actor,
                    "witness" => EventRole::Witness,
                    "victim" => EventRole::Victim,
                    "stakeholder" => EventRole::Stakeholder,
                    "sponsor" => EventRole::Sponsor,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                Ok(ObjectData::EventInvolvement {
                    event: ObjectRef { world: object.world, id: ObjectId(Uuid::from_bytes(entry.get(0)?)) },
                    participant: ObjectRef { world: object.world, id: ObjectId(Uuid::from_bytes(entry.get(1)?)) },
                    role,
                })
            },
        )?,
        "actor_action" => conn.query_row(
            "SELECT event_id, actor_id, target_id, action_kind, intent, outcome FROM actor_action_versions
             WHERE world_id = ?1 AND action_id = ?2 AND branch_id = ?3 AND revision = ?4 AND state_revision = ?5",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(),
                row.get::<_, i64>(0)?, row.get::<_, i64>(11)?],
            |entry| {
                let outcome = match entry.get::<_, String>(5)?.as_str() {
                    "attempted" => ActionOutcome::Attempted,
                    "succeeded" => ActionOutcome::Succeeded,
                    "failed" => ActionOutcome::Failed,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                Ok(ObjectData::ActorAction {
                    event: ObjectRef { world: object.world, id: ObjectId(Uuid::from_bytes(entry.get(0)?)) },
                    actor: ObjectRef { world: object.world, id: ObjectId(Uuid::from_bytes(entry.get(1)?)) },
                    target: entry.get::<_, Option<[u8; 16]>>(2)?.map(|id| ObjectRef {
                        world: object.world, id: ObjectId(Uuid::from_bytes(id)),
                    }),
                    action: entry.get(3)?, intent: entry.get(4)?, outcome,
                })
            },
        )?,
        "claim" => conn.query_row(
            "SELECT subject_id, predicate, statement FROM claim_versions
             WHERE world_id = ?1 AND claim_id = ?2 AND branch_id = ?3 AND revision = ?4 AND state_revision = ?5",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(), row.get::<_, i64>(0)?, row.get::<_, i64>(11)?],
            |claim| Ok(ObjectData::Claim {
                subject: ObjectRef { world: object.world, id: ObjectId(Uuid::from_bytes(claim.get(0)?)) },
                predicate: claim.get(1)?,
                statement: claim.get(2)?,
            }),
        )?,
        "historical_event" => {
            let occurrence = match (
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<String>>(10)?,
            ) {
                (Some(day), None) => conn.query_row(
                    "SELECT calendar_id, calendar_revision, year, month, day, original_label
                     FROM event_calendar_dates WHERE world_id = ?1 AND event_id = ?2 AND branch_id = ?3
                        AND revision = ?4 AND state_revision = ?5",
                    params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(), row.get::<_, i64>(0)?, row.get::<_, i64>(11)?],
                    |calendar| Ok(EventDate::CalendarExact {
                        day: WorldDay(day),
                        calendar: CalendarId(Uuid::from_bytes(calendar.get(0)?)),
                        calendar_revision: AuthorRevision(calendar.get(1)?),
                        date: CalendarDay { year: calendar.get(2)?, month: calendar.get(3)?, day: calendar.get(4)? },
                        label: calendar.get(5)?,
                    }),
                ).optional()?.unwrap_or(EventDate::Exact(WorldDay(day))),
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
        source_branch: branch,
        state_revision: AuthorRevision(row.get(11)?),
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
