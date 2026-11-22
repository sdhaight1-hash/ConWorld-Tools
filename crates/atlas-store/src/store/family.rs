use super::*;
use crate::genealogy::{FamilyLink, ParentageKind, RecordParentage};

fn author_world(conn: &Connection, view: ViewContext) -> Result<World, Error> {
    require_author(view.scope)?;
    let world = load_world(conn, view.world)?;
    if view.branch != world.main {
        return Err(Error::CapabilityDenied);
    }
    if view.revision.0 < 0 || view.revision > world.revision {
        return Err(Error::InvalidRevision);
    }
    Ok(world)
}

impl Store {
    /// Record an explicitly accepted, dated link. Validate all overlapping days.
    pub fn record_parentage(
        &mut self,
        view: ViewContext,
        input: RecordParentage<'_>,
    ) -> Result<Uuid, Error> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = author_world(&tx, view)?;
        for person in [input.parent, input.child] {
            require_person(
                &tx,
                view.world,
                person,
                view.branch,
                view.revision.0,
                view.day,
            )?;
            if input.starts_at != view.day {
                require_person(
                    &tx,
                    view.world,
                    person,
                    view.branch,
                    view.revision.0,
                    input.starts_at,
                )?;
            }
        }
        if input.parent == input.child {
            return Err(Error::InvalidInput("self parentage"));
        }
        if input.ends_before.is_some_and(|end| end <= input.starts_at) {
            return Err(Error::InvalidInterval);
        }
        let kind = match input.kind {
            ParentageKind::Biological => "biological",
            ParentageKind::Adoptive => "adoptive",
            ParentageKind::Legal => "legal",
        };
        let rev = append_revision(
            &tx,
            &world,
            world.main,
            view.revision,
            input.command_id,
            ("Local author", input.rationale),
        )?;
        let w = view.world.0.as_bytes();
        let parent = input.parent.id.0.as_bytes();
        let child = input.child.id.0.as_bytes();
        let lo = input.starts_at.0;
        let hi = input.ends_before.map_or(i64::MAX, |end| end.0 - 1);
        let duplicate: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM accepted_parentage WHERE world_id=?1 AND parent_id=?2 AND child_id=?3
             AND kind=?4 AND withdrawn_revision IS NULL AND starts_at<=?6 AND (ends_before IS NULL OR ends_before>?5))",
            params![w,parent,child,kind,lo,hi], |r| r.get(0))?;
        if duplicate {
            return Err(Error::InvalidInput("overlapping accepted parentage"));
        }
        // Carry the intersection along each path. Opposite links in disjoint
        // historical periods do not form an ancestry cycle.
        let (cycle, work): (bool, usize) = tx.query_row(
            "WITH RECURSIVE descendants(person,lo,hi) AS (
               VALUES(?2,?3,?4) UNION
               SELECT p.child_id,max(d.lo,p.starts_at),min(d.hi,coalesce(p.ends_before-1,9223372036854775807))
               FROM descendants d JOIN accepted_parentage p ON p.world_id=?1 AND p.parent_id=d.person
               WHERE p.withdrawn_revision IS NULL AND p.starts_at<=d.hi AND (p.ends_before IS NULL OR p.ends_before>d.lo)
               LIMIT 10001)
             SELECT max(person=?5),count(*) FROM descendants",
            params![w,child,lo,hi,parent], |r| Ok((r.get(0)?,r.get(1)?)))?;
        if cycle {
            return Err(Error::InvalidInput("accepted ancestry cycle"));
        }
        if work > 10_000 {
            return Err(Error::QueryBudgetExceeded);
        }
        let id = Uuid::new_v4();
        tx.execute("INSERT INTO accepted_parentage(world_id,id,parent_id,child_id,kind,starts_at,ends_before,recorded_revision)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![w,id.as_bytes(),parent,child,kind,lo,input.ends_before.map(|v|v.0),rev])?;
        tx.commit()?;
        Ok(id)
    }

    pub fn withdraw_parentage(
        &mut self,
        view: ViewContext,
        person: ObjectRef,
        link: Uuid,
        command: Uuid,
        rationale: &str,
    ) -> Result<(), Error> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = author_world(&tx, view)?;
        require_person(
            &tx,
            view.world,
            person,
            view.branch,
            view.revision.0,
            view.day,
        )?;
        let rev = append_revision(
            &tx,
            &world,
            world.main,
            view.revision,
            command,
            ("Local author", rationale),
        )?;
        if tx.execute("UPDATE accepted_parentage SET withdrawn_revision=?3 WHERE world_id=?1 AND id=?2 AND withdrawn_revision IS NULL AND (parent_id=?4 OR child_id=?4)",
            params![view.world.0.as_bytes(),link.as_bytes(),rev,person.id.0.as_bytes()])? != 1 { return Err(Error::NotFound); }
        tx.commit()?;
        Ok(())
    }

    pub fn family_links(
        &self,
        view: ViewContext,
        person: ObjectRef,
    ) -> Result<Vec<FamilyLink>, Error> {
        let tx = self.conn.unchecked_transaction()?;
        let (revision, cutoff) = inherited_metadata(&tx, view)?;
        require_person(
            &tx,
            view.world,
            person,
            view.branch,
            view.revision.0,
            view.day,
        )?;
        let mut stmt=tx.prepare("SELECT p.id,parent_id,child_id,kind,starts_at,ends_before,r.rationale
           FROM accepted_parentage p JOIN author_revisions r ON r.world_id=p.world_id AND r.revision=p.recorded_revision
           WHERE p.world_id=?1 AND (parent_id=?2 OR child_id=?2) AND recorded_revision<=?3
             AND (withdrawn_revision IS NULL OR withdrawn_revision>?3) AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?5)
           ORDER BY kind,starts_at,p.id LIMIT 2001")?;
        let mut rows = stmt.query(params![
            view.world.0.as_bytes(),
            person.id.0.as_bytes(),
            revision.0,
            cutoff.0,
            view.day.0
        ])?;
        let mut result = Vec::new();
        let mut count = 0;
        while let Some(row) = rows.next()? {
            count += 1;
            if count > 2000 {
                return Err(Error::QueryBudgetExceeded);
            }
            let endpoint = |bytes| ObjectRef {
                world: view.world,
                id: ObjectId(Uuid::from_bytes(bytes)),
            };
            let parent = endpoint(row.get(1)?);
            let child = endpoint(row.get(2)?);
            let other = if parent == person { child } else { parent };
            let Some(relative) = resolve_object(&tx, other, view.branch, view.revision, view.day)?
            else {
                continue;
            };
            let kind = match row.get::<_, String>(3)?.as_str() {
                "biological" => ParentageKind::Biological,
                "adoptive" => ParentageKind::Adoptive,
                "legal" => ParentageKind::Legal,
                _ => return Err(Error::Integrity("unknown parentage kind".into())),
            };
            result.push(FamilyLink {
                id: Uuid::from_bytes(row.get(0)?),
                parent,
                child,
                kind,
                starts_at: row.get(4)?,
                ends_before: row.get(5)?,
                relative_label: relative.record.label,
                rationale: row.get(6)?,
            });
        }
        Ok(result)
    }
}
