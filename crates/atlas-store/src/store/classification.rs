//! Canonical author-only curated classification persistence and bounded reads.
use super::*;
use crate::semantic::{Classification, ClassificationInput, ConceptProfile, DefinitionPin};

pub(super) const ACTIVE: &str = "
, active_classification AS (
    SELECT p.*,r.branch_id FROM concept_classifications p
    JOIN author_revisions r ON r.world_id=p.world_id AND r.revision=p.recorded_revision
    JOIN eligible l ON l.branch_id=r.branch_id
    WHERE p.world_id=?1 AND p.recorded_revision<=l.rev
      AND (l.depth=0 OR p.starts_at<=l.cutoff)
      AND NOT EXISTS (
        SELECT 1 FROM classification_withdrawals w JOIN eligible e ON e.branch_id=w.branch_id
        WHERE w.world_id=p.world_id AND w.fact_id=p.id
          AND e.depth<=l.depth AND w.revision<=e.rev
      )
)
";

/// Validate a curated endpoint in the same transaction as a network write.
/// A classification is evidence-backed metadata, not an inferred relation.
pub(super) fn has_profile(
    conn: &Connection,
    view: ViewContext,
    subject: ObjectRef,
    profile: ConceptProfile,
) -> Result<bool, Error> {
    let mut statement = conn.prepare(&format!(
        "{METADATA_CTE} {ACTIVE} SELECT source_id,descriptor_id
         FROM active_classification WHERE subject_id=?5 AND profile=?6
         AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4)
         ORDER BY id LIMIT 2"
    ))?;
    let mut rows = statement.query(params![
        view.world.0.as_bytes(),
        view.branch.0.as_bytes(),
        view.revision.0,
        view.day.0,
        subject.id.0.as_bytes(),
        profile.key()
    ])?;
    let Some(row) = rows.next()? else {
        return Ok(false);
    };
    let source = row.get::<_, [u8; 16]>(0)?;
    let descriptor = row.get::<_, [u8; 16]>(1)?;
    if rows.next()?.is_some() {
        return Err(Error::Integrity(
            "overlapping active classifications".into(),
        ));
    }
    for bytes in [source, descriptor] {
        let object = ObjectRef {
            world: view.world,
            id: ObjectId(Uuid::from_bytes(bytes)),
        };
        if resolve_object(conn, object, view.branch, view.revision, view.day)?.is_none() {
            return Ok(false);
        }
    }
    Ok(true)
}

