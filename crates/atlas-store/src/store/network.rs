use super::*;
use crate::network::{GhostProposal, IdentityKind, NetworkFact, NetworkInput, NetworkKind};

fn author_context(conn: &Connection, view: ViewContext) -> Result<World, Error> {
    require_author(view.scope)?;
    let world = load_world(conn, view.world)?;
    if world.main != view.branch {
        return Err(Error::CapabilityDenied);
    }
    if view.revision.0 < 0 || view.revision > world.revision {
        return Err(Error::InvalidRevision);
    }
    Ok(world)
}

fn institution(data: &ObjectData) -> bool {
    matches!(
        data,
        ObjectData::Organization
            | ObjectData::Household
            | ObjectData::FamilyHouse
            | ObjectData::Faction
    )
}

fn bounded_text(value: &str) -> Result<(), Error> {
    if value.trim().is_empty() || value.len() > 800 {
        return Err(Error::InvalidInput("network text"));
    }
    Ok(())
}

fn ref_id(world: WorldId, bytes: [u8; 16]) -> ObjectRef {
    ObjectRef {
        world,
        id: ObjectId(Uuid::from_bytes(bytes)),
    }
}

fn resolved(conn: &Connection, view: ViewContext, object: ObjectRef) -> Result<Version, Error> {
    if object.world != view.world {
        return Err(Error::WrongWorld);
    }
    resolve_object(conn, object, view.branch, view.revision, view.day)?.ok_or(Error::NotFound)
}

impl Store {
    pub fn create_identity(
        &mut self,
        view: ViewContext,
        kind: IdentityKind,
        label: &str,
        command_id: Uuid,
    ) -> Result<ObjectRef, Error> {
        metadata_world(&self.conn, view, true)?;
        bounded_text(label)?;
        let object = ObjectRef::new(view.world);
        self.apply(&Command {
            world: view.world,
            branch: view.branch,
            expected: view.revision,
            id: command_id,
            author: "Local author".into(),
            rationale: "Create typed identity".into(),
            scope: Scope::Author,
            changes: vec![Change::Create(
                object,
                Record {
                    label: label.trim().into(),
                    valid: ValidInterval {
                        start: Some(view.day),
                        end_exclusive: None,
                    },
                    data: kind.data(),
                    source: None,
                },
            )],
        })?;
        Ok(object)
    }

