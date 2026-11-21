# Phase 3 — Stable bounded author query pages

**Status:** Draft for [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4), stacked on PR #36. No storage schema change.

Rust now exposes an in-process opaque QueryCursor and QueryPage. Cursor fields are private and bind the exact typed QuerySpec (including page size), complete ViewContext, last resolved canonical label and stable ID. Continuation checks author capability and equality with the original query/context before execution. Each page reevaluates the same immutable author revision and locates its previous boundary in the deterministic label/UUID ordered set. Cursor mismatch or a missing boundary is an explicit error. The API makes no use of client-invented offsets, persisted result caches or unrestricted SQL.

The existing public unpaged API preserves its 500-result limit. The page API can internally collect at most 10,000 results under existing independent candidate limits. It refuses bounded graph traversal (visited-budget truncation is not the same as additional complete results) and fails explicitly if the underlying result is incomplete. This is bounded stable result paging, not a SQL streaming or deadline/cancellation guarantee. A later host adapter must retain/validate opaque cursors rather than accepting arbitrary client-authored cursor fields, and clear them when context changes.

Regression covers 525 equal labels with stable distinct IDs, a correction between pages, original historical replay, index rebuild, final page and cursor exhaustion, changed world/day/query/limit/revision, and public denial. Desktop page controls and cancellation are not yet implemented. Exact-head CI required; no native media claimed.
