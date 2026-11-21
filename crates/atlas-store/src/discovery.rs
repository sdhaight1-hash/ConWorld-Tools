//! Phase 3 author-only, read-only mention candidates from recorded source claims.
//!
//! Proposals have no write authority: text similarity is not a relationship,
//! membership, parentage, or proven historical fact.

use std::collections::{BTreeSet, HashSet};

use uuid::Uuid;

use crate::{
    AuthorRevision, Error, ObjectData, ObjectRef, Scope, Store, VersionRef, ViewContext, WorldDay,
    WorldId, query::QueryControl,
};

/// Explicitly authored historical name metadata for a stable person identity,
/// not a claim or evidence that a relationship exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonAlias {
    pub id: Uuid,
    pub person: ObjectRef,
    pub label: String,
    pub starts_at: Option<WorldDay>,
    pub ends_before: Option<WorldDay>,
    pub active: bool,
    pub recorded_at: AuthorRevision,
}

pub struct SavePersonAlias<'a> {
    pub world: WorldId,
    pub person: ObjectRef,
    pub day: WorldDay,
    pub expected: AuthorRevision,
    pub alias_id: Option<Uuid>,
    pub label: &'a str,
    pub starts_at: Option<WorldDay>,
    pub ends_before: Option<WorldDay>,
    pub active: bool,
    pub command_id: Uuid,
    pub scope: Scope,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MentionDisposition {
    Approved,
    Dismissed,
}

/// An author-reviewed stable-ID reference to one exact source version and
/// UTF-8 text span. Neither decision creates a canonical world relationship.
#[derive(Clone, Copy, Debug)]
pub struct ReviewMention {
    pub view: ViewContext,
    pub expected: AuthorRevision,
    pub source: VersionRef,
    pub target: ObjectRef,
    pub start_byte: usize,
    pub end_byte: usize,
    pub decision: MentionDisposition,
    pub command_id: Uuid,
}

/// One possible link to a *visible person*, anchored to a precise claim version
/// and a UTF-8 byte span in its original authored statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MentionCandidate {
    pub source: VersionRef,
    pub source_label: String,
    pub statement: String,
    pub target: ObjectRef,
    pub target_label: String,
    /// Exact matched name or alias, distinct from the target's canonical label.
    pub matched_label: String,
    pub start_byte: usize,
    pub end_byte: usize,
    /// Number of distinct possible identities for this exact source span.
    pub collisions: usize,
    pub disposition: Option<MentionDisposition>,
}

#[derive(Clone, Debug)]
pub struct MentionDiscovery {
    pub revision: AuthorRevision,
    pub rows: Vec<MentionCandidate>,
    pub truncated: bool,
}

/// Internal, single-slot, author-only preview. Never accepts client rows.
#[derive(Clone)]
pub(crate) struct MentionPreview {
    view: ViewContext,
    source: Option<ObjectRef>,
    limit: usize,
    result: MentionDiscovery,
}

/// Which existing source previews must be discarded between exact views.
/// A full refresh is necessary when the name dictionary or context changes.
/// A source-only refresh replaces ALL candidates for each returned ID,
/// including an empty result when a claim is no longer visible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MentionRefresh {
    Full,
    Sources(Vec<ObjectRef>),
}

impl Store {
    /// Examine only relevant historical person and claim records.
    pub fn discover_mentions(
        &self,
        view: ViewContext,
        limit: usize,
    ) -> Result<MentionDiscovery, Error> {
        self.discover_mentions_inner(view, None, limit)
    }

    /// Incremental preview of one claim; never reads unrelated claim statements.
    pub fn discover_mentions_for_source(
        &self,
        view: ViewContext,
        source: ObjectRef,
        limit: usize,
    ) -> Result<MentionDiscovery, Error> {
        self.discover_mentions_inner(view, Some(source), limit)
    }

