# Phase 3 — Indexed Historical Claim-Statement Search

**Status:** Limited author-only content-index slice for [Roadmap #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4). Audit baseline `main` `ec67e3f61eab6502f2d0719a6d05c02885bdd65d` (2026-11-21 UTC): README, full tree, design/acceptance and decision docs, migrations 0001–0010, code, open issues/PRs, Linux/Windows CI and screenshot index reviewed.

## Decision and constraints

The authoritative SQLite `claim_versions.statement` is already recorded with world, branch, state revision and author revision; store one **disposable FTS5 index** of these exact statement versions. Migration `0011.sql` backfills existing claim versions into `claim_search_documents` with an explicit stable `INTEGER PRIMARY KEY` `doc_id` (not implicit rowids, which may change after `VACUUM INTO`), adds transactional claim-version insert/update triggers and external-content FTS5 triggers. The existing `Store::rebuild_search_index` rebuilds both label and statement indexes atomically without changing author history. No new canonical entities or duplicate studio database.

The shared `QuerySpec` gains `ClaimFullTextPhrase` and delegates to the **same bounded, typed historical FTS resolver** as name searches, using a static allowlisted SQL query and fully quoted, bound literal phrase (no FTS operators, raw SQL or JavaScript). Index hits are candidate version identities only: Rust re-resolves each claim through the selected world/branch/day/revision and rejects obsolete historical versions before including it. The 10,000 raw historical candidate gate and 1–500 result gate return explicit overflow or truncated results; this is not stable pagination.

Native Node search offers a **claim statement** mode. Search results include the author-visible statement and cited source text if currently resolvable; do not confuse a sourced claim with validated canon. Results reuse existing keyboard-operable result navigation and stale-view invalidation. The Tauri command and Rust query authority are author-only; **never forward this author FTS index to public Display**. Public/character search will require prior policy prefiltering, counts/snippet side-channel tests and a separate reviewed projection.

Extend the saved textual query kind allowlist with `claimtext` while preserving stable saved query identity and append-only author revisions; saved results and permissions are not cached. The existing Rust method/type names `save_named_label_query` and `SavedLabelQuery` are historical API names; a later typed AST rework may rename them without coupling this migration to a refactor.

## Rejected alternatives and verification

Scanning every claim in TypeScript or `objects_at` cannot reach a narrow statement in worlds above 2,000 records and duplicates historic/privacy semantics. FTS of only current statements loses author corrections and fork history. Publishing excerpts from the raw author FTS index violates INV-15. The chosen design preserves an indexed author read without prematurely implementing publication policies.

Tests must cover correction vs old revision, fork snapshot, world and day isolation, FTS operator quoting, author-scope refusal, old schema-10 backfill, rebuild, backup/reopen and narrow matches beyond 2,000 objects, plus saved definition replay and browser-mocked author UI. Real native search-specific screenshots have not been captured in this coding environment; do not fabricate them.

**Still open:** indexed aliases, sources/citations as indexed fields, combined typed AST predicates, public filtered indexes, scalable pagination/cancellation, component bindings and 1k/10k/50k benchmark fixtures. Phase 3 remains open.
