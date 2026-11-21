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