    pub fn record_network_fact(
        &mut self,
        view: ViewContext,
        input: &NetworkInput,
        command_id: Uuid,
    ) -> Result<Uuid, Error> {
        bounded_text(&input.role)?;
        if input.subject == input.target
            || input.ends_before.is_some_and(|end| end <= input.starts_at)
        {
            return Err(Error::InvalidInterval);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = metadata_world(&tx, view, true)?;
        let effective = ViewContext {
            day: WorldDay(input.starts_at),
            ..view
        };
        metadata_world(&tx, effective, true)?;
        let subject = resolved(&tx, effective, input.subject)?;
        let target = resolved(&tx, effective, input.target)?;
        let source = resolved(&tx, effective, input.source)?;
        if !matches!(source.record.data, ObjectData::Source { .. }) {
            return Err(Error::WrongKind);
        }
        match input.kind {
            NetworkKind::CultureLanguage | NetworkKind::WorkingMaterial
                if subject.record.data == ObjectData::Concept
                    && target.record.data == ObjectData::Concept =>
            {
                use crate::semantic::ConceptProfile;
                let (subject_profile, target_profile, role) = match input.kind {
                    NetworkKind::CultureLanguage => (
                        ConceptProfile::Culture,
                        ConceptProfile::Language,
                        "documented_language",
                    ),
                    NetworkKind::WorkingMaterial => (
                        ConceptProfile::Technology,
                        ConceptProfile::Material,
                        "working_medium",
                    ),
                    _ => unreachable!(),
                };
                if input.role != role {
                    return Err(Error::InvalidInput("semantic relation role"));
                }
                if !super::classification::has_profile(
                    &tx,
                    effective,
                    input.subject,
                    subject_profile,
                )? || !super::classification::has_profile(
                    &tx,
                    effective,
                    input.target,
                    target_profile,
                )? {
                    return Err(Error::WrongKind);
                }
            }
            NetworkKind::HeirDesignation
                if matches!(subject.record.data, ObjectData::Person)
                    && matches!(target.record.data, ObjectData::Title) => {}
            NetworkKind::IdentityRedirect
                if subject.record.data == target.record.data
                    && matches!(
                        subject.record.data,
                        ObjectData::Person
                            | ObjectData::Organization
                            | ObjectData::Household
                            | ObjectData::FamilyHouse
                            | ObjectData::Faction
                            | ObjectData::Conflict
                            | ObjectData::Office
                            | ObjectData::Title
                            | ObjectData::Asset
                            | ObjectData::Population
                            | ObjectData::ConflictIssue
                            | ObjectData::ConflictPhase
                            | ObjectData::SuccessionRule
                            | ObjectData::Concept
                    ) => {}
            NetworkKind::Alliance
                if matches!(subject.record.data, ObjectData::Faction)
                    && matches!(target.record.data, ObjectData::Faction) => {}
            NetworkKind::Settlement
                if matches!(subject.record.data, ObjectData::Faction)
                    && matches!(
                        target.record.data,
                        ObjectData::Conflict | ObjectData::ConflictIssue
                    ) => {}
            NetworkKind::Membership
                if matches!(subject.record.data, ObjectData::Person)
                    && institution(&target.record.data) => {}
            NetworkKind::Marriage
                if matches!(subject.record.data, ObjectData::Person)
                    && matches!(target.record.data, ObjectData::Person) => {}
            NetworkKind::Succession
                if institution(&subject.record.data) && institution(&target.record.data) => {}
            NetworkKind::OfficeHolding
                if matches!(subject.record.data, ObjectData::Person)
                    && matches!(target.record.data, ObjectData::Office) => {}
            NetworkKind::OfficeOf
                if matches!(subject.record.data, ObjectData::Office)
                    && institution(&target.record.data) => {}
            NetworkKind::TitleHolding
                if matches!(subject.record.data, ObjectData::Person)
                    && matches!(target.record.data, ObjectData::Title) => {}
            NetworkKind::TitleOf
                if matches!(subject.record.data, ObjectData::Title)
                    && institution(&target.record.data) => {}
            NetworkKind::Ownership
                if (matches!(subject.record.data, ObjectData::Person)
                    || institution(&subject.record.data))
                    && matches!(target.record.data, ObjectData::Asset) => {}
            NetworkKind::IssueOf
                if matches!(subject.record.data, ObjectData::ConflictIssue)
                    && matches!(target.record.data, ObjectData::Conflict) => {}
            NetworkKind::PhaseOf
                if matches!(subject.record.data, ObjectData::ConflictPhase)
                    && matches!(target.record.data, ObjectData::Conflict) => {}
            NetworkKind::PopulationMember
                if matches!(subject.record.data, ObjectData::Person)
                    && matches!(target.record.data, ObjectData::Population) => {}
            NetworkKind::RuleOf
                if matches!(subject.record.data, ObjectData::SuccessionRule)
                    && matches!(target.record.data, ObjectData::Title) =>
            {
                if !matches!(
                    input.role.as_str(),
                    "elective"
                        | "primogeniture"
                        | "absolute_primogeniture"
                        | "matrilineal"
                        | "designated_heir"
                        | "custom"
                ) {
                    return Err(Error::InvalidInput("succession rule kind"));
                }
            }
            NetworkKind::ConflictPosition
                if matches!(subject.record.data, ObjectData::Faction)
                    && matches!(
                        target.record.data,
                        ObjectData::Conflict | ObjectData::ConflictIssue
                    ) =>
            {
                if !matches!(
                    input.role.as_str(),
                    "supports" | "opposes" | "mediates" | "neutral"
                ) {
                    return Err(Error::InvalidInput("conflict stance"));
                }
            }
            NetworkKind::ConfirmedCause | NetworkKind::AllegedCause => {
                let (
                    ObjectData::HistoricalEvent { occurrence: cause },
                    ObjectData::HistoricalEvent { occurrence: effect },
                ) = (&subject.record.data, &target.record.data)
                else {
                    return Err(Error::WrongKind);
                };
                if input.kind == NetworkKind::ConfirmedCause {
                    let (Some(a), Some(b)) = (cause.exact_day(), effect.exact_day()) else {
                        return Err(Error::InvalidInput(
                            "confirmed causes require exact event dates",
                        ));
                    };
                    if a > b {
                        return Err(Error::InvalidInput("invalid causal order"));
                    }
                }
            }
            _ => return Err(Error::WrongKind),
        }
        let kind = input.kind.name();
        let duplicate: bool = tx.query_row(
            &format!("{METADATA_CTE} SELECT EXISTS(SELECT 1 FROM active_network WHERE kind=?5
             AND ((subject_id=?6 AND target_id=?7) OR (?5 IN ('marriage','alliance') AND subject_id=?7 AND target_id=?6))
             AND (?8 IS NULL OR starts_at<?8) AND (ends_before IS NULL OR ends_before>?4))"),
            params![view.world.0.as_bytes(),view.branch.0.as_bytes(),view.revision.0,input.starts_at,
                kind,input.subject.id.0.as_bytes(),input.target.id.0.as_bytes(),input.ends_before], |row| row.get(0))?;
        if duplicate {
            return Err(Error::InvalidInput("overlapping network fact"));
        }
        // An office/title represents one seat, and a title has one active rule.
        // A multi-seat body uses multiple office identities.
        if matches!(
            input.kind,
            NetworkKind::OfficeHolding
                | NetworkKind::TitleHolding
                | NetworkKind::RuleOf
                | NetworkKind::HeirDesignation
        ) {
            let occupied: bool = tx.query_row(
                &format!("{METADATA_CTE} SELECT EXISTS(SELECT 1 FROM active_network WHERE kind=?5
                    AND target_id=?6 AND (?7 IS NULL OR starts_at<?7) AND (ends_before IS NULL OR ends_before>?4))"),
                params![view.world.0.as_bytes(),view.branch.0.as_bytes(),view.revision.0,input.starts_at,
                    kind,input.target.id.0.as_bytes(),input.ends_before], |row| row.get(0))?;
            if occupied {
                return Err(Error::InvalidInput("overlapping exclusive role"));
            }
        }
        if input.kind == NetworkKind::IdentityRedirect {
            let occupied:bool=tx.query_row(
                &format!("{METADATA_CTE} SELECT EXISTS(SELECT 1 FROM active_network WHERE kind='identity_redirect'
                 AND subject_id=?5 AND (?6 IS NULL OR starts_at<?6) AND (ends_before IS NULL OR ends_before>?4))"),
                params![view.world.0.as_bytes(),view.branch.0.as_bytes(),view.revision.0,input.starts_at,input.subject.id.0.as_bytes(),input.ends_before],
                |row|row.get(0))?;
            if occupied {
                return Err(Error::InvalidInput("overlapping identity redirects"));
            }
        }
        // Conservative whole-history DAG: never allow a confirmed cause or succession cycle.
        if matches!(
            input.kind,
            NetworkKind::ConfirmedCause | NetworkKind::Succession | NetworkKind::IdentityRedirect
        ) {
            let mut edges = HashMap::<ObjectRef, Vec<ObjectRef>>::new();
            let mut stmt = tx.prepare(&format!("{METADATA_CTE} SELECT subject_id,target_id FROM active_network WHERE kind=?5 LIMIT 4001"))?;
            let mut rows = stmt.query(params![
                view.world.0.as_bytes(),
                view.branch.0.as_bytes(),
                view.revision.0,
                input.starts_at,
                kind
            ])?;
            let mut count = 0;
            while let Some(row) = rows.next()? {
                count += 1;
                if count > 4000 {
                    return Err(Error::QueryBudgetExceeded);
                }
                edges
                    .entry(ref_id(view.world, row.get(0)?))
                    .or_default()
                    .push(ref_id(view.world, row.get(1)?));
            }
            edges.entry(input.subject).or_default().push(input.target);
            if !crate::graph::acyclic(&edges) {
                return Err(Error::InvalidInput("network cycle"));
            }
        }
        let rev = append_revision(
            &tx,
            &world,
            view.branch,
            view.revision,
            command_id,
            ("Local author", "Record explicit network fact"),
        )?;
        let id = Uuid::new_v4();
        tx.execute("INSERT INTO network_facts(world_id,id,subject_id,target_id,source_id,kind,role,starts_at,ends_before,recorded_revision)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![view.world.0.as_bytes(),id.as_bytes(),input.subject.id.0.as_bytes(),input.target.id.0.as_bytes(),
                input.source.id.0.as_bytes(),kind,input.role,input.starts_at,input.ends_before,rev])?;
        tx.commit()?;
        Ok(id)
    }

