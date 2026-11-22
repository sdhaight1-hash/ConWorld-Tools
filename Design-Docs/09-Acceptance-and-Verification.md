# 09 — Acceptance and Verification

> **Feature scope accepted (2026-11-22 UTC):** [New Feature #68](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/68) received [explicit owner approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/68#issuecomment-7073427305). The [accepted feature specification](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/main/Design-Docs/Features/nested-locations-and-procedural-vector-cartography.md) governs SPA/MAP extensions and SP-01–14 as feature-specific requirements/gates; original roadmap phase exit criteria are unchanged. The earlier hold remains in Git history (8f39572). Draft #67 requires baseline and exact-head design/test review before further coding or merge; no implementation or production verification is implied by scope approval.


**Status:** Executable test plan to implement; not a report of completed tests.

## Golden fixture: Northbridge

Create one deterministic fictional world with:
- Alice Vale, Beatrice Thorne and two families, including biological versus adoptive kinship and a disputed public record.
- A friendship beginning in 1898, financial tensions in 1903, a disclosed-letter betrayal in 1908, and an uneasy political alliance in 1914; trust varies by direction.
- A Merchant Guild, mining company, miners' association, parish and city government with dated memberships/offices, a schism and conflicting organizational stances.
- A mining disaster, strike, a secret sponsor, a conflict with 3+ positions, events/actions/consequences, and a disputed public rumor.
- A private fact unknown to Alice; sources including letter, witness account, decree and diary.
- A character Canvas entry with responsive Grid/Panel/Stack, portrait, image hotspot, rich text, timeline, QueryTable, citation and hidden private block; a published Display variant.
- A saved historical membership query, a link discovery proposal and alternate history branch before 1908.

All named objects need reproducible IDs, explicit dated facts and expected answers at 1898, 1903, 1908 and 1914. Test fixture itself belongs in source control once the application exists.

## Acceptance scenarios

| ID | Scenario | Required result |
|---|---|---|
| AT-01 | Create, rename, save and reopen world entities | IDs and all references survive |
| AT-02 | Biological and adoptive parents + rumor | Each relation category/query stays distinct |
| AT-03 | Reversed interpersonal trust | A→B differs from B→A |
| AT-04 | Tags evolve over dates | Older snapshot retains old tag; future reflects replacement |
| AT-05 | Betrayal event changes relationship | Inspector displays actor, event, before/after and supported explanation |
| AT-06 | Member, officer, expelled, secret sponsor | Membership history independent from event sponsorship |
| AT-07 | Organization schism | Both successor and predecessor histories query correctly |
| AT-08 | Multiple actor roles per event | Victim, financier, mediator etc. remain independently identified |
| AT-09 | Failed action and rumor | Rumor never becomes canonical successful action |
| AT-10 | Three-sided conflict and defection | Issue-specific sides differ over time |
| AT-11 | Event succession/causality | Graph follows confirmed explicit causes, not date adjacency |
| AT-12 | Actor contradicts group stance | Individual event affiliation differs without corrupting membership |
| AT-13 | Author vs character vs public knowledge | Secret only visible to authorized perspective |
| AT-14 | Fork branch before historical event | Child and parent diverge; parent later edits do not silently rewrite child |
| AT-15 | Author correction of fictional event | Historical event identity stays stable; prior author revision recoverable |
| AT-16 | Partial/uncertain date | Possibly true distinct from definitely true |
| AT-17 | Node-only entry gains Canvas template | No copied semantic object |
| AT-18 | Move a bound Canvas component | Same binding and ElementId after reparent |
| AT-19 | Query-bound membership table changes | Node, Canvas and Display agree after edit |
| AT-20 | Hide field/section/edge/image hotspot | Public HTML, search, snippets, backlinks, counts and graphs cannot expose it |
| AT-21 | Alias collision and mention discovery | Candidate remains unapproved; no invented guild membership |
| AT-22 | Saved query in all three studios | Same typed semantics; proper viewer filtering |
| AT-23 | Graph traversal budget | Clear truncation instead of UI freeze |
| AT-24 | Index incremental update/rebuild | Identical authorized results after reindex |
| AT-25 | Permission change after caching | No stale public data; cache invalidated |
| AT-26 | Cross-world IDs | FKs/domain layer reject incorrect reference |
| AT-27 | Import/export with assets | Semantic archive roundtrip, hashes validated |
| AT-28 | Deterministic synthetic DNA | Same seed/version returns same segment outputs |
| AT-29 | Distant relatives | Genealogical kinship can exist without detectable shared synthetic segments |
| AT-30 | Parentage change | Invalidates genetic derived results, not legal records |
| AT-31 | No AI, offline | Standard authoring/search/publication works |
| AT-32 | WebGPU unavailable/device lost | DOM/SVG/Canvas fallback works and remains accessible |
| AT-33 | Invalid multi-effect mutation | Atomic rollback with informative errors |
| AT-34 | Public nested query in hidden block | No execution or side-channel leak |
| AT-35 | Fulltext and graph query with false-name aliases | No silent incorrect ID assignment |
| AT-36 | Focused 50k world traversal | Bounded query/scene; documented hardware timings |

Earlier acceptance examples from v0.1–v0.4 are folded into these testable scenarios. Implement actual automated tests and record their commands/results; this file is only a specification.

## Spatial + cartographic acceptance extension (SP gates)

These unpassed feature #68 production gates consolidate NL-01–14 and VC-01–14 from the studies; no renumbering or retroactive change to prior AT results. Link [13](13-Spatial-and-Nested-Location-Model.md) and ADR-013.

| ID | Scenario and required result | Related AT / issue |
|---|---|---|
| SP-01 | Unlocated Place create/reopen/rename/type history; stable references, unknown not zero; wrong-world/kind rejected | AT-01/26, #4 |
| SP-02 | Multiple parent lenses, overlap, adjacency, event/placement roles; no mixed-lens or ownership/routing inference | AT-08/12/22, #4/#6/#7 |
| SP-03 | Co-effective strict cycle including staged batch/branch rejected atomically; disjoint intervals allowed; bounded failure safe | AT-14/23/33, #4 |
| SP-04 | Dated reparent/control, prior revision, inherited fork/withdrawal/nested sibling isolation, uncertainty/claim separation | AT-14–16, #4/#7/#8 |
| SP-05 | Moving vessel cabin and same XY different floors; uncalibrated frames never compared; documented extent/north/scale only when known | AT-01/22, #4/#5 |
| SP-06 | Scoped recursive query and alias ambiguity, pagination/cancel, saved binding/rebuild/intermediate dependency parity | AT-21–25/35, #4 |
| SP-07 | Exact geometry validity, holes/shared edge and candidate bbox false positive; topology-aware LOD if introduced | AT-23/24, #5/#10 |
| SP-08 | Native Node→Canvas→Display nested map navigation, pinned recipes, accessible list and same historical selection; schematic labeled and unplaced list correct | AT-17–20/22/32, #5 |
| SP-09 | Style/theme changes never mutate world; safe SVG export; deterministic label priorities/collision metrics, visible hit targets | AT-18/20/27, #5 |
| SP-10 | Deterministic draft/locks, unsatisfied/unknown report, no canonical writes until explicit stale-safe transactional promotion; no randomization from unrelated edits | AT-15/33, #5 |
| SP-11 | Public noninterference with secret root/path/anchor/geometry/label/count/extent/legend/hit/export/cache; redacted raster; policy change and reader error/cursor/budget leakage | AT-13/20/25/34, #8 across all |
| SP-12 | Institution residence/property and multi-site conflicts, migration and population origins retain role separation and historical effects; no culture/genetic inference | AT-02/06–12/28–30, #6/#7/#9 |
| SP-13 | Archive upgrade/export/restore preserves Place/fact/geometry/frame/recipe IDs, pins and hashes, missing assets diagnostics, semantic read equality | AT-26/27/33, #10 |
| SP-14 | Real native desktop, offline/no-GPU, 1k/10k/50k dense mixed feature fixture; timing, memory, geometry vertices, labels, pan/zoom/export and device fallback documented on target hardware; no arbitrary SLA from isolated benchmarks | AT-23/31/32/36, #10 |

SP-01–06 are foundation/query gates; SP-08/09 start with SVG/schematic, SP-07 applies to exact geometry when introduced, SP-10 only to candidate feature, SP-11 applies from first read, SP-12–14 cross-phase. No screenshot of a prototype or isolated SQLite experiment passes these production gates.

## Test layers

**Unit:** typed IDs, time/branch semantics, relationship inverse and symmetry, biological cycle rejection, role validation, AST normalization, permission intersections and layout child constraints.

**Integration:** SQLite transactions, FK checks, version overlay, correct author revisions, evidence references, publication compilation, link index and query invalidations; deterministic seed.

**UI end-to-end:** author Alice and Beatrice, create event/action/effect, slide historic timeline, inspect "why changed", move query-bound Canvas component, publish and search as public versus author.

**Security:** fuzz hidden hyperlinks, source excerpts, image metadata, backlinks, counts/facets, graph hop pathways, query results, cached pages and exports. Do not consider CSS-based hiding adequate.

**Resilience:** missing image, stale revision, DB locked, broken import, unsupported GPU, query cancellation and index rebuild while authoring.

**Performance:** seed 1k/10k/50k objects; record startup, render/zoom, traversal depth/time, FTS latency, cache invalidation, memory usage and export. Set numeric limits only after measuring the target Windows hardware.

## Definition of Done

No phase passes until it (a) references requirement IDs in [01](01-Requirements-and-Invariants.md), (b) includes working user-facing behavior or a demonstrably reusable engine capability, (c) has repeatable unit/integration and relevant UI/security tests, (d) validates temporal, revision, branch and visibility semantics, (e) records migration/backup impact, (f) updates linked ADRs, and (g) provides a short evidence log. "SQL smoke-test passed" does not prove the Rust/Tauri/wgpu application exists.
