# 12 — Architecture Decisions, Source Reconciliation and Open Questions

**Status:** Reconciliation record · 2026-11-20. This document disambiguates superseded concepts; it is not a replacement for specialist specifications.

## Scope of source consolidation

Source material consisted of the user-directed conceptual discussion and previously generated, increasingly detailed artifacts:

| Source generation | Material incorporated |
|---|---|
| v0.1 master specification + initial AI brief | genealogy and world modeling, relationships, DNA, institutions, events, actors, conflict mapping, acceptance and agent guidance |
| v0.2 foundational architecture and SQL smoke prototype | Rust/Tauri/HTML/wgpu boundary, typed registry, three time axes, branch and revision, SQLite invariants, query and command interfaces |
| v0.3 three-layer entry RFC | Node semantic graph, Canvas templates, published Display, links/backlinks, keyword and image discovery, visibility, draft nodes |
| v0.4 search/query and compositional Canvas RFC | search across **all three surfaces**, typed query AST, saved queries, discovery index, publication filtering, nested layout primitives and query-driven components |
| Follow-up user decisions | explicit emphasis on event affiliations, actors, conflict sides, event causality, relationship tags/timelines and composite layout/component slots |

The older files are historical design inputs, **not alternate live specifications**. This directory rewrites rather than blindly appends them. Traceability is by major concept here and by normative requirement/test IDs in [01](01-Requirements-and-Invariants.md) and [09](09-Acceptance-and-Verification.md).

## Accepted decisions (ADRs to expand when implementation starts)

| Decision | Accepted direction | Rationale / rejected approach |
|---|---|---|
| D-001 | Rust authoritative domain/store/query service | Reject independent truth in frontend, renderer or per-studio databases |
| D-002 | HTML/CSS/TypeScript + Tauri 2 desktop | Fast composable authoring UI and local Rust IPC |
| D-003 | wgpu GPU views via browser-capable Rust/WASM renderer where feasible | Native wgpu does not draw directly into a webview; require fallback |
| D-004 | SQLite, typed records, universal object registry | Reject arbitrary graph DB/microservice as default; keep typed integrity |
| D-005 | Reified, versioned relationships, memberships and event involvements | History, tags and evidence require their own object identity |
| D-006 | Valid world time + author revision + alternate branch | Static "current" relationship records cannot reconstruct history |
| D-007 | Canon vs claims/beliefs/rumors/proposals separated | Avoid silent promotion of discovery or private knowledge |
| D-008 | Events, actors, actions, conflicts and causal effects distinct | Actor goals, stakeholder roles and effects are not interchangeable |
| D-009 | Node/Canvas/Display are views of one entity | Avoid duplicate canonical fields or independent published truth |
| D-010 | Layout primitives contain components; typed query binding | Avoid hardcoded one-off page templates and freeform-only layouts |
| D-011 | Shared search engine in all three surfaces, early implementation | Supersedes older roadmap which delayed comprehensive search |
| D-012 | Auth scopes applied before query/scene/publication | CSS hiding and client filtering leak secrets |
| D-013 | Incremental discovery creates proposals, not canonical edges | Name mention isn't proof of organization membership |
| D-014 | Genetics synthetic/deterministic and optional | No need for actual whole-genome simulator; genealogy works independently |
| D-015 | First vertical slice proves relationship event change across surfaces | Correct core model before expansive UI or generative simulation |
| D-016 | Offline/local-first authoring, optional AI/NPC adapters | Core product usable without external provider |
| D-017 | Stable historical Place, typed temporal/lens-specific containment and optional versioned geometry/frame | Reject fixed tree, geometry-first dependency and duplicated history |
| D-018 | Exact/schematic/draft modes; explicit author promotion of candidate geography | No rendered or inferred geography silently becomes canon |
| D-019 | Shared authorized spatial QuerySpec and immutable MapScene; policy before all derivation | Prevent hidden paths, anchors, geometry, labels and export leakage |
| D-020 | Versioned map recipes/styles, bounded SVG first, accessible fallback and evidence-gated wgpu | Avoid renderer authority and premature GIS/GPU complexity |

See [ADR-013 cross-phase spatial/cartography](ADRs/cross-phase-spatial-cartography.md) and [13 spatial specialist contract](13-Spatial-and-Nested-Location-Model.md), approved reconciliation of #65/#66; production evidence pending.

## Contradictions resolved

