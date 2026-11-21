//! Phase 3 author-only, read-only mention candidates from recorded source claims.
//!
//! Proposals have no write authority: text similarity is not a relationship,
//! membership, parentage, or proven historical fact.

use uuid::Uuid;

use crate::{
    AuthorRevision, Error, ObjectData, ObjectRef, Scope, Store, VersionRef, ViewContext, WorldDay,
    WorldId,
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

impl PersonAlias {
    pub(crate) fn matches_day(&self, day: WorldDay) -> bool {
        self.active
            && self.starts_at.is_none_or(|from| day >= from)
            && self.ends_before.is_none_or(|until| day < until)
    }
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

impl Store {
    /// Examine a single authorized world/branch/day/recorded-revision snapshot.
    /// This is a candidate preview, not automatic link or relationship creation.
    /// The existing snapshot and comparison budgets are deliberate; larger
    /// universes need the indexed mention pipeline planned in Phase 3.
    pub fn discover_mentions(
        &self,
        view: ViewContext,
        limit: usize,
    ) -> Result<MentionDiscovery, Error> {
        if view.scope != Scope::Author {
            return Err(Error::CapabilityDenied);
        }
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput("discovery result limit"));
        }
        let versions = self.objects_at(view, 2_000)?;
        let people: Vec<_> = versions
            .iter()
            .filter(|item| matches!(item.record.data, ObjectData::Person))
            .collect();
        let claims: Vec<_> = versions
            .iter()
            .filter_map(|item| match &item.record.data {
                ObjectData::Claim { statement, .. } => Some((item, statement)),
                _ => None,
            })
            .collect();

        // The current alias registry is main-branch author metadata, and
        // explicitly returns no values on an alternate historical branch.
        let aliases = self.person_aliases(view)?;
        let mut names = Vec::new();
        for person in &people {
            names.push((person.object, person.record.label.as_str()));
        }
        for alias in &aliases {
            if alias.matches_day(view.day)
                && people.iter().any(|person| person.object == alias.person)
            {
                names.push((alias.person, alias.label.as_str()));
            }
        }

        const MAX_COMPARISONS: usize = 100_000;
        const MAX_CANDIDATES: usize = 2_000;
        let mut comparisons = 0_usize;
        let mut rows = Vec::new();
        for (claim, statement) in claims {
            if statement.len() > 10_000 {
                return Err(Error::QueryBudgetExceeded);
            }
            for (person_id, matching_label) in &names {
                comparisons += 1;
                if comparisons > MAX_COMPARISONS {
                    return Err(Error::QueryBudgetExceeded);
                }
                let Some(person) = people.iter().find(|person| person.object == *person_id) else {
                    continue;
                };
                for (start_byte, end_byte) in matching_spans(statement, matching_label) {
                    rows.push(MentionCandidate {
                        source: claim.reference(),
                        source_label: claim.record.label.clone(),
                        statement: statement.to_string(),
                        target: person.object,
                        target_label: person.record.label.clone(),
                        matched_label: (*matching_label).into(),
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
        // Two aliases (or a canonical label and alias) can denote the same
        // person and source span; this is one candidate, not a collision.
        rows.dedup_by(|a, b| {
            a.source == b.source
                && a.start_byte == b.start_byte
                && a.end_byte == b.end_byte
                && a.target == b.target
        });
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
            candidate.disposition = self.mention_disposition_at(view, candidate)?;
        }
        Ok(MentionDiscovery {
            revision: view.revision,
            rows,
            truncated,
        })
    }
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
