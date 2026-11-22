//! Author annotation links reuse canonical Claim objects and stable fact UUIDs.
use super::*;
use crate::semantic::{FactAnnotation, FactRef, SemanticRef};

impl Store {
    pub fn annotate_semantic_fact(
        &mut self,
        view: ViewContext,
        fact: FactRef,
        claim: ObjectRef,
        command: Uuid,
    ) -> Result<Uuid, Error> {
        SemanticRef::Fact(fact).validate_author_context(view)?;
        SemanticRef::Object(claim).validate_author_context(view)?;
        // unchecked_transaction borrows the connection immutably, allowing
        // the shared resolver to run inside this same atomic write transaction.
        let tx = rusqlite::Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let world = metadata_world(&tx, view, true)?;
        if self.resolve_semantic_fact_in(&tx, fact, view)?.is_none() {
            return Err(Error::NotFound);
        }
        if !visible_sourced_claim(&tx, view, claim)? {
            return Err(Error::WrongKind);
        }
        let duplicate: bool = tx.query_row(
            &format!(
                "{METADATA_CTE} SELECT EXISTS(
                 SELECT 1 FROM fact_annotations a
                 JOIN author_revisions r ON r.world_id=a.world_id AND r.revision=a.recorded_revision
                 JOIN eligible l ON l.branch_id=r.branch_id
                 WHERE a.world_id=?1 AND a.family=?5 AND a.fact_id=?6 AND a.claim_id=?7
                 AND a.recorded_revision<=l.rev AND a.starts_at<=l.cutoff)"
            ),
            params![
                view.world.0.as_bytes(),
                view.branch.0.as_bytes(),
                view.revision.0,
                view.day.0,
                fact.family.key(),
                fact.id.as_bytes(),
                claim.id.0.as_bytes()
            ],
            |row| row.get(0),
        )?;
        if duplicate {
            return Err(Error::InvalidInput("duplicate fact annotation"));
        }
        let revision = append_revision(
            &tx,
            &world,
            view.branch,
            view.revision,
            command,
            ("Local author", "Link sourced fact annotation"),
        )?;
        let id = Uuid::new_v4();
        tx.execute(
            "INSERT INTO fact_annotations VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                view.world.0.as_bytes(),
                id.as_bytes(),
                fact.family.key(),
                fact.id.as_bytes(),
                claim.id.0.as_bytes(),
                view.day.0,
                revision
            ],
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub fn semantic_fact_annotations(
        &self,
        view: ViewContext,
        fact: FactRef,
    ) -> Result<Vec<FactAnnotation>, Error> {
        SemanticRef::Fact(fact).validate_author_context(view)?;
        self.check_query_control()?;
        let tx = self.conn.unchecked_transaction()?;
        metadata_world(&tx, view, false)?;
        if self.resolve_semantic_fact_in(&tx, fact, view)?.is_none() {
            return Ok(Vec::new());
        }
        let mut statement = tx.prepare(&format!(
            "{METADATA_CTE} SELECT a.id,a.claim_id,a.recorded_revision FROM fact_annotations a
             JOIN author_revisions r ON r.world_id=a.world_id AND r.revision=a.recorded_revision
             JOIN eligible l ON l.branch_id=r.branch_id
             WHERE a.world_id=?1 AND a.family=?5 AND a.fact_id=?6
             AND a.recorded_revision<=l.rev AND a.starts_at<=l.cutoff
             ORDER BY a.id LIMIT 2001"
        ))?;
        let mut rows = statement.query(params![
            view.world.0.as_bytes(),
            view.branch.0.as_bytes(),
            view.revision.0,
            view.day.0,
            fact.family.key(),
            fact.id.as_bytes()
        ])?;
        let mut result = Vec::new();
        let mut scanned = 0;
        while let Some(row) = rows.next()? {
            self.check_query_control()?;
            scanned += 1;
            if scanned > 2000 {
                return Err(Error::QueryBudgetExceeded);
            }
            let claim = ObjectRef {
                world: view.world,
                id: ObjectId(Uuid::from_bytes(row.get(1)?)),
            };
            if visible_sourced_claim(&tx, view, claim)? {
                result.push(FactAnnotation {
                    id: Uuid::from_bytes(row.get(0)?),
                    fact,
                    claim,
                    recorded_at: AuthorRevision(row.get(2)?),
                });
            }
        }
        Ok(result)
    }
}

pub(super) fn visible_sourced_claim(
    conn: &Connection,
    view: ViewContext,
    claim: ObjectRef,
) -> Result<bool, Error> {
    let Some(version) = resolve_object(conn, claim, view.branch, view.revision, view.day)? else {
        return Ok(false);
    };
    if !matches!(version.record.data, ObjectData::Claim { .. }) {
        return Ok(false);
    }
    let Some(source) = version.record.source else {
        return Ok(false);
    };
    Ok(
        resolve_object(conn, source, view.branch, view.revision, view.day)?
            .is_some_and(|version| matches!(version.record.data, ObjectData::Source { .. })),
    )
}

pub(crate) fn verify_annotation_integrity(conn: &Connection) -> Result<(), Error> {
    let invalid: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM fact_annotations a
            JOIN objects c ON c.world_id=a.world_id AND c.id=a.claim_id
            WHERE c.kind<>'claim'
            OR NOT (CASE a.family
                WHEN 'network' THEN EXISTS(SELECT 1 FROM network_facts f
                    WHERE f.world_id=a.world_id AND f.id=a.fact_id
                    AND f.recorded_revision<a.recorded_revision)
                WHEN 'parentage' THEN EXISTS(SELECT 1 FROM accepted_parentage f
                    WHERE f.world_id=a.world_id AND f.id=a.fact_id
                    AND f.recorded_revision<a.recorded_revision)
                WHEN 'classification' THEN EXISTS(SELECT 1 FROM concept_classifications f
                    WHERE f.world_id=a.world_id AND f.id=a.fact_id
                    AND f.recorded_revision<a.recorded_revision)
                WHEN 'site_area' THEN EXISTS(SELECT 1 FROM site_area_facts f
                    WHERE f.world_id=a.world_id AND f.id=a.fact_id
                    AND f.recorded_revision<a.recorded_revision)
                ELSE 0 END)
        ) OR EXISTS(
            SELECT 1 FROM descriptor_retirements d
            JOIN objects o ON o.world_id=d.world_id AND o.id=d.descriptor_id
            WHERE o.kind<>'definition'
        )",
        [],
        |row| row.get(0),
    )?;
    if invalid {
        return Err(Error::Integrity(
            "invalid semantic annotation or retirement".into(),
        ));
    }
    Ok(())
}