    pub fn discover_mentions_controlled(
        &self,
        view: ViewContext,
        source: Option<ObjectRef>,
        limit: usize,
        control: &QueryControl,
    ) -> Result<MentionDiscovery, Error> {
        if view.scope != Scope::Author {
            return Err(Error::CapabilityDenied);
        }
        let preview = self.with_query_control(control, |store| {
            if !(1..=100).contains(&limit) {
                return Err(Error::InvalidInput("discovery result limit"));
            }
            store.validate_query_view(view)?;
            if source.is_some_and(|id| id.world != view.world) {
                return Err(Error::WrongWorld);
            }
            let previous = store.mention_preview.borrow().clone();
            if let Some(previous) = previous
                && previous.source == source
                && previous.limit == limit
                && !previous.result.truncated
                && let MentionRefresh::Sources(changed) =
                    store.mention_refresh_plan(previous.view, view)?
            {
                // Bound delta work. A large delta uses the normal scan budgets.
                let affected: Vec<_> = changed
                    .into_iter()
                    .filter(|id| source.is_none_or(|focus| focus == *id))
                    .collect();
                if affected.len() <= 32 {
                    let mut rows = previous.result.rows;
                    rows.retain(|row| !affected.contains(&row.source.object));
                    let mut claims = Vec::new();
                    for id in affected {
                        store.check_query_control()?;
                        if let Some(claim) = store.get_object(id, view)? {
                            claims.push(claim);
                        }
                    }
                    // One scan enforces one comparison and candidate budget for the delta.
                    let latest = store.scan_mentions(view, claims, limit)?;
                    rows.extend(latest.rows);
                    sort_candidates(&mut rows);
                    let truncated = latest.truncated || rows.len() > limit;
                    rows.truncate(limit);
                    return Ok(MentionDiscovery {
                        revision: view.revision,
                        rows,
                        truncated,
                    });
                }
            }
            store.discover_mentions_inner(view, source, limit)
        })?;
        // Publish the cache only after the entire controlled read succeeds.
        // Never use an incomplete preview as a complete corpus.
        *self.mention_preview.borrow_mut() = if preview.truncated {
            None
        } else {
            Some(MentionPreview {
                view,
                source,
                limit,
                result: preview.clone(),
            })
        };
        Ok(preview)
    }

    fn discover_mentions_inner(
        &self,
        view: ViewContext,
        source: Option<ObjectRef>,
        limit: usize,
    ) -> Result<MentionDiscovery, Error> {
        if view.scope != Scope::Author {
            return Err(Error::CapabilityDenied);
        }
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput("discovery result limit"));
        }
        self.validate_query_view(view)?;
        let claims = if let Some(source) = source {
            let version = self.get_object(source, view)?;
            if version
                .as_ref()
                .is_some_and(|item| !matches!(item.record.data, ObjectData::Claim { .. }))
            {
                return Err(Error::WrongKind);
            }
            version.into_iter().collect()
        } else {
            self.discovery_versions_at(view, "claim")?
        };
        self.scan_mentions(view, claims, limit)
    }

    fn scan_mentions(
        &self,
        view: ViewContext,
        claims: Vec<crate::Version>,
        limit: usize,
    ) -> Result<MentionDiscovery, Error> {
        const MAX_COMPARISONS: usize = 100_000;
        const MAX_CANDIDATES: usize = 2_000;
        let mut comparisons = 0_usize;
        let mut rows = Vec::new();
        for claim in claims {
            self.check_query_control()?;
            let ObjectData::Claim { statement, .. } = &claim.record.data else {
                return Err(Error::WrongKind);
            };
            if statement.len() > 10_000 {
                return Err(Error::QueryBudgetExceeded);
            }
            let names = self.discovery_names_at(view, &mention_prefixes(statement))?;
            let mut seen = HashSet::new();
            for (person, target_label, matching_label) in names {
                self.check_query_control()?;
                comparisons += 1;
                if comparisons > MAX_COMPARISONS {
                    return Err(Error::QueryBudgetExceeded);
                }
                for (start_byte, end_byte) in matching_spans(statement, &matching_label) {
                    // Count distinct identities and spans, not duplicate authored aliases.
                    if !seen.insert((person, start_byte, end_byte)) {
                        continue;
                    }
                    rows.push(MentionCandidate {
                        source: claim.reference(),
                        source_label: claim.record.label.clone(),
                        statement: statement.to_string(),
                        target: person,
                        target_label: target_label.clone(),
                        matched_label: matching_label.clone(),
                        start_byte,
                        end_byte,
                        collisions: 1,
                        disposition: None,
                    });
                    if rows.len() > MAX_CANDIDATES {
                        return Err(Error::QueryBudgetExceeded);
                    }
                }
            }
        }

        sort_candidates(&mut rows);
        let mut group = 0;
        while group < rows.len() {
            let mut last = group + 1;
            while last < rows.len()
                && rows[last].source.object == rows[group].source.object
                && rows[last].start_byte == rows[group].start_byte
                && rows[last].end_byte == rows[group].end_byte
            {
                last += 1;
            }
            for candidate in &mut rows[group..last] {
                candidate.collisions = last - group;
            }
            group = last;
        }
        let truncated = rows.len() > limit;
        rows.truncate(limit);
        for candidate in &mut rows {
            self.check_query_control()?;
            candidate.disposition = self.mention_disposition_at(view, candidate)?;
        }
        Ok(MentionDiscovery {
            revision: view.revision,
            rows,
            truncated,
        })
    }
}