    pub fn withdraw_network_fact(
        &mut self,
        view: ViewContext,
        id: Uuid,
        command_id: Uuid,
    ) -> Result<(), Error> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = metadata_world(&tx, view, true)?;
        let exists: bool = tx.query_row(
            &format!("{METADATA_CTE} SELECT EXISTS(SELECT 1 FROM active_network WHERE id=?5)"),
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
        let rev = append_revision(
            &tx,
            &world,
            view.branch,
            view.revision,
            command_id,
            ("Local author", "Withdraw network fact"),
        )?;
        tx.execute("INSERT INTO metadata_withdrawals(world_id,branch_id,kind,fact_id,revision) VALUES(?1,?2,'network',?3,?4)",
            params![view.world.0.as_bytes(),view.branch.0.as_bytes(),id.as_bytes(),rev])?;
        tx.commit()?;
        Ok(())
    }

    pub fn network_facts(
        &self,
        view: ViewContext,
        focus: ObjectRef,
    ) -> Result<Vec<NetworkFact>, Error> {
        metadata_world(&self.conn, view, false)?;
        resolved(&self.conn, view, focus)?;
        let mut stmt=self.conn.prepare(&format!("{METADATA_CTE} SELECT id,subject_id,target_id,source_id,kind,role,starts_at,ends_before FROM active_network WHERE (subject_id=?5 OR target_id=?5) AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4) ORDER BY id LIMIT 501"))?;
        let mut rows = stmt.query(params![
            view.world.0.as_bytes(),
            view.branch.0.as_bytes(),
            view.revision.0,
            view.day.0,
            focus.id.0.as_bytes()
        ])?;
        let mut result = Vec::new();
        let mut count = 0;
        while let Some(row) = rows.next()? {
            self.check_query_control()?;
            count += 1;
            if count > 500 {
                return Err(Error::QueryBudgetExceeded);
            }
            let subject = ref_id(view.world, row.get(1)?);
            let target = ref_id(view.world, row.get(2)?);
            let source = ref_id(view.world, row.get(3)?);
            let Some(a) =
                resolve_object(&self.conn, subject, view.branch, view.revision, view.day)?
            else {
                continue;
            };
            let Some(b) = resolve_object(&self.conn, target, view.branch, view.revision, view.day)?
            else {
                continue;
            };
            if resolve_object(&self.conn, source, view.branch, view.revision, view.day)?.is_none() {
                continue;
            }
            result.push(NetworkFact {
                id: Uuid::from_bytes(row.get(0)?),
                input: NetworkInput {
                    subject,
                    target,
                    source,
                    kind: NetworkKind::parse(&row.get::<_, String>(4)?)?,
                    role: row.get(5)?,
                    starts_at: row.get(6)?,
                    ends_before: row.get(7)?,
                },
                subject_label: a.record.label,
                target_label: b.record.label,
            });
        }
        Ok(result)
    }