impl Store {
    pub fn record_classification(
        &mut self,
        view: ViewContext,
        input: ClassificationInput,
        command: Uuid,
    ) -> Result<Uuid, Error> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = metadata_world(&tx, view, true)?;
        metadata_world(
            &tx,
            ViewContext {
                day: input.starts_at,
                ..view
            },
            true,
        )?;
        if input.ends_before.is_some_and(|end| end <= input.starts_at) {
            return Err(Error::InvalidInterval);
        }
        let pin = input.descriptor.version;
        for endpoint in [input.subject, input.source, pin.object] {
            if endpoint.world != view.world {
                return Err(Error::WrongWorld);
            }
        }
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
        if at(input.subject)?.record.data != ObjectData::Concept
            || !matches!(at(input.source)?.record.data, ObjectData::Source { .. })
        {
            return Err(Error::WrongKind);
        }
        let definition = at(pin.object)?;
        if !matches!(
            definition.record.data,
            ObjectData::Definition {
                definition: Definition::Tag,
                ..
            }
        ) {
            return Err(Error::WrongKind);
        }
        if definition.reference() != pin {
            return Err(Error::InvalidInput("definition version pin"));
        }
        let duplicate: bool = tx.query_row(
            &format!(
                "{METADATA_CTE} {ACTIVE} SELECT EXISTS(
                 SELECT 1 FROM active_classification WHERE subject_id=?5 AND profile=?6
                   AND (?7 IS NULL OR starts_at<?7) AND (ends_before IS NULL OR ends_before>?4))"
            ),
            params![
                view.world.0.as_bytes(),
                view.branch.0.as_bytes(),
                view.revision.0,
                input.starts_at.0,
                input.subject.id.0.as_bytes(),
                input.profile.key(),
                input.ends_before.map(|end| end.0)
            ],
            |row| row.get(0),
        )?;
        if duplicate {
            return Err(Error::InvalidInput("overlapping classification"));
        }
        let revision = append_revision(
            &tx,
            &world,
            view.branch,
            view.revision,
            command,
            ("Local author", "Record concept classification"),
        )?;
        let id = Uuid::new_v4();
        tx.execute(
            "INSERT INTO concept_classifications VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                view.world.0.as_bytes(),
                id.as_bytes(),
                input.subject.id.0.as_bytes(),
                input.profile.key(),
                pin.object.id.0.as_bytes(),
                pin.branch.0.as_bytes(),
                pin.state_revision.0,
                pin.recorded_at.0,
                input.source.id.0.as_bytes(),
                input.starts_at.0,
                input.ends_before.map(|end| end.0),
                revision
            ],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub fn withdraw_classification(
        &mut self,
        view: ViewContext,
        id: Uuid,
        command: Uuid,
    ) -> Result<(), Error> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = metadata_world(&tx, view, true)?;
        let exists: bool = tx.query_row(
            &format!("{METADATA_CTE} {ACTIVE} SELECT EXISTS(SELECT 1 FROM active_classification WHERE id=?5)"),
            params![
                view.world.0.as_bytes(),
                view.branch.0.as_bytes(),
                view.revision.0,
                view.day.0,
                id.as_bytes()
            ],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(Error::NotFound);
        }
        let revision = append_revision(
            &tx,
            &world,
            view.branch,
            view.revision,
            command,
            ("Local author", "Withdraw concept classification"),
        )?;
        tx.execute(
            "INSERT INTO classification_withdrawals VALUES(?1,?2,?3,?4)",
            params![
                view.world.0.as_bytes(),
                view.branch.0.as_bytes(),
                id.as_bytes(),
                revision
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Search a stable descriptor identity, keeping every result's original version pin.
    pub fn classifications_by_descriptor(
        &self,
        view: ViewContext,
        descriptor: ObjectRef,
    ) -> Result<Vec<Classification>, Error> {
        require_author(view.scope)?;
        if descriptor.world != view.world {
            return Err(Error::WrongWorld);
        }
        self.check_query_control()?;
        let tx = self.conn.unchecked_transaction()?;
        metadata_world(&tx, view, false)?;
        let Some(version) =
            resolve_object(&tx, descriptor, view.branch, view.revision, view.day)?
        else {
            return Ok(Vec::new());
        };
        if !matches!(
            version.record.data,
            ObjectData::Definition {
                definition: Definition::Tag,
                ..
            }
        ) {
            return Err(Error::WrongKind);
        }
        let mut statement = tx.prepare(&format!(
            "{METADATA_CTE} {ACTIVE}
             SELECT id,subject_id,profile,descriptor_id,descriptor_branch,descriptor_state_revision,
                    descriptor_revision,source_id,starts_at,ends_before,recorded_revision,branch_id
             FROM active_classification
             WHERE descriptor_id=?5 AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4)
             ORDER BY id LIMIT 2001"
        ))?;
        let mut rows = statement.query(params![
            view.world.0.as_bytes(),
            view.branch.0.as_bytes(),
            view.revision.0,
            view.day.0,
            descriptor.id.0.as_bytes()
        ])?;
        let mut result = Vec::new();
        let mut scanned = 0;
        while let Some(row) = rows.next()? {
            self.check_query_control()?;
            scanned += 1;
            if scanned > 2000 {
                return Err(Error::QueryBudgetExceeded);
            }
            if let Some(assignment) = read_classification(&tx, view, row)? {
                result.push(assignment);
            }
        }
        Ok(result)
    }

    /// Exact subject lookup or bounded profile discovery. No reader projection.
    pub fn classifications(
        &self,
        view: ViewContext,
        subject: Option<ObjectRef>,
        profile: Option<ConceptProfile>,
    ) -> Result<Vec<Classification>, Error> {
        require_author(view.scope)?;
        if subject.is_none() && profile.is_none() {
            return Err(Error::InvalidInput("classification filter"));
        }
        if subject.is_some_and(|object| object.world != view.world) {
            return Err(Error::WrongWorld);
        }
        self.check_query_control()?;
        let tx = self.conn.unchecked_transaction()?;
        metadata_world(&tx, view, false)?;
        let filter = match (subject, profile) {
            (Some(_), Some(_)) => "subject_id=?5 AND profile=?6",
            (Some(_), None) => "subject_id=?5 AND ?6 IS NULL",
            (None, Some(_)) => "?5 IS NULL AND profile=?6",
            (None, None) => unreachable!("filter required"),
        };
        let mut statement = tx.prepare(&format!(
            "{METADATA_CTE} {ACTIVE}
             SELECT id,subject_id,profile,descriptor_id,descriptor_branch,descriptor_state_revision,
                    descriptor_revision,source_id,starts_at,ends_before,recorded_revision,branch_id
             FROM active_classification
             WHERE {filter}
               AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4)
             ORDER BY id LIMIT 2001"
        ))?;
        let mut rows = statement.query(params![
            view.world.0.as_bytes(),
            view.branch.0.as_bytes(),
            view.revision.0,
            view.day.0,
            subject.map(|object| *object.id.0.as_bytes()),
            profile.map(ConceptProfile::key)
        ])?;
        let mut result = Vec::new();
        let mut scanned = 0;
        while let Some(row) = rows.next()? {
            self.check_query_control()?;
            scanned += 1;
            if scanned > 2000 {
                return Err(Error::QueryBudgetExceeded);
            }
            if let Some(assignment) = read_classification(&tx, view, row)? {
                result.push(assignment);
            }
        }
        Ok(result)
    }
}

