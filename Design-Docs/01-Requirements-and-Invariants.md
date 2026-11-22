# 01 — Normative Requirements and Invariants

**Status:** Normative contract. Each requirement needs a test or explicitly documented validation method.

## A. Cross-cutting invariants

- **INV-01 Stable world identity.** Every addressable object, fact, membership, relationship, actor involvement, action and layout element has stable typed identity. Rename never breaks references; cross-world links require explicit import/crossover.
- **INV-02 Historical preservation.** Ending, superseding or correcting a relationship, tag, title, affiliation or fact MUST NOT erase its earlier valid state or recorded revisions. Distinguish tombstones from historical endings.
- **INV-03 Temporal separation.** Fictional effective time, author-recorded revision time and timeline branch are separate coordinates. Dates may be approximate, ranged or unknown without fabricated precision.
- **INV-04 Epistemic separation.** Author canon, in-world document, rumor, belief, hypothesis and proposal are not interchangeable. An unsourced claim cannot silently override canon.
- **INV-05 Directional social state.** A's feelings about B and B's feelings about A are independent; only explicitly defined symmetric or inverse relationship types use symmetry.
- **INV-06 Explicit cause.** An earlier event is not automatically the cause of a later change. A causal edge has role, rationale, certainty and evidence; simulation effects have audited write provenance.
- **INV-07 Typed participation.** Actor, witness, participant, victim, beneficiary, stakeholder and organizational affiliation are separate; one entity may have multiple roles per event.
- **INV-08 Non-numeric option.** Tags, observations, notes and accounts are usable without fixed psychological scores. If numerical dimensions exist, label source and perspective.
- **INV-09 Generation reproducibility.** Synthetic data depends on a world seed, generator version and stable input configuration. Every generated record has origin metadata and author override policy.
- **INV-10 Atomic mutations.** Related state changes commit/rollback together in Rust/SQLite; indexes and publication caches cannot claim stale history is current.
- **INV-11 Controlled extensibility.** Authors can define relationship kinds, event roles, conflict categories and tags with metadata/validation; schema remains strongly typed for core data.
- **INV-12 Bounded computation.** Graph traversal, query execution, component updates, exports and scene generation have budgets, cancellation and informative truncation. Rendering only the focused subgraph is normal.
- **INV-13 One source of truth.** Nodes are semantic objects. Canvas elements are presentation objects. Published elements are authorized projections; none copies and forks canonical people/events.
- **INV-14 No silent inference promotion.** Text extraction, alias matching, AI proposals or graph proximity never establish a canonical role, parent, membership or cause without approved semantics.
- **INV-15 Visibility is a query constraint.** Hidden fields, objects, citations, image hotspots, backlinks, counts and search facets MUST be excluded before data leaves the Rust authorized projection.
- **INV-16 Import/export fidelity.** World bundles preserve IDs, branch/time/revision metadata, provenance, sources, content, layout bindings and ruleset versions; imports validate and report incompatibility.

## B. Functional requirements

