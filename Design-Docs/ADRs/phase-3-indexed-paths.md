# Phase 3 — Indexed historical relationship traversal

**Status:** Draft under [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4), based on integrated compound-planner branch. No phase exit claimed.

## Design

Previously `RelationshipPath` built an adjacency map by enumerating every visible world object. That path fails above the 2,000-object snapshot cap even with only one relevant edge. Use the existing outgoing endpoint index and migration 0016's reverse endpoint index (no canonical record changes). At each BFS expansion select bounded distinct edge IDs by current endpoint and direction, in one SQLite read transaction. Resolve each edge's authoritative version at the selected world/branch/day/author revision; recheck its current endpoints and kind because the index contains historical versions. Resolve each eligible neighbor before admitting it to the walk; never traverse an obsolete or unavailable endpoint. Scope remains author-only.

Stable UUID ordering preserves deterministic BFS truncation. Depth is a semantic maximum; the visited budget counts the starting identity; a result limit is applied after traversal and label/ID sort. A separate aggregate 10,000 raw edge-candidate budget returns explicit error rather than presenting uncertain partial reachability as complete. No second graph authority and no materialized mutable graph copy.

## Verification

Regression creates >2,000 unrelated people and checks outgoing, incoming, both directions, depth and visited/result budgets, fictional end dates, recorded corrections, fork isolation, wrong world and audience, schema-15 upgrade and backup. Reuse existing Phase 3 foundation and desktop path tests. Exact-head CI required. Native UI integration for search-only stable-ID path entry is a separate follow-up; no screenshot claimed.