1. **Genealogy-centric versus universal graph:** genealogy is a specialist engine on the universal historical knowledge model. Biological parentage remains typed.
2. **Node, Canvas and Display copies versus one source:** Canvas stores view elements only, Display is compiled, Node graph stores meaning; all refer to stable ObjectIds.
3. **"All events are actors" ambiguity:** events are first-class historical occurrences; actors are entities exercising agency *within* events; witnesses/stakeholders can be involved without agency.
4. **"Event affiliations" versus organization memberships:** event involvement may have affiliations/sides; enduring membership has different validity and duties.
5. **"Conflict side A vs B" versus many parties:** conflict positions are issue- and date-specific, allowing neutral/multipolar participants.
6. **Causal timeline versus chronology:** time proximity never proves causality; only explicit link/effect with evidence establishes asserted cause.
7. **Automatically propagating nodes versus correctness:** create candidate/draft ghost nodes and link suggestions; require approval before adding factual genealogy, membership or blame.
8. **Hide from Display versus confidentiality:** hidden content must also be removed from public search, backlink, aggregate, image hotspot and scene outputs.
9. **Pure wgpu UI versus Rust/HTML/wgpu:** HTML owns conventional UI; wgpu accelerates focused visualizations where supported; require validated browser integration and fallback.
10. **Graph database versus SQLite:** typed SQLite plus FTS and bounded traversals first; measure before evaluating graph-store alternatives.
11. **Fixed relationship score versus narrative richness:** optional numerical dimensions, directed per-person; structured tags and prose never require a numerical value.
12. **Query/search late versus foundational:** typed query and indexing precede full Canvas; full visual query builder can be improved later.
13. **Historical events versus app events:** keep event/action/effect objects separate from the audit revision ledger.
14. **Uncertain date versus exact surrogate:** store uncertainty and 3-valued query result; never invent a day from year-only input.
15. **Large one-shot program versus vertical proofs:** coherent architecture now, small tested milestones in issues.

The [Phase Zero architecture baseline](ADRs/phase-0-architecture-baseline.md) records the implemented shell and initial decisions for ADR-001, ADR-002, ADR-005, ADR-006, ADR-007 and ADR-011. The [verification log](Phase-0-Verification.md) records platform evidence and open gates. The [Phase One persistence ADR](ADRs/phase-1-persistence-slice.md) records the initial storage and historical-state behavior.

## Open decisions requiring specific ADRs

| ADR | Question | Time to decide | Required evidence |
|---|---|---|---|
| ADR-001 | View branch overlay semantics; fork anchoring, chronology edge cases | Phase 0–1 | Failing/green branch fixture tests |
| ADR-002 | WebView2 WebGPU availability, Rust/WASM wgpu build, text labels and fallback | Phase 0 | Running Windows desktop spike and fallback observation |
| ADR-003 | Fictional calendars: eras, negative years, leap rules, ambiguous ranges | Phase 1 | Valid/uncertain calendar fixtures and parsing tests |
| ADR-004 | Layout schema evolution and template pin/override rules | Phase 4 | Roundtrip migrations of sample layouts |
| ADR-005 | Policy vocabulary, audience/perspective knowledge rules, secret-count protections | Phase 0/4, harden Phase 7 | Adversarial publish/search tests |
| ADR-006 | Exact SQLite version, STRICT availability, fulltext tokenization and backup plan | Phase 0–1 | Target Windows packaging/tests |
| ADR-007 | Typed custom relationship/category extension and data-validation registry | Phase 1–2 | Cross-kind negative tests |
| ADR-008 | Query path constraints, budgets, index rebuild protocol and query AST migration | Phase 3 | 1k/10k/50k fixtures |
| ADR-009 | Synthetic population founder model, recombination approximation and marker density | Phase 8 | Deterministic test corpus and reported error bounds |
| ADR-010 | Edit/merge policy for previously published/derived records | Phase 3–4 | Alias/link refactoring and publication fixtures |
| ADR-011 | React UI versus minimal HTML and view framework-specific accessibility | Phase 0 | Editor and renderer integration prototype |
| ADR-012 | Simulated branch merger, world import cross-references and external NPC write-back | Later | Explicit conflict test and schema migration plan |

**OPEN** does not permit an AI coder to invent a global policy silently. Propose the smallest coherent decision, test it, then update its ADR/status.
| ADR-013 (implementation details open) | Spatial schema reuse, uncertainty/frame/geometry validation, procedural constraints, renderer and privacy gates | Phases 3–9 | SP-01–14, real native and archive tests; design direction accepted |

## Change control

Any change to an accepted decision must have an ADR with context, alternatives, impact on invariants, migration strategy, test evidence and specific superseded sections. Update this table and affected specialist spec *together*. Preserve original author intent and keep downstream phase issues linked to the canonical repository, `sdhaight1-hash/ConWorld-Tools`.

## Verification limits

The predecessor packages included a foundational SQLite smoke test. It does **not** prove Rust, WebGPU, Tauri, search, publishing or DNA have been implemented. The new repository began empty, and these files are **specifications**, not executable products. No acceptance scenario should be marked passed until implemented and actually run.

## 2026-11-21 implementation reconciliation

The [consolidated development record](Consolidation-2026-11-21.md) links the accepted partial implementation ADRs and test evidence. No normative architecture decision is superseded here: shared historical store, explicit author acceptance, stable identities and Rust projection policy remain authoritative. ADR-004, ADR-005, ADR-008 and ADR-010 are still open at full scope (templates, complete reader-safe queries, character/branch visibility, hotspot and caption indexing, performance and refactor semantics). Schema 26 and its author/reader boundaries are a verified implementation baseline, not completion of these ADR contracts.
