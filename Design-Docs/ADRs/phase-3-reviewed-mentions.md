# Phase 3 — Version-Pinned Author Decisions for Mention Links

**Status:** Partial DSC-01, LNK-01 and AT-21/35 implementation; tracks [Phase 3 issue #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4) following merged candidate scanner [PR #24](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/24).

## Decision

Schema `0010.sql` stores an append-only, **author-scoped review decision** for a specific claim version, original UTF-8 source span, and stable person ID. Decisions are `approved` or `dismissed`; later reviews at later author revisions supersede an earlier decision without deleting its audit history. Schema foreign keys require same-world, typed claim source, typed person target and an actual recorded source version.

Each author approval is a **hyperlink/reference**, not a membership, parentage, relationship, guilt claim or evidence of the mentioned action. Before writing, Rust re-resolves the claim and person inside one SQLite write transaction, validates both against the chosen day/branch/current author revision, verifies the source version has not changed and verifies the exact name/span is still present. Stale writes fail without advancing the author revision.

The candidate scanner returns historical review state visible at the requested author revision. `QueryFilter::Backlinks` now includes *only explicitly approved* mention links to a currently visible person from currently visible claim versions, and respects superseding dismissals; names may change without breaking already-approved stable links. A source correction invalidates the old approval in newer snapshots. No author-review data is exposed to public/character search (currently refused in Rust); an approved reference is not permission to publish its source text.

No new query language, AI model, arbitrary SQL input, canon-creating proposal action, or alternate source of truth is introduced.

## Verification and limitations

Test collisions, approve/dismiss/reapprove chronology, historic author revision, claim correction invalidation, person rename stability, old schema upgrade, reopen/backup, wrong-world, old version, invalid span, transaction rollback and author-only scope. Browser-mocked native UI tests do not constitute live native Tauri screenshot evidence.

Snooze, aliases, source ranges beyond existing claim statements, image hotspots, real ghost nodes, indexed mention-at-scale, graph-visible approved edges, Canvas/Display binding and audience-safe public search are still open Phase 3/4/7 work. Keep issue #4 open.
