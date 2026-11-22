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
        };
        let mut statement = tx.prepare(&format!(
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
        let visible = |endpoint| {
            resolve_object(&tx, endpoint, view.branch, view.revision, view.day)
        };
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
        };
        let result = ResolvedFact {
            reference,
            recorded_at: AuthorRevision(row.get(7)?),
            source_branch: BranchId(Uuid::from_bytes(row.get(8)?)),
            rationale: row.get(9)?,
            data,
        };
        self.check_query_control()?;
        Ok(Some(result))
    }
}
