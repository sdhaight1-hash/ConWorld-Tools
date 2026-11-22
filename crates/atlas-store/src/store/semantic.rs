//! Point resolution of existing historical facts; no additional fact table.
use super::*;
use crate::{
    genealogy::{Parentage, ParentageKind},
    network::{NetworkInput, NetworkKind},
    semantic::{FactFamily, FactRef, ResolvedFact, SemanticFactData, SemanticRef},
};

impl Store {
    /// Resolve by world and family-specific stable ID, never by adjacency scan.
    pub fn resolve_semantic_fact(
        &self,
        reference: FactRef,
        view: ViewContext,
    ) -> Result<Option<ResolvedFact>, Error> {
        SemanticRef::Fact(reference).validate_author_context(view)?;
        self.check_query_control()?;
        let tx = self.conn.unchecked_transaction()?;
        metadata_world(&tx, view, false)?;
        self.resolve_semantic_fact_in(&tx, reference, view)
    }

    pub(super) fn resolve_semantic_fact_in(
        &self,
        conn: &Connection,
        reference: FactRef,
        view: ViewContext,
    ) -> Result<Option<ResolvedFact>, Error> {
        use super::classification::{ACTIVE, read_classification};
        self.check_query_control()?;
        if reference.family == FactFamily::Classification {
            let mut statement = conn.prepare(&format!(
                "{METADATA_CTE} {ACTIVE}
                 SELECT p.id,p.subject_id,p.profile,p.descriptor_id,p.descriptor_branch,
                        p.descriptor_state_revision,p.descriptor_revision,p.source_id,p.starts_at,
                        p.ends_before,p.recorded_revision,p.branch_id,r.rationale
                 FROM active_classification p JOIN author_revisions r
                 ON r.world_id=p.world_id AND r.revision=p.recorded_revision
                 WHERE p.id=?5 AND p.starts_at<=?4 AND (p.ends_before IS NULL OR p.ends_before>?4)"
            ))?;
            let mut rows = statement.query(params![
                view.world.0.as_bytes(),
                view.branch.0.as_bytes(),
                view.revision.0,
                view.day.0,
                reference.id.as_bytes()
            ])?;
            let Some(row) = rows.next()? else {
                return Ok(None);
            };
            let Some(assignment) = read_classification(conn, view, row)? else {
                return Ok(None);
            };
            self.check_query_control()?;
            return Ok(Some(ResolvedFact {
                reference,
                recorded_at: assignment.recorded_at,
                source_branch: assignment.source_branch,
                rationale: row.get(12)?,
                corrects: None,
                data: SemanticFactData::Classification(assignment.input),
            }));
        }
        if reference.family == FactFamily::SiteArea {
            use super::site_area::{ACTIVE as AREA, read_site_area};
            let mut statement = conn.prepare(&format!(
                "{METADATA_CTE} {AREA}
                 SELECT p.id,p.place_id,p.descriptor_id,p.descriptor_branch,p.descriptor_state_revision,
                        p.descriptor_revision,p.source_id,p.amount,p.unit,p.starts_at,p.ends_before,
                        p.recorded_revision,p.branch_id,p.corrects,r.rationale
                 FROM active_site_area p JOIN author_revisions r
                 ON r.world_id=p.world_id AND r.revision=p.recorded_revision
                 WHERE p.id=?5 AND p.starts_at<=?4 AND (p.ends_before IS NULL OR p.ends_before>?4)"
            ))?;
            let mut rows = statement.query(params![
                view.world.0.as_bytes(),
                view.branch.0.as_bytes(),
                view.revision.0,
                view.day.0,
                reference.id.as_bytes()
            ])?;
            let Some(row) = rows.next()? else {
                return Ok(None);
            };
            let Some(measurement) = read_site_area(conn, view, row)? else {
                return Ok(None);
            };
            self.check_query_control()?;
            return Ok(Some(ResolvedFact {
                reference,
                recorded_at: measurement.recorded_at,
                source_branch: measurement.source_branch,
                rationale: row.get(14)?,
                corrects: measurement.corrects,
                data: SemanticFactData::SiteArea(measurement.input),
            }));
        }
        let select = match reference.family {
            FactFamily::Network => {
                "SELECT p.subject_id,p.target_id,p.source_id,p.kind,p.role,p.starts_at,
                 p.ends_before,p.recorded_revision,r.branch_id,r.rationale
                 FROM active_network p JOIN author_revisions r
                 ON r.world_id=p.world_id AND r.revision=p.recorded_revision"
            }
            FactFamily::Parentage => {
                "SELECT p.parent_id,p.child_id,NULL,p.kind,NULL,p.starts_at,
                 p.ends_before,p.recorded_revision,r.branch_id,r.rationale
                 FROM active_parentage p JOIN author_revisions r
                 ON r.world_id=p.world_id AND r.revision=p.recorded_revision"
            }
            FactFamily::Classification | FactFamily::SiteArea => {
                unreachable!("handled by typed fact resolver")
            }
        };
        let mut statement = conn.prepare(&format!(
            "{METADATA_CTE} {select}
             WHERE p.id=?5 AND p.starts_at<=?4 AND (p.ends_before IS NULL OR p.ends_before>?4)"
        ))?;
        let mut rows = statement.query(params![
            view.world.0.as_bytes(),
            view.branch.0.as_bytes(),
            view.revision.0,
            view.day.0,
            reference.id.as_bytes()
        ])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let object = |bytes: [u8; 16]| ObjectRef {
            world: view.world,
            id: ObjectId(Uuid::from_bytes(bytes)),
        };
        let subject = object(row.get(0)?);
        let target = object(row.get(1)?);
        let visible =
            |endpoint| resolve_object(conn, endpoint, view.branch, view.revision, view.day);
        let (Some(subject_version), Some(target_version)) = (visible(subject)?, visible(target)?)
        else {
            return Ok(None);
        };
        let kind: String = row.get(3)?;
        let starts_at: i64 = row.get(5)?;
        let ends_before: Option<i64> = row.get(6)?;
        let data = match reference.family {
            FactFamily::Network => {
                let source = object(row.get(2)?);
                if visible(source)?.is_none() {
                    return Ok(None);
                }
                SemanticFactData::Network(NetworkInput {
                    subject,
                    target,
                    source,
                    kind: NetworkKind::parse(&kind)?,
                    role: row.get(4)?,
                    starts_at,
                    ends_before,
                })
            }
            FactFamily::Parentage => {
                if subject_version.record.data != ObjectData::Person
                    || target_version.record.data != ObjectData::Person
                {
                    return Err(Error::WrongKind);
                }
                let kind = match kind.as_str() {
                    "biological" => ParentageKind::Biological,
                    "adoptive" => ParentageKind::Adoptive,
                    "legal" => ParentageKind::Legal,
                    _ => return Err(Error::Integrity("unknown parentage kind".into())),
                };
                SemanticFactData::Parentage(Parentage {
                    parent: subject,
                    child: target,
                    kind,
                    valid: ValidInterval {
                        start: Some(WorldDay(starts_at)),
                        end_exclusive: ends_before.map(WorldDay),
                    },
                    accepted: true,
                })
            }
            FactFamily::Classification | FactFamily::SiteArea => {
                unreachable!("handled by typed fact resolver")
            }
        };
        let result = ResolvedFact {
            reference,
            recorded_at: AuthorRevision(row.get(7)?),
            source_branch: BranchId(Uuid::from_bytes(row.get(8)?)),
            rationale: row.get(9)?,
            corrects: None,
            data,
        };
        self.check_query_control()?;
        Ok(Some(result))
    }

