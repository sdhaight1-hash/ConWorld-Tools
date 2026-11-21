use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use uuid::Uuid;

use crate::discovery::{
    MentionCandidate, MentionDisposition, PersonAlias, ReviewMention, SavePersonAlias,
    matching_spans,
};
use crate::query::{
    PredicateInput, QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec, SaveNamedLabelQuery,
    SavedLabelQuery, validate_predicate,
};
use crate::*;

pub(crate) const APPLICATION_ID: i64 = 0x4357_4c44; // CWLD
pub(crate) const SCHEMA_VERSION: i64 = 16;

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
        }
        if version < 7 {
            tx.execute_batch(include_str!("../migrations/0007.sql"))?;
        }
        if version < 8 {
            tx.execute_batch(include_str!("../migrations/0008.sql"))?;
        }
        if version < 9 {
            tx.execute_batch(include_str!("../migrations/0009.sql"))?;
        }
        if version < 10 {
            tx.execute_batch(include_str!("../migrations/0010.sql"))?;
        }
        if version < 11 {
            tx.execute_batch(include_str!("../migrations/0011.sql"))?;
        }
        if version < 12 {
            tx.execute_batch(include_str!("../migrations/0012.sql"))?;
        }
        if version < 13 {
            tx.execute_batch(include_str!("../migrations/0013.sql"))?;
        }
        if version < 14 {
            tx.execute_batch(include_str!("../migrations/0014.sql"))?;
        }
        // Version 15 adds saved predicate operators without DDL.
        // Version 16 adds the reverse relationship endpoint index.
        if version < 16 {
            tx.execute_batch(include_str!("../migrations/0016.sql"))?;
        }
        if version < SCHEMA_VERSION {
            verify_foreign_keys(&tx)?;
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        tx.commit()?;
        conn.pragma_update(None, "foreign_keys", true)?;
        Ok(Self { conn })
    }

    /// Reject unknown branches (including forks not yet created at the selected
    /// author revision) even when the world contains no objects.
    pub(crate) fn validate_query_view(&self, view: ViewContext) -> Result<(), Error> {
        let world = load_world(&self.conn, view.world)?;
        if view.revision < AuthorRevision(0) || view.revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM branches b
                LEFT JOIN branch_forks f ON f.world_id = b.world_id AND f.branch_id = b.id
                WHERE b.world_id = ?1 AND b.id = ?2
                  AND (b.is_main = 1 OR f.created_revision <= ?3)
            )",
            params![
                view.world.0.as_bytes(),
                view.branch.0.as_bytes(),
                view.revision.0
            ],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    /// Latest alias metadata known by the selected author revision.
    /// Aliases are main-branch author metadata until branch-aware alias
    /// inheritance is specified; no alias rows are projected into forks.
    pub fn person_aliases(&self, view: ViewContext) -> Result<Vec<PersonAlias>, Error> {
        require_author(view.scope)?;
        self.validate_query_view(view)?;
        let world = load_world(&self.conn, view.world)?;
        if view.branch != world.main {
            return Ok(Vec::new());
        }
        active_alias_versions(&self.conn, view)
    }

    /// Create/revise an author-owned alias with optimistic author history.
    /// An alias does not change canonical label, identity or relationships.
    pub fn save_person_alias(
        &mut self,
        request: SavePersonAlias<'_>,
    ) -> Result<PersonAlias, Error> {
        require_author(request.scope)?;
        if request.person.world != request.world {
            return Err(Error::WrongWorld);
        }
        let label = request.label.trim();
        if label.is_empty() || label.len() > 200 {
            return Err(Error::InvalidInput("person alias label"));
        }
        if let (Some(start), Some(end)) = (request.starts_at, request.ends_before)
            && start >= end
        {
            return Err(Error::InvalidInterval);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = load_world(&tx, request.world)?;
        if request.expected != world.revision {
            return Err(Error::RevisionConflict {
                expected: request.expected,
                actual: world.revision,
            });
        }
        let person = resolve_object(
            &tx,
            request.person,
            world.main,
            request.expected,
            request.day,
        )?
        .ok_or(Error::NotFound)?;
        if !matches!(person.record.data, ObjectData::Person) {
            return Err(Error::WrongKind);
        }
        let id = request.alias_id.unwrap_or_else(Uuid::new_v4);
        if request.alias_id.is_some() {
            let existing: Option<[u8; 16]> = tx
                .query_row(
                    "SELECT person_id FROM person_aliases WHERE world_id = ?1 AND id = ?2",
                    params![world.id.0.as_bytes(), id.as_bytes()],
                    |row| row.get(0),
                )
                .optional()?;
            if existing != Some(*request.person.id.0.as_bytes()) {
                return Err(Error::NotFound);
            }
        }
        let revision = append_revision(
            &tx,
            &world,
            world.main,
            request.expected,
            request.command_id,
            ("Local author", "Save person name alias"),
        )?;
        if request.alias_id.is_none() {
            tx.execute(
                "INSERT INTO person_aliases (world_id, id, person_id) VALUES (?1, ?2, ?3)",
                params![
                    world.id.0.as_bytes(),
                    id.as_bytes(),
                    request.person.id.0.as_bytes()
                ],
            )?;
        }
        tx.execute(
            "INSERT INTO person_alias_versions
                (world_id, alias_id, revision, label, starts_at, ends_before, active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                world.id.0.as_bytes(),
                id.as_bytes(),
                revision,
                label,
                request.starts_at.map(|d| d.0),
                request.ends_before.map(|d| d.0),
                request.active,
            ],
        )?;
        tx.commit()?;
        Ok(PersonAlias {
            id,
            person: request.person,
            label: label.into(),
            starts_at: request.starts_at,
            ends_before: request.ends_before,
            active: request.active,
            recorded_at: AuthorRevision(revision),
        })
    }

    /// Reconstruct disposable label and claim statement indexes from canonical versions.
    /// No author revision or object identity is changed.
    pub fn rebuild_search_index(&mut self, scope: Scope) -> Result<(), Error> {
        require_author(scope)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("DELETE FROM object_search_documents", [])?;
        tx.execute(
            "INSERT INTO object_search_documents
                (world_id, object_id, branch_id, state_revision, revision, label)
             SELECT world_id, object_id, branch_id, state_revision, revision, label
             FROM object_versions
             ORDER BY world_id, object_id, branch_id, state_revision, revision",
            [],
        )?;
        tx.execute("DELETE FROM claim_search_documents", [])?;
        tx.execute(
            "INSERT INTO claim_search_documents
                (world_id, claim_id, branch_id, state_revision, revision, statement)
             SELECT world_id, claim_id, branch_id, state_revision, revision, statement
             FROM claim_versions
             ORDER BY world_id, claim_id, branch_id, state_revision, revision",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Find candidate IDs via canonical typed involvement indexes, then resolve
    /// every identity through the authoritative world/branch/day/revision view.
    /// Never treat an older, future or other-branch involvement row as current.
    pub(crate) fn indexed_involvements_at(
        &self,
        view: ViewContext,
        focus: ObjectRef,
        role: Option<&EventRole>,
        by_participant: bool,
        limit: usize,
    ) -> Result<crate::query::QueryResult, Error> {
        require_author(view.scope)?;
        if focus.world != view.world {
            return Err(Error::WrongWorld);
        }
        if let Some(EventRole::Custom(definition)) = role
            && definition.world != view.world
        {
            return Err(Error::WrongWorld);
        }
        self.validate_query_view(view)?;
        let tx = self.conn.unchecked_transaction()?;
        let focus_version = resolve_object(&tx, focus, view.branch, view.revision, view.day)?;
        let allowed = focus_version.is_some_and(|version| {
            matches!(
                (&version.record.data, by_participant),
                (ObjectData::Person, true) | (ObjectData::HistoricalEvent { .. }, false)
            )
        });
        if !allowed {
            return Ok(crate::query::QueryResult {
                rows: Vec::new(),
                revision: view.revision,
                truncated: false,
            });
        }
        const RAW_BUDGET: i64 = 10_000;
        let candidates = {
            let sql = if by_participant {
                "SELECT DISTINCT involvement_id FROM event_involvement_versions
                 WHERE world_id = ?1 AND participant_id = ?2
                 ORDER BY involvement_id LIMIT ?3"
            } else {
                "SELECT DISTINCT involvement_id FROM event_involvement_versions
                 WHERE world_id = ?1 AND event_id = ?2
                 ORDER BY involvement_id LIMIT ?3"
            };
            let mut statement = tx.prepare(sql)?;
            statement
                .query_map(
                    params![
                        view.world.0.as_bytes(),
                        focus.id.0.as_bytes(),
                        RAW_BUDGET + 1
                    ],
                    |row| row.get::<_, [u8; 16]>(0),
                )?
                .collect::<Result<Vec<_>, _>>()?
        };
        if candidates.len() > RAW_BUDGET as usize {
            return Err(Error::QueryBudgetExceeded);
        }
        let mut rows = Vec::new();
        let mut events = HashSet::new();
        for id in candidates {
            let object = ObjectRef {
                world: view.world,
                id: ObjectId(Uuid::from_bytes(id)),
            };
            let Some(involvement) =
                resolve_object(&tx, object, view.branch, view.revision, view.day)?
            else {
                continue;
            };
            let ObjectData::EventInvolvement {
                event,
                participant,
                role: recorded_role,
            } = &involvement.record.data
            else {
                continue;
            };
            if (by_participant && participant != &focus)
                || (!by_participant && event != &focus)
                || role.is_some_and(|wanted| wanted != recorded_role)
            {
                continue;
            }
            let required = if by_participant { *event } else { *participant };
            let Some(target_version) =
                resolve_object(&tx, required, view.branch, view.revision, view.day)?
            else {
                continue;
            };
            let expected = if by_participant {
                matches!(
                    &target_version.record.data,
                    ObjectData::HistoricalEvent { .. }
                )
            } else {
                matches!(&target_version.record.data, ObjectData::Person)
            };
            if !expected {
                continue;
            }
            if by_participant {
                if events.insert(*event) {
                    rows.push(target_version);
                }
            } else {
                rows.push(involvement);
            }
        }
        rows.sort_by(|a, b| {
            a.record
                .label
                .cmp(&b.record.label)
                .then(a.object.id.0.cmp(&b.object.id.0))
        });
        let truncated = rows.len() > limit;
        rows.truncate(limit);
        tx.commit()?;
        Ok(crate::query::QueryResult {
            rows,
            revision: view.revision,
            truncated,
        })
    }

    /// Traverse resolved historical relationships by indexed endpoint instead
    /// of materializing all world records. Raw candidate and visited budgets
    /// are independent; hidden/obsolete edges never enter the walk.
    pub(crate) fn indexed_relationship_path_at(
        &self,
        view: ViewContext,
        filter: &QueryFilter,
        limit: usize,
    ) -> Result<crate::query::QueryResult, Error> {
        use crate::query::{Direction, QueryResult, search_label};

        require_author(view.scope)?;
        let QueryFilter::RelationshipPath {
            start,
            kind,
            direction,
            max_depth,
            max_visited,
        } = filter
        else {
            return Err(Error::InvalidInput("relationship path"));
        };
        if start.world != view.world {
            return Err(Error::WrongWorld);
        }
        let kind = search_label(kind)?;
        if !(1..=16).contains(max_depth) || !(1..=2_000).contains(max_visited) {
            return Err(Error::InvalidInput("relationship traversal budget"));
        }
        self.validate_query_view(view)?;
        let tx = self.conn.unchecked_transaction()?;
        if resolve_object(&tx, *start, view.branch, view.revision, view.day)?.is_none() {
            return Ok(QueryResult {
                rows: Vec::new(),
                revision: view.revision,
                truncated: false,
            });
        }
        let sql = match direction {
            Direction::Outgoing => {
                "SELECT DISTINCT relationship_id FROM relationship_versions
                 WHERE world_id = ?1 AND subject_id = ?2 AND revision <= ?3
                 ORDER BY relationship_id LIMIT ?4"
            }
            Direction::Incoming => {
                "SELECT DISTINCT relationship_id FROM relationship_versions
                 WHERE world_id = ?1 AND target_id = ?2 AND revision <= ?3
                 ORDER BY relationship_id LIMIT ?4"
            }
            Direction::Both => {
                "SELECT relationship_id FROM relationship_versions
                 WHERE world_id = ?1 AND subject_id = ?2 AND revision <= ?3
                 UNION
                 SELECT relationship_id FROM relationship_versions
                 WHERE world_id = ?1 AND target_id = ?2 AND revision <= ?3
                 ORDER BY relationship_id LIMIT ?4"
            }
        };
        let mut remaining = 10_000_usize;
        let mut visited = HashSet::from([*start]);
        let mut queue = VecDeque::from([(*start, 0_u8)]);
        let mut rows = Vec::new();
        let mut truncated = false;
        'walk: while let Some((node, depth)) = queue.pop_front() {
            if depth == *max_depth {
                continue;
            }
            let candidates = {
                let mut statement = tx.prepare_cached(sql)?;
                statement
                    .query_map(
                        params![
                            view.world.0.as_bytes(),
                            node.id.0.as_bytes(),
                            view.revision.0,
                            (remaining + 1) as i64
                        ],
                        |row| row.get::<_, [u8; 16]>(0),
                    )?
                    .collect::<Result<Vec<_>, _>>()?
            };
            if candidates.len() > remaining {
                return Err(Error::QueryBudgetExceeded);
            }
            remaining -= candidates.len();
            let mut neighbors = HashSet::new();
            for id in candidates {
                let object = ObjectRef {
                    world: view.world,
                    id: ObjectId(Uuid::from_bytes(id)),
                };
                let Some(edge) =
                    resolve_object(&tx, object, view.branch, view.revision, view.day)?
                else {
                    continue;
                };
                let ObjectData::Relationship {
                    subject,
                    target,
                    kind: label,
                    ..
                } = edge.record.data
                else {
                    continue;
                };
                if label.to_lowercase() != kind {
                    continue;
                }
                if *direction != Direction::Incoming && subject == node {
                    neighbors.insert(target);
                }
                if *direction != Direction::Outgoing && target == node {
                    neighbors.insert(subject);
                }
            }
            let mut neighbors: Vec<_> = neighbors.into_iter().collect();
            neighbors.sort_unstable_by_key(|item| item.id.0);
            for next in neighbors {
                if visited.contains(&next) {
                    continue;
                }
                let Some(version) =
                    resolve_object(&tx, next, view.branch, view.revision, view.day)?
                else {
                    continue;
                };
                if visited.len() == *max_visited {
                    truncated = true;
                    break 'walk;
                }
                visited.insert(next);
                rows.push(version);
                queue.push_back((next, depth + 1));
            }
        }
        rows.sort_by(|a, b| {
            a.record
                .label
                .cmp(&b.record.label)
                .then(a.object.id.0.cmp(&b.object.id.0))
        });
        truncated |= rows.len() > limit;
        rows.truncate(limit);
        tx.commit()?;
        Ok(QueryResult {
            rows,
            revision: view.revision,
            truncated,
        })
    }

    /// Indexed phrase search still resolves every candidate through the authoritative
    /// branch/day/revision projection. Matching an obsolete version never returns its
    /// current replacement. Raw index candidates are explicitly bounded.
    pub(crate) fn indexed_phrase_at(
        &self,
        phrase: &str,
        view: ViewContext,
        limit: usize,
    ) -> Result<crate::query::QueryResult, Error> {
        self.indexed_text_at(phrase, view, limit, false)
    }

    pub(crate) fn indexed_claim_phrase_at(
        &self,
        phrase: &str,
        view: ViewContext,
        limit: usize,
    ) -> Result<crate::query::QueryResult, Error> {
        self.indexed_text_at(phrase, view, limit, true)
    }

    fn indexed_text_at(
        &self,
        phrase: &str,
        view: ViewContext,
        limit: usize,
        claims_only: bool,
    ) -> Result<crate::query::QueryResult, Error> {
        require_author(view.scope)?;
        let phrase = phrase.trim();
        if phrase.is_empty() || phrase.len() > 800 {
            return Err(Error::InvalidInput("full-text search phrase"));
        }
        // Quote the entire input as a literal FTS5 phrase: never accept FTS
        // operators, SQL, or untrusted MATCH expressions from the UI.
        let match_expression = format!("\"{}\"", phrase.replace('"', "\"\""));
        let tx = self.conn.unchecked_transaction()?;
        let world = load_world(&tx, view.world)?;
        if view.revision < AuthorRevision(0) || view.revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        const CANDIDATE_BUDGET: i64 = 10_000;
        let matches = {
            // Static, allowlisted SQL plans only. User input is always a bound
            // literal FTS phrase; never a table name or MATCH expression.
            let sql = if claims_only {
                "SELECT d.claim_id, d.branch_id, d.state_revision, d.revision
                 FROM claim_search_fts
                 JOIN claim_search_documents d ON d.doc_id = claim_search_fts.rowid
                 WHERE d.world_id = ?1 AND claim_search_fts MATCH ?2
                 ORDER BY d.claim_id, d.branch_id, d.state_revision, d.revision
                 LIMIT ?3"
            } else {
                "SELECT d.object_id, d.branch_id, d.state_revision, d.revision
                 FROM object_search_fts
                 JOIN object_search_documents d ON d.doc_id = object_search_fts.rowid
                 WHERE d.world_id = ?1 AND object_search_fts MATCH ?2
                 ORDER BY d.object_id, d.branch_id, d.state_revision, d.revision
                 LIMIT ?3"
            };
            let mut statement = tx.prepare(sql)?;
            statement
                .query_map(
                    params![
                        view.world.0.as_bytes(),
                        match_expression,
                        CANDIDATE_BUDGET + 1
                    ],
                    |row| {
                        Ok((
                            ObjectId(Uuid::from_bytes(row.get(0)?)),
                            BranchId(Uuid::from_bytes(row.get(1)?)),
                            row.get::<_, i64>(2)?,
                            row.get::<_, i64>(3)?,
                        ))
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?
        };
        if matches.len() > CANDIDATE_BUDGET as usize {
            return Err(Error::QueryBudgetExceeded);
        }
        let mut candidates: HashMap<ObjectId, HashSet<(BranchId, i64, i64)>> = HashMap::new();
        for (object, branch, state_revision, revision) in matches {
            candidates
                .entry(object)
                .or_default()
                .insert((branch, state_revision, revision));
        }
        let mut rows = Vec::new();
        for (id, matching_versions) in candidates {
            let object = ObjectRef {
                world: view.world,
                id,
            };
            if let Some(version) =
                resolve_object(&tx, object, view.branch, view.revision, view.day)?
                && matching_versions.contains(&(
                    version.source_branch,
                    version.state_revision.0,
                    version.recorded_at.0,
                ))
                && (!claims_only || matches!(&version.record.data, ObjectData::Claim { .. }))
            {
                rows.push(version);
            }
        }
        rows.sort_by(|a, b| {
            a.record
                .label
                .cmp(&b.record.label)
                .then(a.object.id.0.cmp(&b.object.id.0))
        });
        let truncated = rows.len() > limit;
        rows.truncate(limit);
        tx.commit()?;
        Ok(crate::query::QueryResult {
            rows,
            revision: view.revision,
            truncated,
        })
    }

    /// Save one of the accepted textual QuerySpec filters as append-only author
    /// metadata. The query definition has a stable ID and its own author history.
    pub fn save_named_label_query(
        &mut self,
        request: SaveNamedLabelQuery<'_>,
    ) -> Result<SavedLabelQuery, Error> {
        let SaveNamedLabelQuery {
            world_id,
            expected,
            query_id,
            name,
            spec,
            command_id,
            scope,
        } = request;
        require_author(scope)?;
        let name = name.trim();
        if name.is_empty() || name.len() > 200 {
            return Err(Error::InvalidInput("saved query name"));
        }
        if spec.schema_version != QUERY_SCHEMA_VERSION || !(1..=500).contains(&spec.limit) {
            return Err(Error::InvalidInput("saved query schema or limit"));
        }
        let (kind, term) = match &spec.filter {
            QueryFilter::LabelContains(text) => ("contains", text.trim().to_string()),
            QueryFilter::ExactLabel(text) => ("exact", text.trim().to_string()),
            QueryFilter::FullTextPhrase(text) => ("fulltext", text.trim().to_string()),
            QueryFilter::ClaimFullTextPhrase(text) => ("claimtext", text.trim().to_string()),
            QueryFilter::Kind(_)
            | QueryFilter::HasSource(_)
            | QueryFilter::And(_)
            | QueryFilter::Or(_)
            | QueryFilter::Not(_) => {
                let mut nodes = 0;
                validate_predicate(&spec.filter, world_id, 0, &mut nodes)?;
                let encoded = PredicateInput::from_filter(&spec.filter)?;
                let text = serde_json::to_string(&encoded)
                    .map_err(|_| Error::InvalidInput("saved predicate serialization"))?;
                ("composite", text)
            }
            _ => return Err(Error::InvalidInput("unsupported saved query filter")),
        };
        if term.is_empty() || term.len() > if kind == "composite" { 4_096 } else { 800 } {
            return Err(Error::InvalidInput("saved query search term"));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = load_world(&tx, world_id)?;
        let id = query_id.unwrap_or_else(Uuid::new_v4);
        if query_id.is_some() {
            let exists: bool = tx.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM saved_queries WHERE world_id = ?1 AND id = ?2
                )",
                params![world_id.0.as_bytes(), id.as_bytes()],
                |row| row.get(0),
            )?;
            if !exists {
                return Err(Error::NotFound);
            }
        }
        let revision = append_revision(
            &tx,
            &world,
            world.main,
            expected,
            command_id,
            ("Local author", "Save versioned label query"),
        )?;
        if query_id.is_none() {
            tx.execute(
                "INSERT INTO saved_queries (world_id, id) VALUES (?1, ?2)",
                params![world_id.0.as_bytes(), id.as_bytes()],
            )?;
        }
        tx.execute(
            "INSERT INTO saved_query_versions
             (world_id, query_id, revision, name, schema_version,
              filter_kind, filter_text, result_limit)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                world_id.0.as_bytes(),
                id.as_bytes(),
                revision,
                name,
                i64::from(spec.schema_version),
                kind,
                term,
                spec.limit as i64
            ],
        )?;
        tx.commit()?;
        Ok(SavedLabelQuery {
            id,
            name: name.into(),
            spec: QuerySpec {
                filter: match kind {
                    "contains" => QueryFilter::LabelContains(term),
                    "exact" => QueryFilter::ExactLabel(term),
                    "fulltext" => QueryFilter::FullTextPhrase(term),
                    "claimtext" => QueryFilter::ClaimFullTextPhrase(term),
                    _ => spec.filter.clone(),
                },
                ..spec.clone()
            },
            recorded_at: AuthorRevision(revision),
        })
    }

    /// Show only definitions known at the selected recorded/author revision.
    /// The actual rows of each query are always resolved when it is executed.
    pub fn saved_label_queries(
        &self,
        world_id: WorldId,
        revision: AuthorRevision,
        scope: Scope,
    ) -> Result<Vec<SavedLabelQuery>, Error> {
        require_author(scope)?;
        let tx = self.conn.unchecked_transaction()?;
        let world = load_world(&tx, world_id)?;
        if revision < AuthorRevision(0) || revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        let records = {
            let mut statement = tx.prepare(
                "SELECT q.query_id, q.name, q.schema_version, q.filter_kind,
                        q.filter_text, q.result_limit, q.revision
                 FROM saved_query_versions q
                 WHERE q.world_id = ?1 AND q.revision = (
                     SELECT MAX(prev.revision) FROM saved_query_versions prev
                     WHERE prev.world_id = q.world_id AND prev.query_id = q.query_id
                       AND prev.revision <= ?2
                 )
                 ORDER BY q.name, q.query_id",
            )?;
            statement
                .query_map(params![world_id.0.as_bytes(), revision.0], |row| {
                    Ok((
                        Uuid::from_bytes(row.get(0)?),
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?
        };
        let mut result = Vec::new();
        for (id, name, schema, kind, term, limit, recorded_at) in records {
            if schema != i64::from(QUERY_SCHEMA_VERSION) || !(1..=500).contains(&limit) {
                return Err(Error::InvalidInput("saved query schema or limit"));
            }
            let filter = match kind.as_str() {
                "contains" => QueryFilter::LabelContains(term),
                "exact" => QueryFilter::ExactLabel(term),
                "fulltext" => QueryFilter::FullTextPhrase(term),
                "claimtext" => QueryFilter::ClaimFullTextPhrase(term),
                "composite" => {
                    let dto: PredicateInput = serde_json::from_str(&term)
                        .map_err(|_| Error::InvalidInput("saved predicate JSON"))?;
                    let mut nodes = 0;
                    let filter = dto.into_filter(world_id, 0, &mut nodes)?;
                    let mut checked = 0;
                    validate_predicate(&filter, world_id, 0, &mut checked)?;
                    filter
                }
                _ => return Err(Error::InvalidInput("saved query filter kind")),
            };
            result.push(SavedLabelQuery {
                id,
                name,
                spec: QuerySpec {
                    schema_version: QUERY_SCHEMA_VERSION,
                    filter,
                    limit: limit as usize,
                },
                recorded_at: AuthorRevision(recorded_at),
            });
        }
        tx.commit()?;
        Ok(result)
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

    /// List existing worlds for the trusted local author UI.
    pub fn worlds(&self, scope: Scope) -> Result<Vec<World>, Error> {
        require_author(scope)?;
        let ids: Vec<[u8; 16]> = {
            let mut statement = self
                .conn
                .prepare("SELECT id FROM worlds ORDER BY name, id")?;
            statement
                .query_map([], |row| row.get(0))?
                .collect::<Result<_, _>>()?
        };
        ids.into_iter()
            .map(|id| load_world(&self.conn, WorldId(Uuid::from_bytes(id))))
            .collect()
    }

    /// Phase Two authoring projection: bounded object snapshots, using one
    /// revision/branch/day and one SQLite read transaction. Phase Three will
    /// replace the scan with the indexed query engine.
    pub fn objects_at(&self, view: ViewContext, limit: usize) -> Result<Vec<Version>, Error> {
        require_author(view.scope)?;
        if !(1..=2_000).contains(&limit) {
            return Err(Error::InvalidInput("object listing limit"));
        }
        self.validate_query_view(view)?;
        let tx = self.conn.unchecked_transaction()?;
        let world = load_world(&tx, view.world)?;
        if view.revision < AuthorRevision(0) || view.revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        // The world registry contains facts from other dates and branches.
        // Applying LIMIT before historical visibility silently hides valid records.
        // Until Phase Three adds indexed/paginated queries, scan a bounded set
        // of candidates and report truncation explicitly instead of returning
        // an incomplete "complete" author workspace.
        const SCAN_BUDGET: i64 = 10_000;
        let mut visible = Vec::new();
        {
            let mut statement =
                tx.prepare("SELECT id FROM objects WHERE world_id = ?1 ORDER BY id LIMIT ?2")?;
            let candidates = statement
                .query_map(params![view.world.0.as_bytes(), SCAN_BUDGET + 1], |row| {
                    row.get::<_, [u8; 16]>(0)
                })?;
            for (scanned, id) in candidates.enumerate() {
                if scanned == SCAN_BUDGET as usize {
                    return Err(Error::QueryBudgetExceeded);
                }
                let object = ObjectRef {
                    world: view.world,
                    id: ObjectId(Uuid::from_bytes(id?)),
                };
                if let Some(version) =
                    resolve_object(&tx, object, view.branch, view.revision, view.day)?
                {
                    visible.push(version);
                    if visible.len() > limit {
                        return Err(Error::QueryBudgetExceeded);
                    }
                }
            }
        }
        tx.commit()?;
        Ok(visible)
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
                Change::EventEffect { target, record, .. }
                | Change::ActionEffect { target, record, .. } => (*target, record),
            };
            if object.world != world.id {
                return Err(Error::WrongWorld);
            }
            if !changed.insert(object.id) {
                return Err(Error::RepeatedObject);
            }
            let effect = match change {
                Change::EventEffect { event, .. } | Change::ActionEffect { event, .. } => {
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
                    let action = match change {
                        Change::ActionEffect { action, .. } => {
                            if action.world != object.world {
                                return Err(Error::WrongWorld);
                            }
                            if *action == object {
                                return Err(Error::InvalidInput(
                                    "action cannot cause its own state change",
                                ));
                            }
                            let version = resolve_object(
                                &tx,
                                *action,
                                branch,
                                AuthorRevision(revision),
                                day,
                            )?
                            .ok_or(Error::NotFound)?;
                            match &version.record.data {
                                ObjectData::ActorAction {
                                    event: action_event,
                                    ..
                                } if *action_event == *event => Some(version.reference()),
                                ObjectData::ActorAction { .. } => {
                                    return Err(Error::InvalidInput(
                                        "action belongs to another event",
                                    ));
                                }
                                _ => return Err(Error::WrongKind),
                            }
                        }
                        _ => None,
                    };
                    let before = resolve_object(&tx, object, branch, world.revision, day)?
                        .ok_or(Error::NotFound)?;
                    Some((origin.reference(), before.reference(), day, action))
                }
                _ => None,
            };
            apply_change(&tx, branch, fork, revision, change, object, record)?;
            if let Some((event, before, day, action)) = effect {
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
                if let Some(action) = action {
                    tx.execute(
                        "INSERT INTO action_effect_links
                         (world_id, target_id, branch_id, effect_revision,
                          action_id, action_branch, action_state_revision, action_revision)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![
                            object.world.0.as_bytes(),
                            object.id.0.as_bytes(),
                            branch.0.as_bytes(),
                            revision,
                            action.object.id.0.as_bytes(),
                            action.branch.0.as_bytes(),
                            action.state_revision.0,
                            action.recorded_at.0
                        ],
                    )?;
                }
            }
            tx.execute(
                "INSERT INTO invalidations (world_id, revision, object_id) VALUES (?1, ?2, ?3)",
                params![object.world.0.as_bytes(), revision, object.id.0.as_bytes()],
            )?;
        }
        tx.commit()?;
        Ok(AuthorRevision(revision))
    }

    /// Author-only explicit decision about a candidate's precise source version
    /// and character boundaries. Re-resolve both ends *inside* the write transaction
    /// before appending an author audit revision.
    pub fn review_mention(&mut self, review: ReviewMention) -> Result<AuthorRevision, Error> {
        require_author(review.view.scope)?;
        let view = review.view;
        if review.source.object.world != view.world || review.target.world != view.world {
            return Err(Error::WrongWorld);
        }
        if review.expected != view.revision {
            return Err(Error::InvalidRevision);
        }
        if review.start_byte >= review.end_byte {
            return Err(Error::InvalidInput("mention byte range"));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = load_world(&tx, view.world)?;
        // The review table currently stores main-branch author decisions only.
        // Without branch lineage on decisions, accepting a fork review could
        // silently make the same source reference appear in its parent.
        if view.branch != world.main {
            return Err(Error::CapabilityDenied);
        }
        if view.revision < AuthorRevision(0) || view.revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        let source = resolve_object(
            &tx,
            review.source.object,
            view.branch,
            view.revision,
            view.day,
        )?
        .ok_or(Error::NotFound)?;
        if source.reference() != review.source {
            return Err(Error::InvalidInput("stale mention source version"));
        }
        let ObjectData::Claim { statement, .. } = &source.record.data else {
            return Err(Error::WrongKind);
        };
        if statement.len() > 10_000 {
            return Err(Error::QueryBudgetExceeded);
        }
        let target = resolve_object(&tx, review.target, view.branch, view.revision, view.day)?
            .ok_or(Error::NotFound)?;
        if !matches!(target.record.data, ObjectData::Person) {
            return Err(Error::WrongKind);
        }
        let matches_name = matching_spans(statement, &target.record.label)
            .contains(&(review.start_byte, review.end_byte));
        let matches_alias = if matches_name {
            false
        } else {
            active_alias_versions(&tx, view)?
                .iter()
                .filter(|alias| alias.person == review.target && alias.matches_day(view.day))
                .any(|alias| {
                    matching_spans(statement, &alias.label)
                        .contains(&(review.start_byte, review.end_byte))
                })
        };
        if !matches_name && !matches_alias {
            return Err(Error::InvalidInput(
                "mention does not match selected person or alias",
            ));
        }
        let revision = append_revision(
            &tx,
            &world,
            world.main,
            review.expected,
            review.command_id,
            ("Local author", "Review historical mention candidate"),
        )?;
        tx.execute(
            "INSERT INTO mention_reviews
             (world_id, source_id, source_branch_id, source_state_revision,
              source_revision, target_id, start_byte, end_byte, decision, revision)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                view.world.0.as_bytes(),
                review.source.object.id.0.as_bytes(),
                review.source.branch.0.as_bytes(),
                review.source.state_revision.0,
                review.source.recorded_at.0,
                review.target.id.0.as_bytes(),
                review.start_byte as i64,
                review.end_byte as i64,
                match review.decision {
                    MentionDisposition::Approved => "approved",
                    MentionDisposition::Dismissed => "dismissed",
                },
                revision,
            ],
        )?;
        tx.commit()?;
        Ok(AuthorRevision(revision))
    }

    /// Show only decisions known at the requested author revision. The
    /// candidate scanner already verified the source and person are visible.
    pub(crate) fn mention_disposition_at(
        &self,
        view: ViewContext,
        candidate: &MentionCandidate,
    ) -> Result<Option<MentionDisposition>, Error> {
        require_author(view.scope)?;
        // Until review decisions are branch-aware, never project main-line
        // approvals into an alternate historical branch.
        if view.branch != load_world(&self.conn, view.world)?.main {
            return Ok(None);
        }
        let decision: Option<String> = self
            .conn
            .query_row(
                "SELECT decision FROM mention_reviews
             WHERE world_id = ?1 AND source_id = ?2 AND source_branch_id = ?3
               AND source_state_revision = ?4 AND source_revision = ?5
               AND target_id = ?6 AND start_byte = ?7 AND end_byte = ?8
               AND revision <= ?9
             ORDER BY revision DESC LIMIT 1",
                params![
                    view.world.0.as_bytes(),
                    candidate.source.object.id.0.as_bytes(),
                    candidate.source.branch.0.as_bytes(),
                    candidate.source.state_revision.0,
                    candidate.source.recorded_at.0,
                    candidate.target.id.0.as_bytes(),
                    candidate.start_byte as i64,
                    candidate.end_byte as i64,
                    view.revision.0,
                ],
                |row| row.get(0),
            )
            .optional()?;
        Ok(match decision.as_deref() {
            Some("approved") => Some(MentionDisposition::Approved),
            Some("dismissed") => Some(MentionDisposition::Dismissed),
            _ => None,
        })
    }

    /// Resolve approved *links*, not inferred relationships, into the existing
    /// author-only Backlinks query. Recheck source version and both objects'
    /// visibility before permitting a reference into the historical result.
    pub(crate) fn approved_mention_sources_at(
        &self,
        view: ViewContext,
        target: ObjectRef,
    ) -> Result<HashSet<ObjectRef>, Error> {
        require_author(view.scope)?;
        if target.world != view.world {
            return Err(Error::WrongWorld);
        }
        if view.branch != load_world(&self.conn, view.world)?.main {
            return Ok(HashSet::new());
        }
        let tx = self.conn.unchecked_transaction()?;
        if resolve_object(&tx, target, view.branch, view.revision, view.day)?.is_none() {
            return Ok(HashSet::new());
        }
        const REVIEW_BUDGET: i64 = 10_000;
        let reviews = {
            let mut statement = tx.prepare(
                "SELECT source_id, source_branch_id, source_state_revision, source_revision,
                        start_byte, end_byte, decision
                 FROM mention_reviews
                 WHERE world_id = ?1 AND target_id = ?2 AND revision <= ?3
                 ORDER BY source_id, source_branch_id, source_state_revision,
                          source_revision, start_byte, end_byte, revision DESC
                 LIMIT ?4",
            )?;
            statement
                .query_map(
                    params![
                        view.world.0.as_bytes(),
                        target.id.0.as_bytes(),
                        view.revision.0,
                        REVIEW_BUDGET + 1
                    ],
                    |row| {
                        Ok((
                            row.get::<_, [u8; 16]>(0)?,
                            row.get::<_, [u8; 16]>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, i64>(3)?,
                            row.get::<_, i64>(4)?,
                            row.get::<_, i64>(5)?,
                            row.get::<_, String>(6)?,
                        ))
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?
        };
        if reviews.len() > REVIEW_BUDGET as usize {
            return Err(Error::QueryBudgetExceeded);
        }
        let mut seen = HashSet::new();
        let mut linked = HashSet::new();
        for (source_id, branch_id, state_revision, recorded_at, start, end, decision) in reviews {
            if !seen.insert((
                source_id,
                branch_id,
                state_revision,
                recorded_at,
                start,
                end,
            )) {
                continue;
            }
            if decision != "approved" {
                continue;
            }
            let source = ObjectRef {
                world: view.world,
                id: ObjectId(Uuid::from_bytes(source_id)),
            };
            if let Some(version) =
                resolve_object(&tx, source, view.branch, view.revision, view.day)?
                && version.source_branch.0.as_bytes() == &branch_id
                && version.state_revision.0 == state_revision
                && version.recorded_at.0 == recorded_at
            {
                linked.insert(source);
            }
        }
        tx.commit()?;
        Ok(linked)
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

    /// Only the explicitly released statement is projected to non-authors.
    /// Event identity, subject links, citations and unreleased counts stay in author reads.
    pub fn evidence_at(
        &self,
        event: ObjectRef,
        view: ViewContext,
    ) -> Result<Vec<EvidenceStatement>, Error> {
        if event.world != view.world {
            return Err(Error::WrongWorld);
        }
        let tx = self.conn.unchecked_transaction()?;
        let world = load_world(&tx, event.world)?;
        if view.revision < AuthorRevision(0) || view.revision > world.revision {
            return Err(Error::InvalidRevision);
        }
        if view.scope != Scope::Author {
            if view.branch != world.main {
                return Err(Error::CapabilityDenied);
            }
            if view.revision != world.revision {
                return Err(Error::InvalidRevision);
            }
        }
        if !resolve_object(&tx, event, view.branch, view.revision, view.day)?.is_some_and(
            |version| matches!(version.record.data, ObjectData::HistoricalEvent { .. }),
        ) {
            return Ok(Vec::new());
        }
        if let Scope::Character(reader) = view.scope {
            require_person(
                &tx,
                world.id,
                reader,
                view.branch,
                view.revision.0,
                view.day,
            )?;
        }
        let mut query = tx.prepare(
            "SELECT DISTINCT claim_id FROM claim_versions WHERE world_id = ?1 AND subject_id = ?2 ORDER BY claim_id"
        )?;
        let ids = query.query_map(
            params![event.world.0.as_bytes(), event.id.0.as_bytes()],
            |row| row.get::<_, [u8; 16]>(0),
        )?;
        let mut results = Vec::new();
        for id in ids {
            let claim = ObjectRef {
                world: event.world,
                id: ObjectId(Uuid::from_bytes(id?)),
            };
            let Some(version) = resolve_object(&tx, claim, view.branch, view.revision, view.day)?
            else {
                continue;
            };
            let ObjectData::Claim {
                subject,
                statement,
                audience,
                ..
            } = version.record.data
            else {
                continue;
            };
            if subject != event || !access::can_view(view.scope, audience) {
                continue;
            }
            let source = if view.scope == Scope::Author {
                version
                    .record
                    .source
                    .map(|object| resolve_object(&tx, object, view.branch, view.revision, view.day))
                    .transpose()?
                    .flatten()
                    .and_then(|source| match source.record.data {
                        ObjectData::Source { citation } => Some(citation),
                        _ => None,
                    })
            } else {
                None
            };
            results.push(EvidenceStatement {
                statement,
                source,
                audience: (view.scope == Scope::Author).then_some(audience),
            });
            if results.len() == 200 {
                break;
            }
        }
        Ok(results)
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
        Ok(self
            .conn
            .query_row(
                "SELECT e.event_id, e.event_branch, e.event_state_revision, e.event_revision,
                    e.before_branch, e.before_state_revision, e.before_revision, e.day,
                    a.action_id, a.action_branch, a.action_state_revision, a.action_revision
             FROM historical_effects AS e
             LEFT JOIN action_effect_links AS a
                ON a.world_id = e.world_id AND a.target_id = e.target_id
                AND a.branch_id = e.branch_id AND a.effect_revision = e.revision
             WHERE e.world_id = ?1 AND e.target_id = ?2 AND e.branch_id = ?3 AND e.revision = ?4",
                params![
                    target.world.0.as_bytes(),
                    target.id.0.as_bytes(),
                    after.branch.0.as_bytes(),
                    after.state_revision.0
                ],
                |row| {
                    Ok(HistoricalEffect {
                        event: VersionRef {
                            object: ObjectRef {
                                world: target.world,
                                id: ObjectId(Uuid::from_bytes(row.get(0)?)),
                            },
                            branch: BranchId(Uuid::from_bytes(row.get(1)?)),
                            state_revision: AuthorRevision(row.get(2)?),
                            recorded_at: AuthorRevision(row.get(3)?),
                        },
                        before: VersionRef {
                            object: target,
                            branch: BranchId(Uuid::from_bytes(row.get(4)?)),
                            state_revision: AuthorRevision(row.get(5)?),
                            recorded_at: AuthorRevision(row.get(6)?),
                        },
                        after: VersionRef {
                            recorded_at: after.state_revision,
                            ..after
                        },
                        day: WorldDay(row.get(7)?),
                        action: row
                            .get::<_, Option<[u8; 16]>>(8)?
                            .map(|id| {
                                Ok::<_, rusqlite::Error>(VersionRef {
                                    object: ObjectRef {
                                        world: target.world,
                                        id: ObjectId(Uuid::from_bytes(id)),
                                    },
                                    branch: BranchId(Uuid::from_bytes(row.get(9)?)),
                                    state_revision: AuthorRevision(row.get(10)?),
                                    recorded_at: AuthorRevision(row.get(11)?),
                                })
                            })
                            .transpose()?,
                    })
                },
            )
            .optional()?)
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
    if let Some((_, prior)) = &existing {
        validate_stable_links(&prior.record.data, &record.data)?;
    }
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
        if let Some(prior) = &inherited {
            validate_stable_links(&prior.record.data, &record.data)?;
        }
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
                Change::RecordState(..)
                | Change::EventEffect { .. }
                | Change::ActionEffect { .. } => {
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
        (
            Change::RecordState(..) | Change::EventEffect { .. } | Change::ActionEffect { .. },
            Some((state, mut previous)),
        ) => {
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

pub(crate) fn verify_foreign_keys(conn: &Connection) -> Result<(), Error> {
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

fn active_alias_versions(conn: &Connection, view: ViewContext) -> Result<Vec<PersonAlias>, Error> {
    const MAX_ALIASES: i64 = 2_000;
    let aliases = {
        let mut statement = conn.prepare(
            "SELECT a.id, a.person_id, v.label, v.starts_at, v.ends_before, v.active, v.revision
             FROM person_aliases a
             JOIN person_alias_versions v ON v.world_id = a.world_id AND v.alias_id = a.id
             WHERE a.world_id = ?1 AND v.revision = (
                 SELECT MAX(prev.revision) FROM person_alias_versions prev
                 WHERE prev.world_id = a.world_id AND prev.alias_id = a.id
                   AND prev.revision <= ?2
             )
             ORDER BY a.id LIMIT ?3",
        )?;
        statement
            .query_map(
                params![view.world.0.as_bytes(), view.revision.0, MAX_ALIASES + 1],
                |row| {
                    Ok(PersonAlias {
                        id: Uuid::from_bytes(row.get(0)?),
                        person: ObjectRef {
                            world: view.world,
                            id: ObjectId(Uuid::from_bytes(row.get(1)?)),
                        },
                        label: row.get(2)?,
                        starts_at: row.get::<_, Option<i64>>(3)?.map(WorldDay),
                        ends_before: row.get::<_, Option<i64>>(4)?.map(WorldDay),
                        active: row.get(5)?,
                        recorded_at: AuthorRevision(row.get(6)?),
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?
    };
    if aliases.len() > MAX_ALIASES as usize {
        return Err(Error::QueryBudgetExceeded);
    }
    Ok(aliases)
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

fn validate_stable_links(old: &ObjectData, new: &ObjectData) -> Result<(), Error> {
    let changed = match (old, new) {
        (
            ObjectData::Definition {
                definition: previous,
                ..
            },
            ObjectData::Definition {
                definition: next, ..
            },
        ) => std::mem::discriminant(previous) != std::mem::discriminant(next),
        (
            ObjectData::Relationship {
                subject: a,
                target: b,
                kind: k,
                ..
            },
            ObjectData::Relationship {
                subject: next_a,
                target: next_b,
                kind: next_k,
                ..
            },
        ) => a != next_a || b != next_b || k != next_k,
        (
            ObjectData::EventInvolvement {
                event: a,
                participant: b,
                ..
            },
            ObjectData::EventInvolvement {
                event: next_a,
                participant: next_b,
                ..
            },
        ) => a != next_a || b != next_b,
        (
            ObjectData::ActorAction {
                event: a, actor: b, ..
            },
            ObjectData::ActorAction {
                event: next_a,
                actor: next_b,
                ..
            },
        ) => a != next_a || b != next_b,
        _ => false,
    };
    if changed {
        Err(Error::InvalidInput(
            "stable relationship or event actor identity",
        ))
    } else {
        Ok(())
    }
}

fn require_visible(
    conn: &Connection,
    world: WorldId,
    reference: ObjectRef,
    branch: BranchId,
    revision: i64,
    day: WorldDay,
) -> Result<Version, Error> {
    if reference.world != world {
        return Err(Error::WrongWorld);
    }
    resolve_object(conn, reference, branch, AuthorRevision(revision), day)?.ok_or(Error::NotFound)
}

fn require_person(
    conn: &Connection,
    world: WorldId,
    reference: ObjectRef,
    branch: BranchId,
    revision: i64,
    day: WorldDay,
) -> Result<(), Error> {
    match require_visible(conn, world, reference, branch, revision, day)?
        .record
        .data
    {
        ObjectData::Person => Ok(()),
        _ => Err(Error::WrongKind),
    }
}

fn require_event(
    conn: &Connection,
    world: WorldId,
    reference: ObjectRef,
    branch: BranchId,
    revision: i64,
    day: WorldDay,
) -> Result<(), Error> {
    match require_visible(conn, world, reference, branch, revision, day)?
        .record
        .data
    {
        ObjectData::HistoricalEvent { occurrence }
            if occurrence.exact_day().is_some_and(|at| at <= day) =>
        {
            Ok(())
        }
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
        ObjectData::Definition {
            definition,
            description,
        } => {
            if record.valid != ValidInterval::default() {
                return Err(Error::InvalidInterval);
            }
            validate_text(description, "definition description")?;
            if let Definition::Relationship {
                reverse_label: Some(reverse),
                symmetric,
            } = definition
            {
                validate_text(reverse, "inverse label")?;
                if *symmetric {
                    return Err(Error::InvalidInput(
                        "a symmetric definition uses the same label in both directions",
                    ));
                }
            }
            ("definition", &EventDate::Unknown, None)
        }
        ObjectData::Relationship {
            subject,
            target,
            kind,
            trust,
            tags,
        } => {
            let day = record.valid.start.ok_or(Error::InvalidInterval)?;
            if subject == target {
                return Err(Error::InvalidInput("self-relationship"));
            }
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
        ObjectData::EventInvolvement {
            event,
            participant,
            role,
        } => {
            let day = record.valid.start.ok_or(Error::InvalidInterval)?;
            require_event(tx, object.world, *event, branch, revision, day)?;
            require_person(tx, object.world, *participant, branch, revision, day)?;
            if let EventRole::Custom(definition) = role {
                if definition.world != object.world {
                    return Err(Error::WrongWorld);
                }
                let definition =
                    resolve_object(tx, *definition, branch, AuthorRevision(revision), day)?
                        .ok_or(Error::NotFound)?;
                if !matches!(
                    definition.record.data,
                    ObjectData::Definition {
                        definition: Definition::Role,
                        ..
                    }
                ) {
                    return Err(Error::WrongKind);
                }
            }
            ("event_involvement", &EventDate::Unknown, None)
        }
        ObjectData::ActorAction {
            event,
            actor,
            target,
            action,
            intent,
            ..
        } => {
            let day = record.valid.start.ok_or(Error::InvalidInterval)?;
            validate_text(action, "action kind")?;
            validate_text(intent, "action intention")?;
            require_event(tx, object.world, *event, branch, revision, day)?;
            require_person(tx, object.world, *actor, branch, revision, day)?;
            if let Some(target) = target {
                require_visible(tx, object.world, *target, branch, revision, day)?;
            }
            ("actor_action", &EventDate::Unknown, None)
        }
        ObjectData::Claim {
            subject,
            predicate,
            statement,
            audience,
        } => {
            if let ClaimAudience::Character(reader) = audience {
                require_person(
                    tx,
                    object.world,
                    *reader,
                    branch,
                    revision,
                    record.valid.start.ok_or(Error::InvalidInterval)?,
                )?;
            }
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
    if let ObjectData::Relationship {
        subject,
        target,
        kind,
        trust,
        tags,
    } = &record.data
    {
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
                params![
                    object.world.0.as_bytes(),
                    object.id.0.as_bytes(),
                    branch.0.as_bytes(),
                    state_revision,
                    revision,
                    tag.trim()
                ],
            )?;
        }
    }
    if let ObjectData::EventInvolvement {
        event,
        participant,
        role,
    } = &record.data
    {
        let (role, definition) = match role {
            EventRole::Actor => ("actor", None),
            EventRole::Witness => ("witness", None),
            EventRole::Victim => ("victim", None),
            EventRole::Stakeholder => ("stakeholder", None),
            EventRole::Sponsor => ("sponsor", None),
            EventRole::Custom(item) => ("stakeholder", Some(*item.id.0.as_bytes())),
        };
        tx.execute(
            "INSERT INTO event_involvement_versions
             (world_id, involvement_id, branch_id, state_revision, revision, event_id, participant_id, role, role_definition_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(),
                state_revision, revision, event.id.0.as_bytes(), participant.id.0.as_bytes(), role, definition],
        )?;
    }
    if let ObjectData::ActorAction {
        event,
        actor,
        target,
        action,
        intent,
        outcome,
    } = &record.data
    {
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
        audience,
    } = &record.data
    {
        let (audience, reader) = match audience {
            ClaimAudience::Author => ("author", None),
            ClaimAudience::Public => ("public", None),
            ClaimAudience::Character(reader) => ("character", Some(*reader.id.0.as_bytes())),
        };
        tx.execute(
            "INSERT INTO claim_versions (world_id, claim_id, branch_id, revision, state_revision, subject_id, predicate, statement, audience, reader_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(),
                revision, state_revision, subject.id.0.as_bytes(), predicate, statement, audience, reader],
        )?;
    }
    if let ObjectData::Definition {
        definition,
        description,
    } = &record.data
    {
        let (category, reverse, symmetric) = match definition {
            Definition::Relationship {
                reverse_label,
                symmetric,
            } => ("relationship", reverse_label.as_deref(), *symmetric),
            Definition::Tag => ("tag", None, false),
            Definition::Role => ("role", None, false),
        };
        tx.execute(
            "INSERT INTO definition_versions (world_id, definition_id, branch_id, state_revision, revision, category, description, reverse_label, symmetric)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(), state_revision, revision,
                category, description, reverse, symmetric],
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
            "SELECT event_id, participant_id, role, role_definition_id FROM event_involvement_versions
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
                    role: entry.get::<_, Option<[u8; 16]>>(3)?.map(|id| EventRole::Custom(ObjectRef {
                        world: object.world, id: ObjectId(Uuid::from_bytes(id)),
                    })).unwrap_or(role),
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
            "SELECT subject_id, predicate, statement, audience, reader_id FROM claim_versions
             WHERE world_id = ?1 AND claim_id = ?2 AND branch_id = ?3 AND revision = ?4 AND state_revision = ?5",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(), row.get::<_, i64>(0)?, row.get::<_, i64>(11)?],
            |claim| Ok(ObjectData::Claim {
                subject: ObjectRef { world: object.world, id: ObjectId(Uuid::from_bytes(claim.get(0)?)) },
                predicate: claim.get(1)?,
                statement: claim.get(2)?,
                audience: match claim.get::<_, String>(3)?.as_str() {
                    "author" => ClaimAudience::Author,
                    "public" => ClaimAudience::Public,
                    "character" => ClaimAudience::Character(ObjectRef {
                        world: object.world, id: ObjectId(Uuid::from_bytes(claim.get(4)?)),
                    }),
                    _ => return Err(rusqlite::Error::InvalidQuery),
                },
            }),
        )?,
        "definition" => conn.query_row(
            "SELECT category, description, reverse_label, symmetric FROM definition_versions
             WHERE world_id = ?1 AND definition_id = ?2 AND branch_id = ?3 AND revision = ?4 AND state_revision = ?5",
            params![object.world.0.as_bytes(), object.id.0.as_bytes(), branch.0.as_bytes(), row.get::<_, i64>(0)?, row.get::<_, i64>(11)?],
            |item| Ok(ObjectData::Definition {
                definition: match item.get::<_, String>(0)?.as_str() {
                    "relationship" => Definition::Relationship { reverse_label: item.get(2)?, symmetric: item.get(3)? },
                    "tag" => Definition::Tag,
                    "role" => Definition::Role,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                },
                description: item.get(1)?,
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
