# Phase Two — Relationship and Event Agency Slice

**Status:** Initial implementation contract; partial Phase 2, see [Roadmap Phase 2 issue #3](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3). **Date:** 2026-11-20.

## Existing foundation reused

Build on [Phase 1 persistence](phase-1-persistence-slice.md): `Store::apply` provides a world-scoped, author-revision-pinned transaction, optional stable sources, fork isolation and `EventEffect` provenance. No second graph store or separate timeline database is introduced. The new kinds use the same object registry, `object_versions`, valid time, author audit ledger and existing `get_object` / `effect_on` reads.

## Scoped design decisions

- A **Relationship** is an addressable object with stable, directed `subject` and `target`, kind (e.g. friendship or political alliance), optional 0–100 `trust`, and dated tags. A separate object stores the reverse point of view. Changing endpoints or kind requires a *new* relationship object; `RecordState` and `EventEffect` may modify tags/trust over time. Tags are case-insensitively unique on each authored state; a record's history is preserved by the inherited Phase 1 revision model.
- **EventInvolvement** is a dated, independently addressable role link (actor, witness, victim, stakeholder, sponsor). A witness is not automatically an actor.
- **ActorAction** explicitly identifies the actor, source event, optional target, action, intent and attempted/succeeded/failed outcome. Do not automatically infer that an attempt succeeded or changed any relationship.
- The initial `person` endpoint validation and exact-event-date requirement are intentional Phase 2 boundaries. Organization affiliations, extended event durations, custom role catalogs, relationship-definition registries, psychological dimensions, rich beliefs and viewer-specific secrecy remain for their planned phases.
- Every new typed object resolves referenced IDs on the current world/branch/date, validates endpoint kinds and requires an explicit valid-start date. Uncertain/unknown-date events cannot assert precise action involvement under this initial contract. The strict rule is an integrity guard, not a rejection of uncertain events in the larger system.
- Migration `0006.sql` extends the typed registry and stores normalized versions/tags/actions in foreign-key-checked, STRICT SQLite tables. Old world IDs, valid-time histories and sources survive migration. UI-facing histories must be read from Rust projections rather than direct SQL in TypeScript.

## Fixture and acceptance

The Northbridge integration fixture creates Alice and Beatrice with separate directional friendship states. A sourced betrayal event includes Beatrice as actor, Alice as witness and a recorded action. The action-attributed atomic event-effect command changes Alice's trust and friendship tags, without rewriting Beatrice's view. Later political cooperation is a separate dated relationship object rather than an erasure of the estrangement. Tests exercise historical and author-revision reads, reopen, broken references, invalid tags/kinds, immutable linkage and rollback.

**Progress on Phase 2:** The desktop now embeds an **author-only, read-only Node/relationship history inspector** backed by an ephemeral Northbridge fixture stored in the actual Rust `Store` and queried through typed Tauri IPC. The date control reconstructs independent directional trust, dated relationship tags, the 1908 event/actor/action/source and the 1914 alliance. This is a working historical projection, not a persistent character editor or publication engine. The native smoke adds a `history_verified` gate; the browser-only preview does not receive native author data.

**Still required for full Phase 2:** persistent world editing and source-backed relationship/event inspector beyond the bundled fixture, author/public perspective UX, supported custom relationship/role definitions, and richer source/belief projection. Their absence must not be reported as completion of Phase 2.

## Durable Phase Two authoring workspace

**Implementation follow-up:** The native desktop owns one `AuthorWorkspace` behind author-only Tauri commands. On startup, the trusted host creates its app-data directory and opens `worlds.sqlite` through the existing `atlas-store::Store`. The file is separate from the ephemeral Northbridge fixture, but uses the same migrations, IDs, author revisions, historical timestamps, source records, typed relationships and atomic effects. No frontend JSON shadow store or independent history database is introduced.

- `Store::worlds(Scope)` discovers isolated worlds. `Store::objects_at(ViewContext, limit)` reads bounded historical object snapshots in one SQLite transaction, and both reject public scope. Its 2,000-object ID scan is a temporary Phase Two reader; indexed, permission-safe querying belongs to Phase Three.
- The minimal author UI creates worlds, people, directional relationships and dated tags/trust. A sourced historical change atomically writes a `HistoricalEvent`, `Source`, actor `EventInvolvement`, `ActorAction`, and `ActionEffect` for one relationship. Reverse relationships remain independent.
- The historical view reconstructs the relationship and, if an explicit causal effect exists, reads event, actor, action outcome, intent and source citation from Rust. The frontend uses DTOs rather than SQL or inferred causal guesses.
- New state dates must be strictly later than the relationship's prior state. Public/character query contexts are not enabled through these trusted author commands. Full calendar editing, author-revision browsing and more elaborate events are deferred.
- A desktop Rust test creates an on-disk temporary database, adds people/relationship and a source-backed event effect, verifies same-day rejection, closes/reopens the file and checks earlier/later states and citation. The store separately checks bounded listing, world isolation and public denial. Browser preview does not expose native author data.

**Still needed for full Phase Two:** multiple independently authored event actors, author revision selection, configurable relation/role vocabularies, richer claims/knowledge and perspective-aware views. Full publication security belongs to Phase Seven. No development screenshot is claimed without observing an actual running application.

## Phase Two audit and data-integrity hardening (2026-11-20/21)

This pass re-inspected the root README; historical, temporal, agency, storage, query, and verification contracts; current GitHub issues/PRs; screenshot rules; SQLite migrations; store/renderer/calendar crates; Tauri command and capability code; browser forms; and Windows/Linux CI workflows. It does **not** claim full verification of unimplemented roadmap systems or a native user-machine screenshot.

| Risk | Code-confirmed cause | Resolution and acceptance evidence |
| --- | --- | --- |
| High — valid objects omitted | A global SQL ID limit was applied **before** branch/day visibility. Future or other-branch records could consume the quota ahead of visible people. | `Store::objects_at` now filters history before the return limit. Deterministic-ID regression tests prove low-ID future entries no longer eclipse current characters. |
| High — silent incomplete listings | The 2,000-object projection returned an incomplete list without communicating truncation, making missing people appear deleted. | Explicit `Error::QueryBudgetExceeded` when the result limit or the temporary 10,000-candidate scan budget is reached. |
| High — wrong day from blank field | JavaScript `Number("") === 0` silently turned a cleared date field into fictional day zero. | Required HTML date input plus explicit nonempty signed-integer and safe-range validation; browser mock-bridge test ensures no write invocation on blank day. |
| High — archived author revision could be edited | Workspace always projected current author revision, without safely pinning an older one. | Optional revision-scoped snapshot; event attribution reads the same revision; forms are read-only while a revision is pinned. Rust reopen test compares states and cause before/after revision. |
| Known Phase Two scale boundary | Temporary reader scans IDs and resolves historical records individually; it is not yet an indexed, paginated query service. | 10,000-candidate and 2,000-visible-record limits fail explicitly rather than silently; true indexed pagination belongs to Roadmap Phase Three. |

### Test and integration boundaries

- Rust unit/integration coverage: author scope, world isolation, historical visibility, overflow error, event effect rollback, revision history, reopen and source attribution.
- Frontend: browser fallback without native author content; native-command bridge **mock** for missing-day handling and archived-revision read-only behavior. A mocked native bridge is not evidence of a native Windows WebView run.
- CI gates: Rust formatter, Clippy and workspace tests on both platforms, Linux Playwright, and native Windows WebView2 smoke. Link the final successful run to the Phase Two issue only after it completes.
- This pass required no change to SQLite migrations, stable identities, factual authority, publication scopes, or the three-layer architecture.

### Outstanding investigations and limitations

The current author workspace remains narrow: no general node graph editing, full event actor roster, authored relationship type catalog, human-friendly calendar input, evidence/claim browser, published perspectives, public search, or end-user backup/export. Northbridge is still a separate platform fixture. Browsing older author revisions is read-only; new edits commit against the current revision through `Store::apply`. GPU-capable WebView2, device-loss and performance measurement retain separate platform acceptance gates. Development screenshots/videos may only be committed from observations of actual running software.

## Saved world graph and multi-actor event editing

- The graph and its keyboard-accessible inspector now use the saved world's author-scoped snapshot. Selecting a historical day or a read-only author revision refreshes people, directed relationship states, event roles, and recorded causal links together. Northbridge remains available from the world selector.
- The event editor writes one sourced event with zero or more role rows. People may hold several roles or perform several independent actions. Actor rows record intent, optional target, and attempted/succeeded/failed outcome. A relationship effect is optional and requires an explicitly chosen action and explanation. The event, action, and effect write in one transaction.
- Authors can correct a saved event's title, citation, and existing actions, or add roles and actions. The occurrence day and actor identity stay fixed. The relationship inspector shows the recorded before and after versions and the original action attribution and evidence. A separate relationship state editor appends dated trust and tag changes without inventing an event.
- These changes use the existing schema 0006 and stable record IDs. Event corrections and dated relationship-state edits check the editor's expected world revision. The author graph uses the existing 2,000-object snapshot bound and the main branch's normalized days. The editor uses built-in roles. Search, publication, and richer perspectives follow the existing delivery roadmap.

### Verification

- The Rust workspace suite passed all 26 tests, including SQLite reopen, action-specific rollback, stale edits, historical and author revision reads, and source retention after event corrections. Formatting and strict workspace Clippy also passed.
- The frontend production build and Tauri command registration check passed. Chromium headless Playwright passed seven checks, with the optional software WebGPU test skipped.
- In native Linux WebKitGTK 2.54.0, the editor saved a failed action, a successful action and a witness on one event. The inspector followed the resulting effect and showed independent reverse trust. A later political bond preserved the estrangement. The editor saved a correction without rewriting the original attribution and then appended an unscored relationship state without an event. The archived author revision stayed read-only and showed the earlier trust of 90. [Development captures](../../Development-Screenshots/Dev-Screenshots.md#chronological-development-media-index).

## Phase 2 exit slice

- A world has author-versioned definitions for relationship kinds, relationship tags, and event participant roles. Relationship definitions supply a symmetric label or a separate inverse label for person-to-person links. Trust remains directional and can be left unscored. An event's custom role links to a typed definition ID. Built-in Actor continues to require an explicit action with an independent outcome.
- A sourced account is a versioned claim attached to a historical event. Recording a claim leaves the action, its outcome and every causal effect intact. Accounts become known on their effective day and default to author-only. An author can deliberately share the statement with a single character or the public. The author inspector's account selector uses a Rust projection that excludes unshared statements, source citations and audience metadata before serialization. A character gets public accounts plus their own. Non-author projections require the current author revision so an old revision cannot replay a withdrawn public account. General world and graph reads remain author-only.
- Schema 0007 preserves stable IDs, backfills existing claims as author-only, and preserves schema 0006 built-in participation. The catalog is scoped to the world's author history. The registered inverse label changes the interpretation of the incoming relationship; it does not create a second bond or copy trust. Full actor affiliations, extended role histories, multi-character knowledge propagation and publication authorization use their roadmap phases.