    /// Traverse only persisted, active, confirmed event links; allegations never enter the DAG.
    pub fn persisted_impact(
        &self,
        view: ViewContext,
        start: ObjectRef,
        max_depth: u8,
        max_visited: usize,
    ) -> Result<crate::conflict::Impact, Error> {
        metadata_world(&self.conn, view, false)?;
        let mut stmt=self.conn.prepare(&format!("{METADATA_CTE} SELECT subject_id,target_id,source_id FROM active_network WHERE kind='confirmed_cause' AND starts_at<=?4 AND (ends_before IS NULL OR ends_before>?4) ORDER BY id LIMIT 4001"))?;
        let mut rows = stmt.query(params![
            view.world.0.as_bytes(),
            view.branch.0.as_bytes(),
            view.revision.0,
            view.day.0
        ])?;
        let mut links = Vec::new();
        while let Some(row) = rows.next()? {
            self.check_query_control()?;
            if links.len() == 4000 {
                return Err(Error::QueryBudgetExceeded);
            }
            links.push(crate::conflict::CausalLink {
                cause: ref_id(view.world, row.get(0)?),
                consequence: ref_id(view.world, row.get(1)?),
                source: ref_id(view.world, row.get(2)?),
                status: crate::conflict::LinkStatus::Confirmed,
            });
        }
        drop(rows);
        drop(stmt);
        self.confirmed_impact(view, start, &links, max_depth, max_visited)
    }

