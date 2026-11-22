//! Source-backed site measurements; containment and geometry remain independent.
use super::*;
use crate::semantic::{
    AreaReading, AreaTest, AreaUnit, DefinitionPin, SiteAreaFact, SiteAreaInput,
};

pub(super) const ACTIVE: &str = "
, active_site_area AS (
    SELECT p.*,r.branch_id FROM site_area_facts p
    JOIN author_revisions r ON r.world_id=p.world_id AND r.revision=p.recorded_revision
    JOIN eligible l ON l.branch_id=r.branch_id
    WHERE p.world_id=?1 AND p.recorded_revision<=l.rev
      AND (l.depth=0 OR p.starts_at<=l.cutoff)
      AND NOT EXISTS (
        SELECT 1 FROM site_area_withdrawals w JOIN eligible e ON e.branch_id=w.branch_id
        WHERE w.world_id=p.world_id AND w.fact_id=p.id
          AND e.depth<=l.depth AND w.revision<=e.rev
      )
)
";

impl Store {
    /// A correction replaces the same interval in author history atomically.
    /// A real change must be a separate nonoverlapping dated observation.
    pub fn record_site_area(
        &mut self,
        view: ViewContext,
        input: SiteAreaInput,
        corrects: Option<Uuid>,
        command: Uuid,
    ) -> Result<Uuid, Error> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = metadata_world(&tx, view, true)?;
        let effective = ViewContext {
            day: input.starts_at,
            ..view
        };
        metadata_world(&tx, effective, true)?;
        if input.ends_before.is_some_and(|end| end <= input.starts_at) {
            return Err(Error::InvalidInterval);
        }
        let (amount, unit) = match input.reading {
            AreaReading::Unknown => (None, None),
            AreaReading::Known { amount, unit } if amount >= 0 => (Some(amount), Some(unit.key())),
            _ => return Err(Error::InvalidInput("site area amount")),
        };
        let pin = input.descriptor.version;
        let at = |object| {
            require_visible(
                &tx,
                view.world,
                object,
                view.branch,
                view.revision.0,
                input.starts_at,
            )
        };
        if at(input.place)?.record.data != ObjectData::Place
            || !matches!(at(input.source)?.record.data, ObjectData::Source { .. })
        {
            return Err(Error::WrongKind);
        }
        let descriptor = at(pin.object)?;
        if !matches!(
            descriptor.record.data,
            ObjectData::Definition {
                definition: Definition::Tag,
                ..
            }
        ) {
            return Err(Error::WrongKind);
        }
        if descriptor.reference() != pin {
            return Err(Error::InvalidInput("definition version pin"));
        }
        if let Some(old) = corrects {
            let previous: Option<(i64, Option<i64>)> = tx.query_row(
                &format!("{METADATA_CTE} {ACTIVE} SELECT starts_at,ends_before FROM active_site_area WHERE id=?5 AND place_id=?6"),
                params![view.world.0.as_bytes(),view.branch.0.as_bytes(),view.revision.0,input.starts_at.0,old.as_bytes(),input.place.id.0.as_bytes()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            ).optional()?;
            if previous != Some((input.starts_at.0, input.ends_before.map(|day| day.0))) {
                return Err(Error::InvalidInput("site area correction interval"));
            }
        }
        let duplicate: bool = tx.query_row(
            &format!(
                "{METADATA_CTE} {ACTIVE} SELECT EXISTS(SELECT 1 FROM active_site_area
                WHERE place_id=?5 AND (?6 IS NULL OR starts_at<?6)
                AND (ends_before IS NULL OR ends_before>?4) AND (?7 IS NULL OR id<>?7))"
            ),
            params![
                view.world.0.as_bytes(),
                view.branch.0.as_bytes(),
                view.revision.0,
                input.starts_at.0,
                input.place.id.0.as_bytes(),
                input.ends_before.map(|day| day.0),
                corrects.map(|id| *id.as_bytes())
            ],
            |row| row.get(0),
        )?;
        if duplicate {
            return Err(Error::InvalidInput("overlapping site area"));
        }
        let revision = append_revision(
            &tx,
            &world,
            view.branch,
            view.revision,
            command,
            ("Local author", "Record source-backed site area"),
        )?;
        if let Some(old) = corrects {
            tx.execute(
                "INSERT INTO site_area_withdrawals VALUES(?1,?2,?3,?4)",
                params![
                    view.world.0.as_bytes(),
                    view.branch.0.as_bytes(),
                    old.as_bytes(),
                    revision
                ],
            )?;
        }
        let id = Uuid::new_v4();
        tx.execute(
            "INSERT INTO site_area_facts VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                view.world.0.as_bytes(),
                id.as_bytes(),
                input.place.id.0.as_bytes(),
                pin.object.id.0.as_bytes(),
                pin.branch.0.as_bytes(),
                pin.state_revision.0,
                pin.recorded_at.0,
                input.source.id.0.as_bytes(),
                amount,
                unit,
                input.starts_at.0,
                input.ends_before.map(|day| day.0),
                revision,
                corrects.map(|id| *id.as_bytes())
            ],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub fn site_area(
        &self,
        view: ViewContext,
        place: ObjectRef,
    ) -> Result<Option<SiteAreaFact>, Error> {
        require_author(view.scope)?;
        if place.world != view.world {
            return Err(Error::WrongWorld);
        }
        self.check_query_control()?;
        let tx = self.conn.unchecked_transaction()?;
        metadata_world(&tx, view, false)?;
        let mut statement = tx.prepare(&format!(
            "{METADATA_CTE} {ACTIVE}
             SELECT id,place_id,descriptor_id,descriptor_branch,descriptor_state_revision,
                    descriptor_revision,source_id,amount,unit,starts_at,ends_before,
                    recorded_revision,branch_id,corrects
             FROM active_site_area WHERE place_id=?5
             AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4)
             ORDER BY id LIMIT 2"
        ))?;
        let mut rows = statement.query(params![
            view.world.0.as_bytes(),
            view.branch.0.as_bytes(),
            view.revision.0,
            view.day.0,
            place.id.0.as_bytes()
        ])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let result = read_site_area(&tx, view, row)?;
        if rows.next()?.is_some() {
            return Err(Error::Integrity("overlapping site area".into()));
        }
        self.check_query_control()?;
        Ok(result)
    }

    pub(crate) fn site_area_matches(
        &self,
        view: ViewContext,
        test: AreaTest,
    ) -> Result<Vec<Version>, Error> {
        require_author(view.scope)?;
        test.validate()?;
        self.check_query_control()?;
        let tx = self.conn.unchecked_transaction()?;
        metadata_world(&tx, view, false)?;
        let (condition, amount, unit) = match test {
            AreaTest::Unknown => ("amount IS NULL AND ?5 IS NULL AND ?6 IS NULL", None, None),
            AreaTest::AtLeast { amount, unit } => {
                ("amount>=?5 AND unit=?6", Some(amount), Some(unit.key()))
            }
        };
        let mut statement = tx.prepare(&format!(
            "{METADATA_CTE} {ACTIVE}
             SELECT id,place_id,descriptor_id,descriptor_branch,descriptor_state_revision,
                    descriptor_revision,source_id,amount,unit,starts_at,ends_before,
                    recorded_revision,branch_id,corrects
             FROM active_site_area WHERE {condition}
             AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4)
             ORDER BY id LIMIT 2001"
        ))?;
        let mut rows = statement.query(params![
            view.world.0.as_bytes(),
            view.branch.0.as_bytes(),
            view.revision.0,
            view.day.0,
            amount,
            unit
        ])?;
        let mut scanned = 0;
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        while let Some(row) = rows.next()? {
            self.check_query_control()?;
            scanned += 1;
            if scanned > 2000 {
                return Err(Error::QueryBudgetExceeded);
            }
            if let Some(fact) = read_site_area(&tx, view, row)? {
                if !seen.insert(fact.input.place) {
                    return Err(Error::Integrity("overlapping site area".into()));
                }
                if let Some(version) =
                    resolve_object(&tx, fact.input.place, view.branch, view.revision, view.day)?
                {
                    result.push(version);
                }
            }
        }
        Ok(result)
    }
}

