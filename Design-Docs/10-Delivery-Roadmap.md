# 10 — Delivery Roadmap: Numbered Design and Implementation Phases

**Status:** Accepted sequence; each phase has one GitHub issue titled exactly \`Roadmap Phase N — ...\`. Issue bodies contain accountable design deliverables and exit gates. Links to those issues are added once created.

The roadmap is **dependency ordered**, not an invitation to build all systems in parallel. Every phase first settles its architecture, then implements a tested narrow vertical slice. Preserve the full future model without implementing speculative flexibility.

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

## Issue content template

Each Roadmap Phase issue MUST contain: objectives, prerequisites, design links, architecture decisions required, engineering deliverables, acceptance tests (referencing AT- and INV-/feature IDs), risks/unknowns, non-goals and a checklist including design review, implementation, tests, actual validation evidence and docs update. Prefer subissues for large implementation tasks after the phase design is accepted. Never create duplicate source-of-truth specifications inside issue descriptions.