    /// Author-only stable fact backlinks, including source and descriptor links.
    /// Resolve every candidate within one historical transaction before returning it.
    pub fn semantic_fact_backlinks(
        &self,
        target: ObjectRef,
        view: ViewContext,
    ) -> Result<Vec<ResolvedFact>, Error> {
        use super::{classification::ACTIVE, site_area::ACTIVE as AREA};
        SemanticRef::Object(target).validate_author_context(view)?;
        self.check_query_control()?;
        let tx = self.conn.unchecked_transaction()?;
        metadata_world(&tx, view, false)?;
        if resolve_object(&tx, target, view.branch, view.revision, view.day)?.is_none() {
            return Ok(Vec::new());
        }
        let annotation_target = super::annotation::visible_sourced_claim(&tx, view, target)?;
        let mut statement = tx.prepare(&format!(
            "{METADATA_CTE} {ACTIVE} {AREA}
             SELECT 0 AS family,id FROM active_network
               WHERE (subject_id=?5 OR target_id=?5 OR source_id=?5)
               AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4)
             UNION ALL
             SELECT 1,id FROM active_parentage
               WHERE (parent_id=?5 OR child_id=?5)
               AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4)
             UNION ALL
             SELECT 2,id FROM active_classification
               WHERE (subject_id=?5 OR source_id=?5 OR descriptor_id=?5)
               AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4)
             UNION ALL
             SELECT 3,id FROM active_site_area
               WHERE (place_id=?5 OR source_id=?5 OR descriptor_id=?5)
               AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4)
             UNION
             SELECT CASE a.family
                 WHEN 'network' THEN 0 WHEN 'parentage' THEN 1
                 WHEN 'classification' THEN 2 WHEN 'site_area' THEN 3 END,a.fact_id
               FROM fact_annotations a
               JOIN author_revisions r ON r.world_id=a.world_id AND r.revision=a.recorded_revision
               JOIN eligible l ON l.branch_id=r.branch_id
               WHERE a.world_id=?1 AND a.claim_id=?5 AND ?6
               AND a.recorded_revision<=l.rev AND a.starts_at<=l.cutoff
             ORDER BY family,id LIMIT 2001"
        ))?;
        let mut rows = statement.query(params![
            view.world.0.as_bytes(),
            view.branch.0.as_bytes(),
            view.revision.0,
            view.day.0,
            target.id.0.as_bytes(),
            annotation_target
        ])?;
        let mut result = Vec::new();
        let mut scanned = 0;
        while let Some(row) = rows.next()? {
            self.check_query_control()?;
            scanned += 1;
            if scanned > 2000 {
                return Err(Error::QueryBudgetExceeded);
            }
            let family = match row.get::<_, i64>(0)? {
                0 => FactFamily::Network,
                1 => FactFamily::Parentage,
                2 => FactFamily::Classification,
                3 => FactFamily::SiteArea,
                _ => return Err(Error::Integrity("unknown semantic fact family".into())),
            };
            let reference = FactRef {
                world: view.world,
                family,
                id: Uuid::from_bytes(row.get(1)?),
            };
            if let Some(fact) = self.resolve_semantic_fact_in(&tx, reference, view)? {
                result.push(fact);
            }
        }
        Ok(result)
    }
}
