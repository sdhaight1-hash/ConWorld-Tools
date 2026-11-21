# Phase 3 — Indexed exact historical aliases

**Status:** Draft for [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4), stacked on indexed relationship traversal PR #34. Schema 17 adds disposable exact alias-name keys keyed by world/alias/version. It does not change canonical alias records, relationships or claim semantics.

Rust generates Unicode lowercase keys (same normalization as existing exact canonical label search), not SQLite's ASCII-only LOWER. Migration and full search-index rebuild derive keys from canonical alias versions; alias writes update the key in the same transaction. No fuzzy matching, transliteration, or locale-dependent casefold claim.

QueryFilter::ExactAlias and the shared typed predicate use the historical key index to retrieve distinct candidate person IDs, with latest alias revision, active flag and half-open fictional-day interval validated before canonical Person projection. World and author scope are checked; aliases remain main-branch metadata until branch-aware policy is implemented. All matching visible identities are returned subject to deterministic label/ID ordering and explicit budgets; no implicit choice or link creation. Top-level and composed queries can be saved in the existing composite AST envelope. Older binaries reject storage version 17.

Desktop provides exact alias-only and combined alias search. Tests target Unicode lowercase, duplicate identity candidates beyond snapshot size, dates, rename, old revision, rebuild, migration and public refusal. CI required. Full alias-aware mention indexing and locale/transliteration remain outstanding. No native capture claimed.