    /// Resolve explicitly reviewed identity redirects without rewriting canonical references.
    pub fn canonical_identity(
        &self,
        view: ViewContext,
        start: ObjectRef,
    ) -> Result<Vec<ObjectRef>, Error> {
        let mut path = vec![start];
        for _ in 0..32 {
            let focus = *path.last().ok_or(Error::NotFound)?;
            let facts = self.network_facts(view, focus)?;
            let mut targets = facts.into_iter().filter(|fact| {
                fact.input.kind == NetworkKind::IdentityRedirect && fact.input.subject == focus
            });
            let Some(fact) = targets.next() else {
                return Ok(path);
            };
            if targets.next().is_some() || path.contains(&fact.input.target) {
                return Err(Error::Integrity("ambiguous identity redirect".into()));
            }
            path.push(fact.input.target);
        }
        Err(Error::QueryBudgetExceeded)
    }

    /// An explicit, sourced designated heir is the only supported automatic rule.
    /// Other rule names are records, not sufficient birth-order/eligibility evidence.
    pub fn succession_preview(
        &self,
        view: ViewContext,
        title: ObjectRef,
    ) -> Result<crate::network::SuccessionPreview, Error> {
        metadata_world(&self.conn, view, false)?;
        if !matches!(
            resolved(&self.conn, view, title)?.record.data,
            ObjectData::Title
        ) {
            return Err(Error::WrongKind);
        }
        let facts = self.network_facts(view, title)?;
        let rule = facts
            .iter()
            .find(|fact| fact.input.kind == NetworkKind::RuleOf && fact.input.target == title)
            .map(|fact| fact.input.role.clone());
        let incumbent = facts
            .iter()
            .find(|fact| fact.input.kind == NetworkKind::TitleHolding && fact.input.target == title)
            .map(|fact| fact.input.subject);
        let designated = facts
            .iter()
            .find(|fact| {
                fact.input.kind == NetworkKind::HeirDesignation && fact.input.target == title
            })
            .map(|fact| fact.input.subject);
        let eligible = rule.as_deref() == Some("designated_heir")
            && incumbent.is_none()
            && designated.is_some();
        let explanation = if incumbent.is_some() {
            "Title currently occupied."
        } else if rule.as_deref() != Some("designated_heir") {
            "No executable designated-heir rule; other laws require explicit interpretation."
        } else if designated.is_none() {
            "No active sourced heir designation."
        } else {
            "Vacant title and active designated heir. Author confirmation required."
        };
        Ok(crate::network::SuccessionPreview {
            rule,
            incumbent,
            designated,
            eligible,
            explanation: explanation.into(),
        })
    }

