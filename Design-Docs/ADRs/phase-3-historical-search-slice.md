# Phase 3 — Saved-World Historical Search Slice

**Status:** Initial historical name/backlink search merged in [PR #20](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/20) at `93036d0`, verified by [Linux/Windows CI run 44872977952](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44872977952). Bounded relationship path UI is the follow-up, pending validation. **Tracks:** [Roadmap Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4). **Base audited:** `main` at `cce6ec9a552885da5e26684f208ffd63a72dff57` (2026-11-21 UTC).

## Audit and existing boundaries

The full repository tree, readme, [design index](../README.md), [decisions](../12-Decisions-and-Reconciliation.md), [requirements](../01-Requirements-and-Invariants.md), [query specification](../07-Search-Query-and-Discovery.md), [storage contract](../08-Storage-and-API-Contracts.md), [roadmap](../10-Delivery-Roadmap.md), [verification](../09-Acceptance-and-Verification.md), GitHub issues/PRs, [development evidence](../../Development-Screenshots/Dev-Screenshots.md), store migrations, desktop handlers/frontend, tests and CI workflows were checked before changing code. Roadmap Phases 0–2 are closed; Phase 3 is open. PR #17 merged reusable query foundations, not the full query/search experience.

Before this change, `crates/atlas-store/src/query.rs` already implemented versioned `QuerySpec` with exact/substring label matching, explicit backlinks, typed event-role filters, and bounded relationship traversal. It obtains an author-only, branch/day/revision-pinned snapshot via `Store::objects_at`, and rejects public/character reads rather than exposing unfiltered canonical data. An existing duplicate-name regression ensures identical labels remain separate IDs.

Important limits: `objects_at` has a 2,000-visible-object cap and a 10,000-candidate scan cap. It returns an explicit budget error, not a cursor. Search is not yet indexed by FTS5, and this slice cannot claim scalable pagination or a universal QuerySpec AST. The Node graph already shows people, relationships and events only; sources, claims, involvement records and actions do not have standalone graph nodes.

## Narrow vertical decision

Expose **the existing Rust query service**, not a second frontend search implementation:

- A trusted, author-only `workspace_search` Tauri command accepts world, normalized fictional day, optional author revision, exact/substring/backlink match mode, search text or target ID, and result budget.
- Rust loads the current world, constructs the same explicit `ViewContext`, validates query schema and budget via `Store::execute_query`, and serializes stable IDs, labels, typed record categories, source revision and truncation.
- Desktop HTML renders keyboard-accessible results; duplicate names show their separate stable IDs. Only records already present in the graph have active navigation buttons. Non-graph records remain readable text and never promise a nonexistent graph selection. Every search hit also offers a read-only **Find references** action backed by the typed `Backlinks(ObjectRef)` filter.
- Any world/day/revision change clears stale results and invalidates outstanding asynchronous requests. No search result or inferred mention mutates the world.
- Search remains **author-only**. Do not reuse this command as Display/public search. Phase 7 hardens per-audience prefiltering, and Phase 3 still needs a policy-aware search engine before public exposure.

No new migration, external dependency, renderer change, or alternate source of truth is introduced.

## Verification / acceptance map

- **AT-21 / AT-35 partial:** two people with the same label are separate search hits; selecting one uses its unique object ID. Backlink lookup is stable-ID based rather than ambiguous text based. No mention discovery implemented.
- **AT-23 partial:** `QuerySpec` validates the result budget and preserves truncation/error behavior. General cancellation and scalable graph planner remain open.
- **Temporal/revision:** search reads one world/day/revision context and returns the chosen revision, including older archived author history.
- **UI:** Playwright mock-native bridge tests search payloads, results, navigation and invalidation on revision change. This is **not** a native Tauri screenshot or live Rust IPC proof.
- **Backend:** Rust desktop test covers author revisions, duplicate names, invalid input and result budgets.

Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, desktop `npm run build`, and Playwright browser fallback tests. Record observed CI results in PR #20 and #4. No actual desktop screenshot or recording was available in this tool session, so none is claimed or fabricated.

## Remaining Phase 3 work (issue #4 stays open)

1. Agree and implement the richer validated/query-planned AST and source/field allowlist across objects, roles, events, date, provenance, and authorized graph paths.
2. Add FTS5 plus rebuildable search indexes with transactional invalidation, incremental/rebuilt equivalence checks, and measured 1k/10k/50k performance.
3. Implement durable versioned saved queries and typed bindings, including scope and dependency invalidation.
4. Add explicit link/backlink indexes, alias collision proposals, mention provenance, approval/rejection and draft ghost nodes; never infer guild membership automatically.
5. Expand the Node query builder to typed event-role, path and provenance filters (beyond the current backlink shortcut); add robust pagination, query cancellation and UI feedback.
6. Only expose authorized public/character search after prefilter/count/snippet/caching side-channel tests pass; preserve a single Rust query authority across Node/Canvas/Display.

**Migration impact:** none in this slice. Future FTS5 and saved-query tables require versioned migrations and upgrade/reopen tests. **Status integrity:** code committed and automated checks pending; no claim of full Phase 3 or native Windows acceptance.

## Follow-up slice — bounded Node relationship paths

The existing `QueryFilter::RelationshipPath` already implements directed bounded BFS over the selected historical snapshot. Surface that read via a typed author-only Tauri command (`workspace_query_path`) and HTML controls for starting person, relationship kind, direction, maximum hops (1–16), visited-node budget (1–2,000), and result limit. Reuse the original query projection and accessible results view; the frontend never traverses relationships or infers new facts. Changing world/day/author revision clears stale results. This involves no migrations and leaves the original FTS5/plan/saved-query gaps unchanged.

**Acceptance:** backend test covers outgoing/incoming path, historical day, old author revision, visit-budget truncation and invalid input. Playwright mock-native test verifies IPC parameters, selection, truncation notice and view invalidation. Run native CI and note that a mock-native Playwright test is not an authentic native search screenshot. Stage under Phase 3 issue #4; do not claim universal query completion.