pub(super) fn read_site_area(
    conn: &Connection,
    view: ViewContext,
    row: &rusqlite::Row<'_>,
) -> Result<Option<SiteAreaFact>, Error> {
    let object = |bytes: [u8; 16]| ObjectRef {
        world: view.world,
        id: ObjectId(Uuid::from_bytes(bytes)),
    };
    let place = object(row.get(1)?);
    let descriptor = object(row.get(2)?);
    let source = object(row.get(6)?);
    for endpoint in [place, descriptor, source] {
        if resolve_object(conn, endpoint, view.branch, view.revision, view.day)?.is_none() {
            return Ok(None);
        }
    }
    let reading = match (
        row.get::<_, Option<i64>>(7)?,
        row.get::<_, Option<String>>(8)?,
    ) {
        (None, None) => AreaReading::Unknown,
        (Some(amount), Some(unit)) if amount >= 0 => AreaReading::Known {
            amount,
            unit: AreaUnit::parse(&unit)?,
        },
        _ => return Err(Error::Integrity("invalid site area value".into())),
    };
    Ok(Some(SiteAreaFact {
        id: Uuid::from_bytes(row.get(0)?),
        input: SiteAreaInput {
            place,
            descriptor: DefinitionPin {
                version: VersionRef {
                    object: descriptor,
                    branch: BranchId(Uuid::from_bytes(row.get(3)?)),
                    state_revision: AuthorRevision(row.get(4)?),
                    recorded_at: AuthorRevision(row.get(5)?),
                },
            },
            source,
            reading,
            starts_at: WorldDay(row.get(9)?),
            ends_before: row.get::<_, Option<i64>>(10)?.map(WorldDay),
        },
        recorded_at: AuthorRevision(row.get(11)?),
        source_branch: BranchId(Uuid::from_bytes(row.get(12)?)),
        corrects: row.get::<_, Option<[u8; 16]>>(13)?.map(Uuid::from_bytes),
    }))
}

pub(crate) fn verify_site_area_integrity(conn: &Connection) -> Result<(), Error> {
    let invalid: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM site_area_facts a
            JOIN objects place ON place.world_id=a.world_id AND place.id=a.place_id
            JOIN objects source ON source.world_id=a.world_id AND source.id=a.source_id
            LEFT JOIN definition_versions d ON d.world_id=a.world_id
                AND d.definition_id=a.descriptor_id AND d.branch_id=a.descriptor_branch
                AND d.state_revision=a.descriptor_state_revision AND d.revision=a.descriptor_revision
            WHERE place.kind<>'place' OR source.kind<>'source'
                OR d.category IS NULL OR d.category<>'tag'
                OR a.descriptor_revision>=a.recorded_revision
        ) OR EXISTS(
            SELECT 1 FROM site_area_facts a JOIN site_area_facts old
            ON old.world_id=a.world_id AND old.id=a.corrects
            WHERE old.place_id<>a.place_id OR old.starts_at<>a.starts_at
                OR old.ends_before IS NOT a.ends_before OR old.recorded_revision>=a.recorded_revision
        )",
        [],
        |row| row.get(0),
    )?;
    if invalid {
        return Err(Error::Integrity("invalid site area history".into()));
    }
    Ok(())
}