| ID | Requirement | Validation |
|---|---|---|
| REL-01 | Multiple coexisting typed relationships among arbitrary compatible objects | Two different relationships on one pair remain independently addressable |
| REL-02 | Effective-dated, source-aware tags on objects, edges, events and memberships | Query older date shows expired tag; current date does not |
| REL-03 | Relationship histories, directional state and change explanation | Inspector shows event, affected state before/after, evidence, missing-cause warning |
| REL-04 | Graph lenses: genealogical, social, institutional, wealth, politics, belief, DNA, event impact | One context preserves branch/date/audience across lenses |
| GEN-01 | Biological, legal, adoptive, step, foster, guardian, chosen family, marriages, households, dynasties, inheritance | Contradictory public record coexists with known biological ancestry |
| GEN-02 | Ancestor/descendant, common ancestor, degree of relation, surname/house branch, kinship/path queries | Correct directed traversals without permanent all-pairs edges |
| DNA-01 | Deterministic synthetic genome segments, autosomal recombination, founder populations, relative matching | Re-run stable seed; distant documented kin may share zero detectable markers |
| DNA-02 | Genetic ancestry reports, chromosome painting, Y-DNA and mtDNA when applicable, evidentiary mismatch reports | No inference of citizenship, religion, culture or personality from genetics |
| ORG-01 | Organizations, factions, houses, guilds, churches, governments, clubs and institutional succession | Schism and mergers maintain historic names and memberships |
| ORG-02 | Time-bounded roles, offices, secret memberships, approvals, allegiances, and group-group relations | Affiliation at two dates correctly differs |
| EVT-01 | Events with uncertain durations, hierarchical parts, locations, tags, evidence and linked events | Multi-part sequence navigable both directions |
| EVT-02 | Event actors, stakeholder roles, affiliations, actions, goals, knowledge, attempts, outcomes | Failed sabotage is not falsely recorded as succeeded |
| EVT-03 | Causal/contributory/retaliatory/enabling links and direct effects to typed changes | Impact explorer does not infer cause from proximity |
| CON-01 | Conflicts with issue-specific multipolar positions, factions, negotiations, phases and events | Side-switch and neutrality time-query correctly |
| CON-02 | Conflict evolution, maps and biographies, direct versus indirect impact | Query shows supported causal chain and uncertainty |
| EVD-01 | Canon, claims, testimonies, public narratives, personal knowledge, conflicting evidence | Character perspective never reveals unknown secret |
| ENT-01 | Every world entry has Node, Canvas and Display representations | Changing one fact updates bound views, without copying fields |
| CAN-01 | Responsive layout primitives containing components with typed query bindings | Move component between Stack/Grid without changing binding |
| CAN-02 | Templates, slots, breakpoint variants, hide/omit rules and versioned layouts | Public compile strips hidden children and references |
| LNK-01 | Stable hyperlink targets in prose/images, hotspots, aliases and automatic backlinks | Rename preserves links; hidden backlinks don't leak |
| DSC-01 | Candidate detection, disambiguation, approval and draft ghost-node proposals | Mention of guild never silently asserts membership |
| QRY-01 | Text, structured historical, graph-path, aggregation and temporal search across all three studios | Same typed query respects selected world/branch/date/revision |
| QRY-02 | Save parameterized queries and bind to Canvas components | Widget refreshes when dependencies change |
| QRY-03 | Audience-safe display search, facets, snippets, exported indexes and graph navigation | Side-channel tests verify no withheld data leakage |
| IMP-01 | Offline portable, versioned, backup-compatible project bundles | Import/export semantic equality and missing-assets report |
| NPC-01 | Optional typed external NPC adapter, no dependency on an LLM | Offline authoring still complete with adapter disabled |
| SPA-01 | Stable Place identity and optional location type/geometry, typed historical containment/placement; independent lenses and branch/revision | SP-01–05 |
| SPA-02 | Spatial queries use shared bounded authorized engine; independent roles, optional frames and unresolved locations | SP-05–06, SP-11–12 |
| MAP-01 | Exact and clearly marked schematic vector maps, nested navigation, typed recipes and accessible fallback as derived views | SP-07–09 |
| MAP-02 | Bounded reproducible constraints-based draft geography, no silent canonical promotion; explicit transactional approval | SP-10–11 |
| MAP-03 | Safe map publication/export, optional GPU, portable recipes/assets, scoped noninterference and measured performance | SP-09/11/13/14 |

## C. User-facing interaction requirements

- **Node Studio:** create incomplete nodes, connect typed relations, traverse/filter by time or path, select graph layouts, inspect proposals and backlinks, search nodes and edges independently.
- **Canvas Studio:** template/element palette, responsive Grid/Stack/Flex/Panel/SplitPane/Overlay, bound data components, content picker, in-page find, query editor, image hotspots and visibility previews.
- **Display Studio:** read-only public/character/author previews, contextual navigation, permitted search, historic snapshot selector, source detail when authorized, accessible non-GPU equivalent.
- **All studios:** keyboard navigation, undo/redo, stable deep links, clear unknown/uncertain flags, errors rather than fabricated results, and shared ViewContext.

## D. Prioritization

**P0 (foundation):** INV-01 through INV-16 except generation implementation, REL-01–03, EVT-01–03 limited vertical slice, ENT-01, basic CAN-01/02, LNK-01, QRY-01/03, EVD-01, IMP-01 scaffold, SPA-01/02 (stable identity, historical containment and scoped query foundation).

**P1 (core product):** full GEN, ORG, CON, advanced Canvas components, visual query builder, conflict/genealogy views, robust evidence and publication, MAP-01 and basic MAP-03 (bounded maps, navigation, safe export).

**P2 (simulation):** DNA-01/02, population dynamics, generative histories, enhanced geography (advanced frames, constrained procedural generation MAP-02, optional acceleration/tiles in MAP-03), AI-NPC bidirectional integration.

Do not interpret P2 as permission to omit data-model hooks, IDs, schema-versioning or test seams now.

See [13](13-Spatial-and-Nested-Location-Model.md) and SP acceptance gates in [09](09-Acceptance-and-Verification.md). Basic spatial modeling is no longer deferred with enhanced geography; phase ownership remains in [10](10-Delivery-Roadmap.md).
