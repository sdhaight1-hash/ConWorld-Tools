# Phase 3 — Focused and incremental historical mention discovery

Status: draft stacked on #41; tracks #4. Schema 20.

Discovery no longer requires a whole-world object snapshot. Only person and claim versions on the selected historical lineage enter typed, indexed candidate enumeration; Rust resolves canonical visibility before matching. Focus on one claim ID to bypass every other claim statement. Existing exact byte spans, ASCII case matching, canonical names, dated main-line aliases, separate ambiguous identities, review decisions, comparison/candidate caps and author-only permissions remain intact. The focused result can be empty and must replace the prior source preview in full.

A conservative revision-delta plan consumes canonical invalidations, alias-version history and mention reviews: at the same fictional day and main branch, changed claims/reviews invalidate only their source IDs; any person/alias change invalidates every source preview because names/collisions can change. Changed context, reversed revision, fork context, or excessive delta requires full refresh. No derived results are persisted; callers must not reuse truncated previews as a complete corpus. QueryControl enables SQLite and matching-loop cancellation for both scan modes. This is targeted recomputation and dependency planning, not asynchronous materialized mention indexing or a guarantee of arbitrary-sized world discovery.

Regression: over 2,100 unrelated source records exceed the old scene budget, but scoped and global discovery preserve both collision identities. Source corrections invalidate one source, old versions replay unchanged, alias revisions require full refresh, deadlines and author scope are enforced, and rebuild reproduces historical results. Native/UI integration and exact-head CI required.

## Desktop integration

The same bounded native job registry handles discovery and ordinary search. The existing discovery command accepts optional source ID and request ID; the author can launch it directly from a claim search result even when the graph is too large to render. The claim-only path skips other claim statements; reviewed links remain version-specific. Refresh and replacement scan cancel pending work and invalidate the client generation. Focus is retained for the immediate rescan following review. The revision-delta planner is a reusable Rust API; no client-side materialized corpus or automatic background indexing is introduced here. Browser regression covers a focused large-world scan, exact request context, cancellation and stale-reply suppression.