fn sort_candidates(rows: &mut [MentionCandidate]) {
    rows.sort_by(|a, b| {
        a.source
            .object
            .id
            .0
            .cmp(&b.source.object.id.0)
            .then(a.start_byte.cmp(&b.start_byte))
            .then(a.end_byte.cmp(&b.end_byte))
            .then(a.target.id.0.cmp(&b.target.id.0))
    });
}

/// Conservative keys for SQLite's ASCII-lowercased, three-character
/// expression index. Include shorter prefixes for one/two-character names,
/// punctuation and whitespace. SQLite substr on TEXT stops at NUL; include
/// an empty key for a NUL at a starting boundary. Exact matching below still
/// decides case, boundaries and byte offsets.
fn mention_prefixes(text: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut previous = None;
    for (start, character) in text.char_indices() {
        if previous.is_none_or(|c: char| !c.is_alphanumeric() && c != '_') {
            let mut prefix = String::new();
            for c in text[start..].chars().take(3) {
                if c == '\0' {
                    keys.insert(prefix.clone());
                    break;
                }
                prefix.push(c.to_ascii_lowercase());
                keys.insert(prefix.clone());
            }
        }
        previous = Some(character);
    }
    keys
}

/// Byte-stable source spans. ASCII case differences are matched without
/// changing offsets; non-ASCII names currently require exact casing rather
/// than unreliable Unicode lowercasing that would corrupt source positions.
pub(crate) fn matching_spans(text: &str, label: &str) -> Vec<(usize, usize)> {
    if label.is_empty() {
        return Vec::new();
    }
    let mut spans = Vec::new();
    for (start, _) in text.char_indices() {
        let end = start + label.len();
        let Some(found) = text.get(start..end) else {
            continue;
        };
        if found != label
            && !(found.is_ascii() && label.is_ascii() && found.eq_ignore_ascii_case(label))
        {
            continue;
        }
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        let boundary = |character: char| !character.is_alphanumeric() && character != '_';
        if before.is_none_or(boundary) && after.is_none_or(boundary) {
            spans.push((start, end));
        }
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::matching_spans;

    #[test]
    fn exact_boundaries_and_utf8_byte_offsets() {
        assert_eq!(matching_spans("Joanne, Ann! Annette", "Ann"), vec![(8, 11)]);
        assert_eq!(
            matching_spans("Élodie and Élodie", "Élodie"),
            vec![(0, 7), (12, 19)]
        );
        assert_eq!(matching_spans("ana anabelle", "Ana"), vec![(0, 3)]);
    }
}
