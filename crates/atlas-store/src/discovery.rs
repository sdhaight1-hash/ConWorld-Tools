//! Phase 3 author-only, read-only mention candidates from recorded source claims.
//!
//! Proposals have no write authority: text similarity is not a relationship,
//! membership, parentage, or proven historical fact.

use crate::{AuthorRevision, Error, ObjectData, ObjectRef, Scope, Store, VersionRef, ViewContext};

/// One possible link to a *visible person*, anchored to a precise claim version
/// and a UTF-8 byte span in its original authored statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MentionCandidate {
    pub source: VersionRef,
    pub source_label: String,
    pub statement: String,
    pub target: ObjectRef,
    pub target_label: String,
    pub start_byte: usize,
    pub end_byte: usize,
    /// Number of distinct possible identities for this exact source span.
    pub collisions: usize,
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

        const MAX_COMPARISONS: usize = 100_000;
        const MAX_CANDIDATES: usize = 2_000;
        let mut comparisons = 0_usize;
        let mut rows = Vec::new();
        for (claim, statement) in claims {
            if statement.len() > 10_000 {
                return Err(Error::QueryBudgetExceeded);
            }
            for person in &people {
                comparisons += 1;
                if comparisons > MAX_COMPARISONS {
                    return Err(Error::QueryBudgetExceeded);
                }
                for (start_byte, end_byte) in matching_spans(statement, &person.record.label) {
                    rows.push(MentionCandidate {
                        source: claim.reference(),
                        source_label: claim.record.label.clone(),
                        statement: statement.to_string(),
                        target: person.object,
                        target_label: person.record.label.clone(),
                        start_byte,
                        end_byte,
                        collisions: 1,
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
fn matching_spans(text: &str, label: &str) -> Vec<(usize, usize)> {
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
