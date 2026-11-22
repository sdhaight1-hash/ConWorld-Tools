# 10 — Delivery Roadmap: Numbered Design and Implementation Phases

> **Feature scope accepted (2026-11-22 UTC):** [New Feature #68](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/68) received [explicit owner approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/68#issuecomment-7073427305). The [accepted feature specification](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/main/Design-Docs/Features/nested-locations-and-procedural-vector-cartography.md) governs SPA/MAP extensions and SP-01–14 as feature-specific requirements/gates; original roadmap phase exit criteria are unchanged. The earlier hold remains in Git history (8f39572). Draft #67 requires baseline and exact-head design/test review before further coding or merge; no implementation or production verification is implied by scope approval.


**Status:** Accepted sequence; each phase has one GitHub issue titled exactly \`Roadmap Phase N — ...\`. Issue bodies contain accountable design deliverables and exit gates. Links to those issues are added once created.

The roadmap is **dependency ordered**, not an invitation to build all systems in parallel. Every phase first settles its architecture, then implements a tested narrow vertical slice. Preserve the full future model without implementing speculative flexibility.

See the [implementation foundation](Phase-Foundation.md) for current evidence and shared contracts across all phases.

## Active roadmap issues

- [Roadmap Phase 0 — Architecture Baseline & Platform Feasibility](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1)
- [Roadmap Phase 1 — World Persistence, Temporal Model & Provenance](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/2)
- [Roadmap Phase 2 — Relationships, Tags & Event-Driven History](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3)
- [Roadmap Phase 3 — Universal Search, Queries, Backlinks & Discovery](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4)
- [Roadmap Phase 4 — Three-Layer Entries & Composable Canvas](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/5)
- [Roadmap Phase 5 — Genealogy, Dynasties, Households & Organizations](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/6)
- [Roadmap Phase 6 — Event Studio, Actor Agency & Conflict Atlas](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/7)
- [Roadmap Phase 7 — Evidence, Perspectives, Branches & Privacy](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/8)
- [Roadmap Phase 8 — Synthetic DNA & Population History](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/9)
- [Roadmap Phase 9 — Release Hardening, Export & Optional NPC Integration](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/10)

All ten issues start **open** and link to their relevant design contracts and acceptance gates. The planned phase number is independent of the GitHub issue number.

## Roadmap Phase 0 — Architecture baseline and feasibility

**Dependencies:** none. **Design:** confirm v1 scope and code layout; ADRs for identity/temporal model, branch semantics, WebGPU-in-Tauri/fallback, SQLite versioning, security scopes, user-defined types and backups. **Validation:** minimal running desktop shell plus GPU/DOM feasibility spike; document observations. **Exit:** architecture contracts align, no unstated data-model assumptions.

## Roadmap Phase 1 — World persistence, time and provenance

**Dependencies:** 0. **Design:** core Rust types, world identity and isolation, SQL migrations, branch/revision model, claims/sources and transaction/error strategy. **Validation:** create/save/reopen/correct historic records; half-open intervals, approximate dates, branch fork, multi-effect rollback. **Exit:** AT-01, AT-14–16, AT-26, AT-33.

## Roadmap Phase 2 — Relationship/Event first vertical slice

**Dependencies:** 1. **Design:** relation definitions, reified edges and tags, event/actor/action/effect contracts, history inspector, view context. **Validation:** Alice↔Beatrice friendship to betrayal, asymmetric trust, event actor and "why" trace in working Rust/UI. **Exit:** AT-03–05, AT-08–09, AT-13 core.

## Roadmap Phase 3 — Universal query, backlinks and discovery

**Dependencies:** 1–2. **Design:** versioned QuerySpec AST/planner, temporal/permission constraints, FTS5, adjacency, authored refs, alias/mention proposals and cache invalidation. **Validation:** text, date, path, role and provenance query in Node surface; save query; collision does not create a false membership. **Exit:** AT-21–25, AT-35, bounded query AT-23.

## Roadmap Phase 4 — Three-layer entry, composable Canvas and publishing

**Dependencies:** 1–3. **Design:** Node/Canvas/Display contract, layout primitives/component registry, template versioning, typed bindings, publication compiler, nested authorization, accessible fallback. **Validation:** bound RelationshipTimeline and QueryTable in responsive Grid/Stack; hide a subtree and compare author vs public link/search/exports. **Exit:** AT-17–20, AT-22, AT-25, AT-34.

## Roadmap Phase 5 — Genealogy, families, dynasties and organizations

**Dependencies:** 1–4. **Design:** parentage kinds, marriages/households, inheritance/succession, institutional structure, memberships/offices and lineage. **Validation:** dated family tree, legal vs biological kinship, organizational succession, member role history, search by time. **Exit:** AT-02, AT-06–07, AT-12.

## Roadmap Phase 6 — Events, conflict mapping and causality

**Dependencies:** 2–5. **Design:** event parts, affiliation network, actor agency/intent, conflict issues/positions/phases, causal attribution and impact traversal. **Validation:** three-sided strike, competing goals, side-switch, event-to-event causes and effects, source-backed conflict biography. **Exit:** AT-08–12 and AT-11 with multi-hop impact budget.

## Roadmap Phase 7 — Evidence, perspectives, privacy and branch hardening

**Dependencies:** 1–6. **Design:** canon vs claim vs rumor vs individual knowledge, stronger source/visibility policies, alternate-branch editing and publication audit. **Validation:** leaks through counts, nested queries, metadata, image hotspots, backlinks, cache and unauthorized paths impossible in tests. **Exit:** AT-13–16, AT-20, AT-25, AT-34 plus security fuzz set.

## Roadmap Phase 8 — Synthetic DNA and population history

**Dependencies:** 5, 7. **Design:** synthetic marker/segment model, founder populations, recombination versioning, genetic-relative inference, uncertainty and derived-cache invalidation; separate genetic ancestry from culture. **Validation:** reproducible segments, hypothetical match report, remote kin may match no segments, biological edit recomputes derived data. **Exit:** AT-28–30.

## Roadmap Phase 9 — Hardening, export, scale and optional integration

**Dependencies:** 0–8 for full phase; export/backup basics should begin at Phase 1. **Design:** portable world archive, offline resilience, GPU fallback, accessibility, renderer/query limits, profiling and optional AI-NPC connector specification. **Validation:** complete import/export, fresh install, backup/recover, focused 50k-world query, no GPU, no AI, documented hardware performance. **Exit:** AT-23–24, AT-27, AT-31–32, AT-36 and consolidated acceptance suite.

## Cross-phase priorities

- Query foundation is Phase 3, **not postponed until full Canvas**. Canvas bound components depend on it.
- Visibility policy shapes data reads from Phase 1, though adversarial audit is Phase 7.
- Historical event/relationship effects precede conflict modeling.
- Genealogy must work without the genetic simulator.
- A minimal layout schema precedes a full drag/drop visual designer.
- Use only one read query engine and one publication policy enforcement path across three studios.
- "Done" always means implemented and tested; roadmap issue existence does not imply feature completion.

## Accepted feature #68 — spatial/cartographic integration dependencies (not phase exit gates)

[ADR-013](ADRs/cross-phase-spatial-cartography.md) and [13](13-Spatial-and-Nested-Location-Model.md) reconcile research #65/#66. No Phase 10, no automatic reopening #1–#3. Foundation scope is stable Place, exact typed containment/roles, branch/time and shared query (P0); bounded exact/schematic maps and authoring (P1); advanced geographical generation/simulation (P2). Existing accepted order preserved: prior branch/media consolidation and baseline checks before spatial migrations; do not reserve a migration number.

- #4 Phase 3 anchors foundation integration of #1/#2/#3 contracts plus spatial query/discovery (SP-01–06, policy from first query).
- #5 Phase 4 owns exact local geometry and schematic interactive SVG, typed MapView style/recipe, links and fallback (SP-07–09); later constrained proposal generation (SP-10) after relevant privacy and data constraints.
- #6/#7 Phases 5–6 consume residence/asset/institution location and historical event/control/claim effects (SP-12).
- #8 Phase 7 owns adversarial spatial privacy/branch hardening (SP-04/11) across all surfaces including exports and generated layouts.
- #9 Phase 8 population origin/migration (SP-12), no cultural inference.
- #10 Phase 9 archive portability, actual native dense fixture performance and optional accelerated renderer (SP-13/14). Geometry library, uncertainty schema, advanced constraints, tiles/planet projections and GPU choice remain explicit gated decisions.

Full acceptance in [09](09-Acceptance-and-Verification.md). Research findings are not production acceptance evidence.

## Accepted feature #70 — Transform composition integration dependencies

Subsequent explicit owner approval: [#70](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70) and [approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70#issuecomment-7073776197) accept finite typed recipe composition/replay scope under the [accepted feature design](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/main/Design-Docs/Features/composable-worldbuilding-transforms-and-exploration-workflows.md). Readiness Gate A requires verified baseline and demonstrated author-value gap versus existing contextual/saved queries; stop/defer if insufficient. This is separate feature scope, not a change to #69 findings, original phase exits or #68. Dedicated lore workbench remains deferred. No WT production gate passed.

#4/#5 existing contextual actions are readiness prerequisites; #6–#9 verified domain/branch/access services and #10 native/archive tests support feature integration. #68 spatial operators optional after applicable SP gates. WT-01–10 are #70-specific, not original phase exits.

## Issue content template

Each Roadmap Phase issue MUST contain: objectives, prerequisites, design links, architecture decisions required, engineering deliverables, acceptance tests (referencing AT- and INV-/feature IDs), risks/unknowns, non-goals and a checklist including design review, implementation, tests, actual validation evidence and docs update. Prefer subissues for large implementation tasks after the phase design is accepted. Never create duplicate source-of-truth specifications inside issue descriptions.

## Accepted ontology evolution #72 — contract reconciliation

[Accepted Architectural Evolution #72](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72) ([design](Architectural-Evolutions/universal-worldbuilding-ontology-and-governed-semantic-extensibility.md), [ADR-007](ADRs/adr-007-governed-semantic-extensibility.md), [approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72#issuecomment-7074159822)). AE-01–09 production gates remain unpassed; original phase/feature gates unchanged.

#4/#5 original Concept/definitions/query/authoring; #6/#7 typed facts/roles; #8 history and access from first slice; #9 optional genetics unchanged; #10 archive/native evidence. #68/#70 independently gated. No new phase or reopening; #67 hold unchanged.
