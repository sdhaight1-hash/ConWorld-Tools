# Phase 3 — Versioned Author Person Aliases for Mention Discovery

**Status:** Partial DSC-01/LNK-01 and [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4). Audit baseline `main f3af1b963a25fa76e082800ca9a7d8516d244020` included README, domain/search requirements, ADRs, issue/PR state, full tree, migrations 0001–0013, Rust/desktop tests, CI and screenshot archive.

## Decision

Migration `0014.sql` adds world-scoped `person_aliases` keyed by a stable alias UUID and a typed person FK, with append-only `person_alias_versions` carrying an author revision, label, optional fictional-day `[starts_at,ends_before)` interval and enabled flag. These are **author-controlled name metadata**, not canonical identities, new Person objects, source claims or inferred relationships. Changes use optimistic author revision and command UUID; unknown/wrong-world/non-person references, invalid dates/labels and stale revisions fail without advancing audit history. Existing canonical person labels and object history are untouched. Existing world upgrades and SQLite backup preserve aliases.

**Deliberate branch gate:** Like Phase 3 mention reviews, alias metadata currently applies to the main author-history branch only, and a fork sees no alias annotations. This avoids silently leaking a post-fork nickname into historical alternatives; branch-lineage semantics are deferred, not assumed. A previously approved source-to-person link remains tied to its stable identity if an author later edits or disables an alias.

The author-only `discover_mentions` scanner considers historically active aliases **in addition to** the visible canonical Person label, anchored to exact source Claim version and UTF-8 byte positions. Canonical/alias duplicate matches for the same person and span are deduplicated; distinct persons sharing a name remain separate collision candidates. Raw comparison/candidate budgets remain explicit. Approval verifies the exact visible name **or active alias** at selected author revision and fictional day in the same write transaction. Merely discovering a nickname never creates a link; only explicit author approval contributes to backlinks.

The existing native Node author workspace adds a simple create/revise/disable alias form and shows matched nicknames separately from canonical identities in candidate results. Pinned author revisions are read-only. Reader/character/public search remains refused, and aliases are not materialized as an unaudited public index.

## Verification and limitations

Tests cover time intervals, same-name collisions, source span/identity review and backlink, alias changes and deactivation, past author revisions, wrong world, public denial, backup/reopen, schema-13 upgrade, invalid input and transaction rollback. Browser mocked-native regression covers creation, discovery, edit, disable and pinned-read-only UI. No screenshot of native alias UI claimed without a live capture.

**Still open:** alias locales, transliterations and name type registry; branch-aware alias inheritance, Unicode caseless and lexical normalization, indexed mention scanning above 2,000 objects, hotspot discovery, graph ghosts, public permission-filtered search, pagination and full 50k scale fixtures.