/// Shared payload decoding and historical endpoint checks for point and list reads.
pub(super) fn read_classification(
    conn: &Connection,
    view: ViewContext,
    row: &rusqlite::Row<'_>,
) -> Result<Option<Classification>, Error> {
    let object = |bytes: [u8; 16]| ObjectRef {
        world: view.world,
        id: ObjectId(Uuid::from_bytes(bytes)),
    };
    let subject = object(row.get(1)?);
    let source = object(row.get(7)?);
    let descriptor = object(row.get(3)?);
    for endpoint in [subject, source, descriptor] {
        if resolve_object(conn, endpoint, view.branch, view.revision, view.day)?.is_none() {
            return Ok(None);
        }
    }
    Ok(Some(Classification {
        id: Uuid::from_bytes(row.get(0)?),
        input: ClassificationInput {
            subject,
            profile: ConceptProfile::parse(&row.get::<_, String>(2)?)?,
            descriptor: DefinitionPin {
                version: VersionRef {
                    object: descriptor,
                    branch: BranchId(Uuid::from_bytes(row.get(4)?)),
                    state_revision: AuthorRevision(row.get(5)?),
                    recorded_at: AuthorRevision(row.get(6)?),
                },
            },
            source,
            starts_at: WorldDay(row.get(8)?),
            ends_before: row.get::<_, Option<i64>>(9)?.map(WorldDay),
        },
        recorded_at: AuthorRevision(row.get(10)?),
        source_branch: BranchId(Uuid::from_bytes(row.get(11)?)),
    }))
}

/// Additional archive validation beyond foreign keys: references must retain
/// compatible canonical kinds and refer to a Tag definition from prior history.
pub(crate) fn verify_classification_integrity(conn: &Connection) -> Result<(), Error> {
    let invalid: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM concept_classifications c
            JOIN objects subject ON subject.world_id=c.world_id AND subject.id=c.subject_id
            JOIN objects source ON source.world_id=c.world_id AND source.id=c.source_id
            LEFT JOIN definition_versions d ON d.world_id=c.world_id
                AND d.definition_id=c.descriptor_id AND d.branch_id=c.descriptor_branch
                AND d.state_revision=c.descriptor_state_revision AND d.revision=c.descriptor_revision
            WHERE subject.kind<>'concept' OR source.kind<>'source'
                OR d.category IS NULL OR d.category<>'tag'
                OR c.descriptor_revision>=c.recorded_revision
        ) OR EXISTS(
            SELECT 1 FROM classification_withdrawals w
            JOIN concept_classifications c ON c.world_id=w.world_id AND c.id=w.fact_id
            WHERE w.revision<=c.recorded_revision
        )",
        [],
        |row| row.get(0),
    )?;
    if invalid {
        return Err(Error::Integrity("invalid historical classification".into()));
    }
    Ok(())
}
