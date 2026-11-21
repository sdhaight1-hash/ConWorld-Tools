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

The Northbridge integration fixture creates Alice and Beatrice with separate directional friendship states. A sourced betrayal event includes Beatrice as actor, Alice as witness and a recorded action. The existing atomic event-effect command changes Alice's trust and friendship tags, without rewriting Beatrice's view. Later political cooperation is a separate dated relationship object rather than an erasure of the estrangement. Tests exercise historical and author-revision reads, reopen, broken references, invalid tags/kinds, immutable linkage and rollback.

**Still required for full Phase 2:** explicit action-to-effect attribution, relationship/event inspector in the Tauri UI, author/public perspective UX, supported custom relationship/role definitions, and richer source/belief projection. Their absence must not be reported as completion of Phase 2.
