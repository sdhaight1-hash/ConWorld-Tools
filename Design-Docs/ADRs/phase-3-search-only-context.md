# Phase 3 — Search-only workspace and consistent historical read context

**Status:** Draft implementation for [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4). Baseline `main 4ce4b37d` after PR #31. Rechecked complete tree, issue/PR status, README, accepted query/temporal/acceptance contracts and screenshot archive. This is a bounded workspace integration slice, not full large-world authoring or AT-22/23/25 acceptance.

## Decision

Previously, `workspace_snapshot` used `objects_at(context, 2_000)`; exceeding its budget prevented all author search even when an indexed query could return a single match. A later alias-catalog budget failure had the same effect. Return an explicit `scene_limited` search-only snapshot on those **typed budget errors only**. It carries the real world, revision and day, with no partial editable graph or authoring collections. All other errors remain errors. The UI hides graph-dependent editing, traversal controls and discovery in this mode rather than interpreting empty arrays as an empty world. Indexed author search, combined query and saved definition controls remain available; unindexed operations may still return explicit budget errors. No changes to canonical records or authorization.

Every read launched from the loaded workspace (saved definitions, search, path, participant events, mention discovery, account preview) now passes the resolved numerical author revision. Latest selection is resolved on refresh, not separately for each IPC call. Revision-mismatched search/discovery responses are rejected before rendering. The graph is hidden while the context changes, and late evidence replies are discarded. A Refresh historical view button explicitly reloads latest or pinned context. Existing author-revision optimistic writes and public/character current-only evidence checks remain unchanged.

## Verification

Rust desktop regression creates >2,000 objects, checks search-only mode, retrieves an indexed result at exactly that revision and returns to a smaller historical scene. Mock-native Playwright exercises empty scene versus searchable records, disabled editing and discovery, saved definition revision, external revision advancement, rejected mismatched replies, explicit refresh and recovery of a historical graph. Existing IPC assertions are updated to require numerical revision. Exact-head CI is required; no native screenshot captured or claimed here.

**Limitations:** This does not implement focused graph exploration or pagination. Editing a large world's records requires the future focused authoring path. The initial bounded scene attempt still occurs; no measured latency/50k claim is made. Public and character query execution remains denied, and cross-studio authorization and cancellation remain open.

## Focused navigation extension

A search-only author can explicitly select a person result by stable ID, enabling the existing relationship and participant-event queries without a full graph. Refreshing the world/day/revision discards focus. Only standard/any role choices are available without a loaded definition catalog; no partial authoring graph is implied. Large-world traversal itself requires the indexed path work in PR #34. Browser regressions exercise the exact IPC identities/revision and focus reset.