    pub fn enact_designated_succession(
        &mut self,
        view: ViewContext,
        title: ObjectRef,
        source: ObjectRef,
        command_id: Uuid,
    ) -> Result<Uuid, Error> {
        let preview = self.succession_preview(view, title)?;
        if !preview.eligible {
            return Err(Error::InvalidInput("succession not eligible"));
        }
        self.record_network_fact(
            view,
            &NetworkInput {
                subject: preview.designated.ok_or(Error::NotFound)?,
                target: title,
                source,
                kind: NetworkKind::TitleHolding,
                role: "designated succession".into(),
                starts_at: view.day.0,
                ends_before: None,
            },
            command_id,
        )
    }

    /// Read-only candidate generation. The author still chooses the type and exact span.
    /// A conservative capitalization heuristic is not a semantic classifier.
    pub fn ghost_candidates(
        &self,
        view: ViewContext,
        claim: ObjectRef,
    ) -> Result<Vec<crate::network::GhostCandidate>, Error> {
        author_context(&self.conn, view)?;
        let version = resolved(&self.conn, view, claim)?;
        let ObjectData::Claim { statement, .. } = &version.record.data else {
            return Err(Error::WrongKind);
        };
        if statement.len() > 10_000 {
            return Err(Error::QueryBudgetExceeded);
        }
        let spans = crate::network::capitalized_spans(statement);
        let mut results = Vec::new();
        for (start_byte, end_byte) in spans {
            self.check_query_control()?;
            let label = &statement[start_byte..end_byte];
            let spec = QuerySpec {
                schema_version: QUERY_SCHEMA_VERSION,
                filter: QueryFilter::Or(vec![
                    QueryFilter::ExactLabel(label.into()),
                    QueryFilter::ExactAlias(label.into()),
                ]),
                limit: 50,
            };
            let found = self.execute_query(&spec, view)?;
            if found.truncated {
                return Err(Error::QueryBudgetExceeded);
            }
            let existing = found
                .rows
                .into_iter()
                .filter(|item| {
                    matches!(
                        item.record.data,
                        ObjectData::Person
                            | ObjectData::Organization
                            | ObjectData::FamilyHouse
                            | ObjectData::Household
                            | ObjectData::Faction
                            | ObjectData::Conflict
                            | ObjectData::Office
                            | ObjectData::Title
                            | ObjectData::Asset
                            | ObjectData::Population
                            | ObjectData::ConflictIssue
                            | ObjectData::ConflictPhase
                            | ObjectData::SuccessionRule
                            | ObjectData::Concept
                    )
                })
                .map(|item| item.object)
                .collect();
            results.push(crate::network::GhostCandidate {
                label: label.into(),
                start_byte,
                end_byte,
                existing,
            });
        }
        Ok(results)
    }

