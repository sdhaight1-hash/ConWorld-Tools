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

Each issue links its governing design contracts and acceptance gates. The planned phase number is independent of the GitHub issue number. As of the 2026-11-22 research reconciliation, Phases 0–2 (#1–3) are closed; Phases 3–9 (#4–10) remain open. See live tickets for verification and status; the accepted additions below do not reopen or renumber completed phases.

## Roadmap Phase 0 — Architecture baseline and feasibility

**Dependencies:** none. **Design:** confirm v1 scope and code layout; ADRs for identity/temporal model, branch semantics, WebGPU-in-Tauri/fallback, SQLite versioning, security scopes, user-defined types and backups. **Validation:** minimal running desktop shell plus GPU/DOM feasibility spike; document observations. **Exit:** architecture contracts align, no unstated data-model assumptions.

**Research #76 integration:** retain the accepted platform, per-world storage, Rust policy boundary and optional GPU/fallback decisions. No reopening of #1. Later UI and portability verification belongs to its active owners.

## Roadmap Phase 1 — World persistence, time and provenance

**Dependencies:** 0. **Design:** core Rust types, world identity and isolation, SQL migrations, branch/revision model, claims/sources and transaction/error strategy. **Validation:** create/save/reopen/correct historic records; half-open intervals, approximate dates, branch fork, multi-effect rollback. **Exit:** AT-01, AT-14–16, AT-26, AT-33.

**Research #76 integration:** retain stable IDs, temporal axes, transactions, provenance and world isolation. Future #77 release-selection migration reuses these guarantees under #77/#5/#8/#10, not a reopening of #2. Protect the actual released predecessor and verify archive/restore before migration. Distinguish fictional date, recorded revision and publication selection.

## Roadmap Phase 2 — Relationship/Event first vertical slice

**Dependencies:** 1. **Design:** relation definitions, reified edges and tags, event/actor/action/effect contracts, history inspector, view context. **Validation:** Alice↔Beatrice friendship to betrayal, asymmetric trust, event actor and "why" trace in working Rust/UI. **Exit:** AT-03–05, AT-08–09, AT-13 core.

**Research #76 integration:** retain accepted relationship/action/effect contracts. Contextual linking to a person, event or source is an authored reference, not automatic semantic fact creation. No reopening of #3; subsequent authoring and discovery integration belongs to #4/#5.

## Roadmap Phase 3 — Universal query, backlinks and discovery

**Dependencies:** 1–2. **Design:** versioned QuerySpec AST/planner, temporal/permission constraints, FTS5, adjacency, authored refs, alias/mention proposals and cache invalidation. **Validation:** text, date, path, role and provenance query in Node surface; save query; collision does not create a false membership. **Exit:** AT-21–25, AT-35, bounded query AT-23.

**Approved implementation emphasis (research #76; #4):** universal kind-aware entry explorer, compact named compatible pickers and revision-safe contextual creation; shared selection and supported historical context across studios; unified search affordance for identity/alias lookup, full text, saved views and allowlisted actions; article-grouped authorized results with stable element anchors and meaningful snippets, bounded continuation and explicit scope. Separate typed facts, authored links, sources, reviewed mentions and candidates; preserve Unicode source spans and disambiguation. Reuse typed saved queries, provide narrow list/table parity before additional view shapes, and preserve cancellation and 50k focused access. Rank only by permitted data; do not use mixed hidden-corpus scores. #77 reader indices and #78 dashboards are separate gated integrations, not new query engines. Related research RQ-01/02/04/07, ENC-01/02/09–11, F-01–04/07.

## Roadmap Phase 4 — Three-layer entry, composable Canvas and publishing

**Dependencies:** 1–3. **Design:** Node/Canvas/Display contract, layout primitives/component registry, template versioning, typed bindings, publication compiler, nested authorization, accessible fallback. **Validation:** bound RelationshipTimeline and QueryTable in responsive Grid/Stack; hide a subtree and compare author vs public link/search/exports. **Exit:** AT-17–20, AT-22, AT-25, AT-34.

**Approved implementation emphasis (research #76; #5):** deliver coherent Node (Explore), Canvas (Write & design), and Display (Read & publish) workspaces over the same selection, with contextual inspector, keyboard navigation, honest scope, dirty-draft guards and consistent archival-atlas visual language. Node exposes focused typed connections; Canvas gains writing-first composition, component palette, headings/rich text, inline stable references/citations, responsive editing, template previews and pinned slot/binding semantics; Display gains encyclopedia homepage/directory, authorized article titles, contents, infoboxes, link/back navigation and readable editorial presentation. Prefer existing tree/ElementIds, compiler, typed services and custom templates; test shallow vs literal template rebinding and preserve old layouts. Start with one authorized live component and bounded saved-query list/table rendering; additional views require supported fields and policies. Current save/publication semantics must be explicit until #77; named author dashboard configuration belongs to #78, after core workbench readiness. Existing scope and AT exits unchanged. Related RQ-01–09, ENC-01–12, UX/V audits.

## Roadmap Phase 5 — Genealogy, families, dynasties and organizations

**Dependencies:** 1–4. **Design:** parentage kinds, marriages/households, inheritance/succession, institutional structure, memberships/offices and lineage. **Validation:** dated family tree, legal vs biological kinship, organizational succession, member role history, search by time. **Exit:** AT-02, AT-06–07, AT-12.

**Approved implementation emphasis (research #76; #6):** integrate existing genealogy, house/household, institutional membership, office/title and succession services with focused Node navigation, historical dates, encyclopedia templates and authorized read components. Specialized panels reuse shared selection and query services; #78 dynasty/institution dashboard presets may consume these views only once the underlying domain capability is verified. No new genealogy engine or automatic new succession-rule scope. Related RQ-05/08/10.

## Roadmap Phase 6 — Events, conflict mapping and causality

**Dependencies:** 2–5. **Design:** event parts, affiliation network, actor agency/intent, conflict issues/positions/phases, causal attribution and impact traversal. **Validation:** three-sided strike, competing goals, side-switch, event-to-event causes and effects, source-backed conflict biography. **Exit:** AT-08–12 and AT-11 with multi-hop impact budget.

**Approved implementation emphasis (research #76; #7):** expose calendar-aware event/actor editing, historical timelines, sourced causal and multiparty conflict exploration, and linked encyclopedia sections through the existing shared workspace. Preserve accepted/inferred/claimed distinctions and bounded impact. Integrate spatial representations under #68; #78 may consume proven event/conflict views, without duplicating canonical records or inventing causal links. Related RQ-05/08/10, F-06.

## Roadmap Phase 7 — Evidence, perspectives, privacy and branch hardening

**Dependencies:** 1–6. **Design:** canon vs claim vs rumor vs individual knowledge, stronger source/visibility policies, alternate-branch editing and publication audit. **Validation:** leaks through counts, nested queries, metadata, image hotspots, backlinks, cache and unauthorized paths impossible in tests. **Exit:** AT-13–16, AT-20, AT-25, AT-34 plus security fuzz set.

**Approved implementation emphasis (research #76; #8):** make audience, branch, fictional date and recorded revision explicit in each supported surface. Current-publication and character knowledge are separate from player reveal or narrative time. Compile/prune before returning text, headings, navigation, counts, rank, snippets, backlinks, anchors, assets or bound results; reauthorize navigation and caches on policy/release changes. Maintain fail-closed unsupported scopes. #77 adds independently staged author draft and current release selection after A1; restoration cannot restore revoked grants. #78 stays author-only. Existing scope protections apply from first slice, not only at this phase's audit. Related RQ-09, ENC-05/06/08/09/11.

## Roadmap Phase 8 — Synthetic DNA and population history

**Dependencies:** 5, 7. **Design:** synthetic marker/segment model, founder populations, recombination versioning, genetic-relative inference, uncertainty and derived-cache invalidation; separate genetic ancestry from culture. **Validation:** reproducible segments, hypothetical match report, remote kin may match no segments, biological edit recomputes derived data. **Exit:** AT-28–30.

**Approved implementation emphasis (research #76; #9):** retain synthetic model, versioning, provenance and genetic/cultural separation. Present verified population and ancestry results through shared query/read components and appropriate authorized views when available; independent of #78. No new genetics behavior or simulation scope.

## Roadmap Phase 9 — Hardening, export, scale and optional integration

**Dependencies:** 0–8 for full phase; export/backup basics should begin at Phase 1. **Design:** portable world archive, offline resilience, GPU fallback, accessibility, renderer/query limits, profiling and optional AI-NPC connector specification. **Validation:** complete import/export, fresh install, backup/recover, focused 50k-world query, no GPU, no AI, documented hardware performance. **Exit:** AT-23–24, AT-27, AT-31–32, AT-36 and consolidated acceptance suite.

**Approved implementation emphasis (research #76; #10):** verify local-first author archive, distinct authorized reader export, installer/offline/fallback and accessibility. Coordinate #77 migration, explicit release preservation/revocation, index rebuild and restored history; #78 local configuration persistence/restore or optional author-only archive section only after readiness decision. Measure actual mixed published corpora and task workflows (not synthetic SQL alone), focused large worlds, query/compile/render time, memory/bytes, keyboard and 460x540/1080x760/1920x1080/200% zoom behavior, on native Windows/Linux. Preserve existing phase exits and independent feature gates; #77/#78 do not become release prerequisites by implication.

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

## Approved research #76 — coordinated functional and interface integration (2026-11-22)

[Owner approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/76#issuecomment-7079741458), [research disposition](Research-Studies/comparative-research-ticket-disposition-2026-11-22.md) and [comparative study](Research-Studies/comparative-worldbuilding-workspaces-and-encyclopedia-ux.md) govern this refinement. The [functional](Functionality-Audit-2026-11-22.md), [interaction](UI-UX-Audit-2026-11-22.md), [visual](UI-Visual-Design-Audit-2026-11-22.md) and [encyclopedia](Encyclopedia-Audit-2026-11-22.md) audits form one coordinated integration effort, not three new editors or independent stores. The research is reviewed and its direction accepted; comparative native author tasks remain outstanding. Neither the isolated model assertions nor SQLite study are application acceptance evidence.

### Accepted cross-phase architectural evolution #77

[Independent Editorial Draft and Publication Lifecycle](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/77) — [accepted design](Architectural-Evolutions/independent-editorial-draft-and-publication-lifecycle.md), [ADR-014](ADRs/adr-014-independent-editorial-release.md). A1–A5 are evolution-specific gates, not new original AT exits.

- #4 (Phase 3): release-dependent authorized article search, references, anchors, pagination and rebuild parity.
- #5 (Phase 4): revision-safe draft/edit/restore commands and explicit release/withdrawal; separate latest author layout from current released immutable version only after A1 and reconciled schema/wire/time contract. Until then explain current combined save behavior; do not silently promise staged releases.
- #8 (Phase 7): current release authorization, noninterference, revoke/cache invalidation, no implicit permission revival on historical restore. No wider branch or audience access until supported.
- #10 (Phase 9): protected migration from actual predecessor, stable IDs/pins/history, backup/restore and archive/export verification. Phase 1 foundation is reused, not reopened.
- Readiness A1: show actual staged editing need against explicit existing actions, finalize reviewed contract and migration before production. Stop/reconsider if current behavior suffices or policy/compatibility fails. Do not delay ordinary dirty guard/context fixes.

### Accepted cross-phase feature #78

[Configurable World Management Workspaces and Dashboards](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/78) — [accepted design](Features/configurable-world-management-workspaces-and-dashboards.md). B1–B5 are feature-specific gates, not new original AT exits.

- #4/#5 provide stable shared context, query pins and paginated views, fixed workbench, dirty guards and reusable author panels. First deliver existing ordinary saved views and contextual navigation.
- #78 owns durable named per-world author-only workspace configuration, bounded panel arrangement and reopen/default recovery. Initial panel types are saved-query list/table and compatible entry shortcuts; no duplicate query engine, semantic object type, public dashboard, arbitrary scripts/formulas or fourth canonical entry layer.
- #6/#7/#68 provide only already verified domain panels for later dashboard presets. #8 owns privacy/context boundary; #10 owns persistence/portability/offline/native evidence. #70 readiness WT-01 is independent; dashboard is not a recipe runner.
- Readiness B1: compare fixed explorer + saved links and ordinary Canvas page against proposed workspace for equivalent actual dynasty, institution and research/writing tasks. Record task benefit and bounded refresh/persistence budgets before implementation; defer if simpler existing tools suffice. Independent of #77; do not make either ticket an artificial dependency of the other.

### Immediate dependency-ordered execution

This is the approved integration emphasis, not a replacement phase sequence or an authorization to bypass separate readiness:

1. Review frozen ontology consolidation [#75](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/75) and #72 gates; reconcile verified recovery and integration fixture branches deliberately, not by mechanical merge. Preserve #67/#73/#74 hold/supersession review and #68/#70 constraints. No merge or gate closure implied by this document.
2. Address high-risk existing gaps: shared supported context, revision-safe commands, unsaved-draft protection and complete kind/picker coverage (#4/#5/#8/#72). Unsupported branches/results are visibly unavailable, not silently substituted. No new schema just for UI polish.
3. Deliver the coordinated shell and visual integration, contextual explorer/search/selection, and encyclopedia vertical slice (#4/#5): create, locate, compose, link, save/reopen, read and return without loss. Apply visual tokens and accessible controls with genuine native evidence.
4. Complete writing-first Canvas, safe template bindings/preview, citations, scoped article search/anchors, and essential dynamic/query presentation under existing contracts. Evaluate A1 for #77 on the usable baseline; execute its independent release implementation only after readiness.
5. Complete shared saved-query list/table and authorized narrow bound components, then B1 for #78. Add named workspace configuration only if approved readiness is demonstrated. Additional timeline/gallery/graph views consume actual compatible authorized fields and domain services.
6. Finish and integrate genealogy, event/conflict, perspective, synthetic genetics, spatial/cartography, portability and release validation through their existing phase/feature owners. This sequencing guides available slices; do not postpone already needed domain prerequisites until after dashboards. #68 SP-01–14, #70 WT-01–10, #72 AE-01–09, #77 A1–A5 and #78 B1–B5 retain their independent authority and stop conditions.

### Verification and scope limits

Use original Northbridge and verified Alder Basin fixtures; expanded workflow/linked article fixture must be verified before measurement. Record actual stable identity, canonical edit/authorized projection parity, native task actions/time/errors, keyboard focus, draft conflict/revocation, archive/reopen, offline and representative corpus performance. Capture authentic UI in Development-Screenshots per its guide when implementation is exercised. No numerical application claim based solely on the study. All existing AT gates, phase numbers, dependencies and accepted invariants remain unchanged.

Reusable linked authored sections, selective cross-world content/template interchange and independently curated reading collections are endorsed future considerations, not approved production-ready specifications. Prefer anchors, ordinary links, templates and saved queries first; new scope requires its concrete readiness and decision record. No general database/schema/formula/plugin runtime, cloud dependency or duplicate canonical store.