    /// A reviewed draft can resolve to a pre-existing identity without rewriting
    /// any canonical record. Preserve the original claim version and span.
    pub fn resolve_ghost_to_existing(
        &mut self,
        view: ViewContext,
        proposal: Uuid,
        target: ObjectRef,
        command_id: Uuid,
    ) -> Result<(), Error> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = author_context(&tx, view)?;
        let (claim, branch, state, recorded, label, kind, day): (
            [u8; 16],
            [u8; 16],
            i64,
            i64,
            String,
            String,
            i64,
        ) = tx
            .query_row(
                "SELECT claim_id,claim_branch,claim_state_revision,claim_revision,label,kind,day
                 FROM ghost_proposals WHERE world_id=?1 AND id=?2 AND reviewed_revision IS NULL",
                params![view.world.0.as_bytes(), proposal.as_bytes()],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                    ))
                },
            )
            .optional()?
            .ok_or(Error::NotFound)?;
        let effective = ViewContext {
            day: WorldDay(day),
            ..view
        };
        let latest = resolved(&tx, effective, ref_id(view.world, claim))?;
        if latest.source_branch.0.as_bytes() != &branch
            || latest.state_revision.0 != state
            || latest.recorded_at.0 != recorded
        {
            return Err(Error::InvalidInput("ghost source changed"));
        }
        let existing = resolved(&tx, effective, target)?;
        if existing.record.data != IdentityKind::parse(&kind)?.data() {
            return Err(Error::WrongKind);
        }
        let matches_alias = if kind == "person" {
            active_alias_versions(&tx, effective)?.iter().any(|alias| {
                alias.person == target
                    && alias.active
                    && alias.starts_at.is_none_or(|start| start <= effective.day)
                    && alias.ends_before.is_none_or(|end| effective.day < end)
                    && alias.label.to_lowercase() == label.to_lowercase()
            })
        } else {
            false
        };
        if existing.record.label.to_lowercase() != label.to_lowercase() && !matches_alias {
            return Err(Error::InvalidInput(
                "existing identity name or alias does not match",
            ));
        }
        let revision = append_revision(
            &tx,
            &world,
            world.main,
            view.revision,
            command_id,
            ("Local author", "Resolve draft to existing identity"),
        )?;
        tx.execute(
            "UPDATE ghost_proposals SET reviewed_revision=?3,promoted_id=?4 WHERE world_id=?1 AND id=?2",
            params![view.world.0.as_bytes(), proposal.as_bytes(), revision, target.id.0.as_bytes()],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// A proposal is explicitly selected from an exact claim version and UTF-8 byte span.
    pub fn propose_ghost(
        &mut self,
        view: ViewContext,
        claim: ObjectRef,
        span: (usize, usize),
        kind: IdentityKind,
        command_id: Uuid,
    ) -> Result<Uuid, Error> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = author_context(&tx, view)?;
        let version = resolved(&tx, view, claim)?;
        let ObjectData::Claim { statement, .. } = &version.record.data else {
            return Err(Error::WrongKind);
        };
        let label = statement
            .get(span.0..span.1)
            .ok_or(Error::InvalidInput("ghost source span"))?;
        bounded_text(label)?;
        if label.trim() != label {
            return Err(Error::InvalidInput("ghost span whitespace"));
        }
        let duplicate: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM ghost_proposals WHERE world_id=?1 AND claim_id=?2
            AND claim_revision=?3 AND start_byte=?4 AND end_byte=?5 AND reviewed_revision IS NULL)",
            params![
                view.world.0.as_bytes(),
                claim.id.0.as_bytes(),
                version.recorded_at.0,
                span.0,
                span.1
            ],
            |row| row.get(0),
        )?;
        if duplicate {
            return Err(Error::InvalidInput("existing ghost proposal"));
        }
        let revision = append_revision(
            &tx,
            &world,
            world.main,
            view.revision,
            command_id,
            ("Local author", "Propose draft identity"),
        )?;
        let id = Uuid::new_v4();
        tx.execute("INSERT INTO ghost_proposals(world_id,id,claim_id,claim_branch,claim_state_revision,claim_revision,start_byte,end_byte,label,kind,day,recorded_revision)
            VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![view.world.0.as_bytes(),id.as_bytes(),claim.id.0.as_bytes(),
            version.source_branch.0.as_bytes(),version.state_revision.0,version.recorded_at.0,span.0,span.1,label,kind.name(),view.day.0,revision])?;
        tx.commit()?;
        Ok(id)
    }

    pub fn ghost_proposals(&self, view: ViewContext) -> Result<Vec<GhostProposal>, Error> {
        author_context(&self.conn, view)?;
        let mut stmt=self.conn.prepare("SELECT id,claim_id,label,kind,day,recorded_revision,
            CASE WHEN reviewed_revision<=?2 THEN 1 ELSE 0 END,
            CASE WHEN reviewed_revision<=?2 THEN promoted_id ELSE NULL END
            FROM ghost_proposals WHERE world_id=?1 AND recorded_revision<=?2 ORDER BY recorded_revision,id LIMIT 501")?;
        let mut rows = stmt.query(params![view.world.0.as_bytes(), view.revision.0])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            if result.len() == 500 {
                return Err(Error::QueryBudgetExceeded);
            }
            result.push(GhostProposal {
                id: Uuid::from_bytes(row.get(0)?),
                claim: ref_id(view.world, row.get(1)?),
                label: row.get(2)?,
                kind: IdentityKind::parse(&row.get::<_, String>(3)?)?,
                day: row.get(4)?,
                recorded_revision: row.get(5)?,
                reviewed: row.get(6)?,
                promoted: row
                    .get::<_, Option<[u8; 16]>>(7)?
                    .map(|id| ref_id(view.world, id)),
            });
        }
        Ok(result)
    }

    /// Promote a new identity or dismiss. Reject stale source evidence and exact duplicate names.
    pub fn review_ghost(
        &mut self,
        view: ViewContext,
        id: Uuid,
        promote: bool,
        command_id: Uuid,
    ) -> Result<Option<ObjectRef>, Error> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = author_context(&tx, view)?;
        let (claim,branch,state,recorded,label,kind,day):([u8;16],[u8;16],i64,i64,String,String,i64)=tx.query_row(
            "SELECT claim_id,claim_branch,claim_state_revision,claim_revision,label,kind,day FROM ghost_proposals
             WHERE world_id=?1 AND id=?2 AND reviewed_revision IS NULL",
            params![view.world.0.as_bytes(),id.as_bytes()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?)))
            .optional()?.ok_or(Error::NotFound)?;
        let revision = append_revision(
            &tx,
            &world,
            world.main,
            view.revision,
            command_id,
            ("Local author", "Review draft identity"),
        )?;
        let object = if promote {
            let effective = ViewContext {
                day: WorldDay(day),
                ..view
            };
            let latest = resolved(&tx, effective, ref_id(view.world, claim))?;
            if latest.source_branch.0.as_bytes() != &branch
                || latest.state_revision.0 != state
                || latest.recorded_at.0 != recorded
            {
                return Err(Error::InvalidInput("ghost source changed"));
            }
            // Registry scan is deliberately bounded, and never silently drops possible duplicates.
            let mut stmt =
                tx.prepare("SELECT id FROM objects WHERE world_id=?1 AND kind=?2 LIMIT 10001")?;
            let mut rows = stmt.query(params![view.world.0.as_bytes(), kind])?;
            let mut count = 0;
            while let Some(row) = rows.next()? {
                count += 1;
                if count > 10000 {
                    return Err(Error::QueryBudgetExceeded);
                }
                let candidate = ref_id(view.world, row.get(0)?);
                if resolve_object(&tx, candidate, view.branch, view.revision, WorldDay(day))?
                    .is_some_and(|v| v.record.label.trim().to_lowercase() == label.to_lowercase())
                {
                    return Err(Error::InvalidInput("duplicate identity requires review"));
                }
            }
            let object = ObjectRef::new(view.world);
            write_version(
                &tx,
                world.main,
                revision,
                revision,
                object,
                &Record {
                    label,
                    valid: ValidInterval {
                        start: Some(WorldDay(day)),
                        end_exclusive: None,
                    },
                    data: IdentityKind::parse(&kind)?.data(),
                    source: latest.record.source,
                },
                true,
            )?;
            tx.execute(
                "INSERT INTO invalidations(world_id,revision,object_id) VALUES(?1,?2,?3)",
                params![view.world.0.as_bytes(), revision, object.id.0.as_bytes()],
            )?;
            Some(object)
        } else {
            None
        };
        tx.execute("UPDATE ghost_proposals SET reviewed_revision=?3,promoted_id=?4 WHERE world_id=?1 AND id=?2",
            params![view.world.0.as_bytes(),id.as_bytes(),revision,object.map(|v|*v.id.0.as_bytes())])?;
        tx.commit()?;
        Ok(object)
    }
}
