# 13 — Historical Spatial Model & Procedural Vector Cartography

> **Feature scope accepted (2026-11-22 UTC):** [New Feature #68](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/68) received [explicit owner approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/68#issuecomment-7073427305). The [accepted feature specification](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/main/Design-Docs/Features/nested-locations-and-procedural-vector-cartography.md) governs SPA/MAP extensions and SP-01–14 as feature-specific requirements/gates; original roadmap phase exit criteria are unchanged. The earlier hold remains in Git history (8f39572). Draft #67 requires baseline and exact-head design/test review before further coding or merge; no implementation or production verification is implied by scope approval.


**Status:** Accepted architectural direction (owner-approved reconciliation, 2026-11-21 local / 2026-11-22 UTC). Implementation and detailed library/schema choices remain unverified.  
**Authority:** [D-017–D-020](12-Decisions-and-Reconciliation.md), [requirements](01-Requirements-and-Invariants.md), [ADR-013](ADRs/cross-phase-spatial-cartography.md).  
**Research:** [#65 NestedLocations](Research-Studies/nested-locations.md) and [#66 Procedural Vector Cartography](Research-Studies/procedural-vector-cartography.md). This contract supersedes conflicting *recommendations* in both studies, not their recorded observations. No new standalone database, studio or roadmap phase.

## 1. Responsibility and invariant boundaries

One canonical historical world: Rust/SQLite stores Place, spatial facts and optional geometry; query/policy resolves the historical snapshot; deterministic cartographic composition and proposal generation consume only permitted facts; Node, Canvas and Display share the resulting scoped view.

    Canonical Place + reified spatial facts + historical geometry
                 | ViewContext + QuerySpec + authorization
                 v
     authorized spatial snapshot ----> schematic / exact composition
                 |                              |
                 +--> constrained draft --------+
                           |                     v
                    explicit author review   MapScene
                           |                     |
                canonical transaction        SVG / Canvas2D / optional wgpu

Geometry, author proposal, rendering path and map pixel are different types. A map is never a second semantic store. Style/label placement never mutates a place. A newly inferred border or room never becomes a world fact merely because it was rendered. Imports and generator output are proposals unless a validated author mutation intentionally establishes canon. Use INV-01–04, INV-09–16 and SPA/MAP requirements in 01.

## 2. Canonical spatial model (logical types, not compiled API)

    Place { object_id, versioned place_type }
    PlaceEdge { fact_id, child, parent, kind, lens, valid_interval,
                branch, recorded_revision, provenance, audience }
    LocatedAt { fact_id, subject, place, role, valid_interval,
                branch, recorded_revision, provenance, audience }
    Frame { id, versions: units, origin, axes, extent?,
            parent_transform?, calibration?, provenance, audience }
    GeometryVersion { id, place, frame, role, geometry,
                      valid_interval, branch, revision, uncertainty,
                      provenance, audience }

Use the universal world-scoped ObjectId registry; give reified facts stable identity, evidence and history. Use existing branch/version machinery, not a second revision ledger. Place types may be authored metadata (planet, valley, city, estate, building, room, vessel interior) without a fixed hierarchy or mandatory coordinates. Unknown coordinates are absent, never (0,0). Rename, reclassification, boundary correction or movement never automatically creates a new identity; split/merge with successor records is explicit. Physical object and spatial identity differ: ship Asset may have a linked interior Place; a polity Organization may govern one or several territorial Places. Ownership uses existing Asset links and network facts, not a duplicate Place ownership field.

| Fact kind | Composition rule |
|---|---|
| ProperPartWithin, Place→Place | Strict irreflexive containment, transitive only within compatible spatial lens; no co-effective cycle; multiple parents when meaningful |
| AdminPartOf, Place→Place | Transitive within same declared administrative hierarchy, never across unrelated lenses; no co-effective cycle |
| Overlaps, AdjacentTo | Symmetric; neither is transitive nor implies authority |
| ConnectedTo, AccessibleVia | Directional; cycles allowed, no automatic access right or distance |
| LocatedAt | Typed subject and role: residence, birth, death, presence, event site, headquarters, asset placement, population origin; role-specific temporal meaning |
| Jurisdiction, Control, Claim | Distinct facts institution/faction→Place; legal authority, actual control and asserted claim are independent |
| Ownership / successor | Existing asset ownership and explicit event-driven continuity respectively; no implicit containment |

For semantic physical containment no coordinate frame is required. Cross-frame **geometric** calculations require compatible frames or a valid explicit transform. A cabin remains nested within its ship interior while the Asset moves. Broader geographic presence of a cabin or occupant is inferred only through supported co-effective placement and containment, with permission along every step. No inference of ownership, administration, access or citizenship from physical inclusion. Primary UI parent is a presentation choice, not a canonical single parent. A tree view can repeat a link to the same ID; deduplicate query results.

## 3. History, validation and uncertainty

Exact valid intervals use existing half-open WorldDay semantics. The branch/revision resolver must freeze inherited parent history at the fork as in 03 and accepted persistence rules. Do not copy a fork by current parent state or let a branch-local withdrawal affect siblings/main. Author correction creates recorded revision, not an in-world move. Dated reparenting may be an explicit event effect, atomic with all associated changes.

For strict containment check proposed child→parent over its entire effective interval, in relevant branch at expected revision including staged transaction facts. Traverse parent toward child in same compatible relation/lens; intersect each edge with running interval. Reachability with nonempty interval is a canonical cycle. Disjoint historical reverse edges are permitted. Self-edge is invalid. Maintain bounded visited node/interval coverage and fail closed on budget exhaustion. Check author-authorized canonical mutation graph, not reader-filtered graph. No universal acyclic requirement on adjacency or travel. Reject wrong-world, wrong-kind, stale revision, invalid interval, conflicting exclusivity and invalid frame/geometry without partial write.

First slice accepts exact intervals with optional bounds where supported by existing store. Uncertain place assertions remain sourced claims/proposals unless typed uncertain spatial facts are separately approved and implemented. Queries must preserve definitely/possibly/not-established semantics; unsupported correlation never promoted. Contradictory accounts, disputed claims and canon are separate. Never infer a precise historical date or location merely to display it.

## 4. Geometry and frames

Start with geometry-optional semantic hierarchy and local planar coordinates; then exact point, line, polygon/multipolygon (holes), explicit geometry role (representative point, footprint, boundary, route, uncertainty extent). Independent frame per world/region/map/interior/floor when necessary; a local frame may have a calibrated transform, but a floor plan does not need planetary coordinates. Validate units, axes, finite values, vertex/byte caps, geometry validity, floor/elevation and transform compatibility before semantic writes. Fictional geodetic conventions require explicit planet model, projection and wrap policy; no automatic Earth/WGS84 assumption. Preserve source geometry at full supported precision; simplify derived presentation only.

Raster maps/hotspots are independent versioned assets and navigable references, not authoritative coordinate data. A map can link to another frame even where no transform exists. Calibrated extent, scale and north are optional; schematic layout never claims measurement. R*Tree bounding box candidates may be added only with verified shipped SQLite availability and exact predicate checks in the relevant frame. Derived index never constitutes proof of geometric containment. Shared-boundary topology/LOD is a derived optimization, with stable geometry references, coordinated simplification and topology tests. No geometry library, tessellator, GIS engine or tiling dependency accepted by this decision.

## 5. Shared query contract

Extend the existing QuerySpec/versioned bindings, not a parallel spatial query language:

    WithinPlace(root, lens, include_self, max_depth)
    LocatedIn(root, location_role, lens, include_descendants)
    SpatialRelationTo(place, predicate, geometry_role)
    PlaceName(text, optional_context_place)
    MapFeatures(root, frame, viewport, layer_binding)  // scoped projection

Names/aliases/ghost proposals use the current discovery mechanism; textual mention does not create canon. Resolve all queries using world, branch, time, author revision and scope; never assume author when context missing. Physical/admin lenses compose separately. For interval queries carry intersection of path validity, not snapshot union. Return authorized stable refs, sources where allowed, certainty, stable pagination and explicit incomplete/truncated status; bound depth, expansions, vertices, bytes, deadlines and cancellation. Invalidate dependent views on changes to paths, prior/new parents, geometry, placement, names, policies, query, map recipe or frame. Use indexed adjacency initially; optional derived closure/caches only after profiling. No claim of production efficiency from synthetic fixtures.

## 6. Map recipes, scene composition and generation

    MapSpec { root_place, lens, frame?, mode, layers, theme_pin,
              viewport, generator_pin?, seed?, locks?, overrides? }
    LayerSpec { id, query_binding, feature_kind, role, style, zoom_range, priority }
    MapScene { safe_snapshot_token, bounds, primitives, labels, legends,
               authorized_hit_targets, permitted_omissions, truncation }
    RenderFeature { stable_id, authorized_object_ref?, geometry_ref?,
                    visual_role, origin_status, style_token, path }

A recipe is a Canvas/presentation object with stable identity and version pins. MapScene is an immutable scoped DTO, not database authority. Use typed safe geometry and style props, not arbitrary markup or scripts. Feature IDs must resolve to authorized objects or explicit proposal/presentation IDs, never silently assign world identities. Styling never mutates source. Three feature-level origin modes:
- Authored: faithfully project existing geometry; unplaced locations remain available in an authorized list.
- Schematic: deterministic hierarchy/layout without physical claims; visibly not to scale; no fabricated distance, shape, adjacency or boundaries.
- Draft: proposed geometry from permitted constraints, marked noncanonical and visibly distinct, directed to author review. No publication as canon. If an author releases an explicitly noncanonical illustration, publication must preserve its status and permission checks; not an initial slice requirement.

Stage exact local geometry and schematic maps independently of procedural geometry. For proposals use sorted stable input IDs, pinned algorithm/config, deterministic per-ID streams and explicit seed, not global revision or hidden records for randomness. Accepted geometry and hard locks are fixed constraints. Previous candidate preferences and generator defaults cannot override them. Validate all hard constraints; return actionable unsatisfied/unknown report, no fabricated compliance. Author promotion requires explicit preview, existing reference validation, stale dependency/expected revision check, atomic provenance-backed command and re-query. Style-only edits do not promote geometry. Candidate constraints store provenance and only releasable input IDs.

Algorithm sequence: schematic tree/graph layouts; bounded planar candidate subdivision (Voronoi as baseline, not proof of territory); constrained subdivision/weighted alternatives only when supported; street graphs, parcels and floor plans later with genuine input/rules and separate validation. No automatic roads/access or settlement status from polygon layout. No tectonics, full CAD/GIS, procedural world simulator, mandatory tiles or external services in the initial contract.

## 7. Cartographic visual system and interaction

Use versioned semantic layer/style/symbol vocabulary (land, water, terrain, boundaries, routes, buildings, settlements, event areas, labels). Reusable local vector symbols have paths, anchors, viewboxes, style tokens and license/provenance. Map themes change fill, stroke, hatching, labels, symbols and visibility subject to query permissions; distinguish disputed and draft status beyond color. No unsupported terrain invented as fact. Scope legend, extent, label collision/priority, classification and aggregate values to released data. Use deterministic priority/ID tie-breaking and measured text bounds; suppress labels under budget while retaining accessible navigation. Label offsets live in recipe.

Node: author Place and dated links, choose lens, inspect evidence, navigate and select stable nodes. Canvas: MapView/ConflictMap with versioned bindings, map recipe, historical selector, optional underlay, nested navigation/links, style and later proposal editor. Display: read-only authorized interactive map and keyboard/screen-reader-friendly list, same historical context, no raw author data. Drill down through semantic links world→region→city→estate→floor even when frames are independent. Render only compatible layers, no fake continuous zoom across uncalibrated frames.

SVG first for bounded exact/schematic maps and vector export, typed allowlist/escaping with no script, foreignObject, event attributes, external resources or raw author XML. Canvas2D for larger/no-GPU view once measured. Existing wgpu renderer remains optional acceleration after tessellation and target hardware evidence; common scene DTO, parity and picking tests, no extra world engine. Do not set universal thresholds from browser microbenchmarks. Optimize viewport, LOD, culling, batches, dirty state and labels before adopting tiles or new GPU backend.

## 8. Security and portability

Authorization precedes traversing paths, candidate layout/seed input, geometry union, culling, label placement, classification, legends, statistics, hit targets, serialization and caching. Unauthorized/nonexistent targets must be indistinguishable where applicable; hidden intermediate cannot establish visible containment absent a separate approved broader fact. Canonical validation sees full authorized author graph; reader derivation sees filtered graph. If secret geometry is embedded in an otherwise public image/polygon, release approved redacted derivative or withhold it. Never rely on CSS hiding, overlay toggle or removing labels. Test public noninterference with secret presence/absence across draft boundaries, labels, extents, counts, metadata, links, exported SVG and archive/published tile outputs. No raw global revision/budget diagnostics exposing secret edits to readers. Cache internal key binds full view/policy, recipe, frame, generator version, viewport and authorized dependencies; released token is scoped. Timing/budget/error leak tests required; prototype is not proof.

Portable archive preserves canonical IDs, spatial facts/frames/provenance/history, recipes and pins, approved constraints/overrides, local assets and hashes, plus versioned proposal metadata as applicable. Mesh, adjacency/geometry indexes, labels and tiles rebuildable. Restore validates schema, cross-world references, missing frames/assets and semantically equivalent results. Public export distinct from full author backup. Offline operation and no-GPU fallback mandatory.

## 9. Dependency-ordered delivery

Existing #1–#3 remain closed. Feature #68 owns extension implementation and acceptance; #4–#10 are integration dependencies, not new original phase exit requirements.

| Slice | Responsible issue | Gate before downstream |
|---|---|---|
| A: spatial identity, exact containment, LocatedAt, branch/revision, no geometry required | #4 with foundation integration from #1–#3 | Stable Place create/reopen, dated moves, cycle rejection, scoped recursive query, branch isolation |
| B: native exact local geometry + schematic MapView, accessible links/list, shared scene | #5 (depends A, media/branch consolidation) | Authored shapes/holes, unknown location, region→town→building, history, author/public parity, export safety |
| C: residences/assets/institutions and conflict/history overlays | #6, #7 | No duplicated ownership, typed event sites vs impacted areas/control, real historical corrections |
| D: stronger privacy, uncertainty and branch integration | #8, starts A | Adversarial hidden path/geometry and map generation noninterference; no privilege widening across surfaces |
| E: bounded candidate-generation prototype and product slice | #5 with #4/#8; advanced generation only after A–D relevant gates | Stable seed/constraints, no unauthorized influence, review/acceptance/rollback; deferred complex algorithms |
| F: population origins/migration overlays | #9 | Geography separate from cultural/genetic inference |
| G: scale/portability/renderer qualification | #10 | Dense 1k/10k/50k mixed fixture, native target measurements, offline/fallback/recovery |

Before code: consolidate/recheck #62/#63 exact current heads, choose migration against actual latest schema (no prescribed number), inspect existing typed fact/branch constraints, approve concrete schema/DTO version and license/geometry spike. Tests first; first native vertical slice no mandatory maps or procedural generation. Implement in existing Rust/store/query and Tauri front end, splitting modules only as needed. No production implementation approved by *research ticket closure* alone; this contract is the design basis for subsequent explicitly scoped development.

Verification: [09](09-Acceptance-and-Verification.md) lists SP-01–SP-14, the reconciled nonduplicated gates derived from both studies. Neither study's isolated test evidence counts as a production gate. Research #66 bounded follow-up unknown/constraint/label experiments are documented in its study section 11. Real production unknown persistence, full applicable constraint validation, security, native/renderer and archive gates remain open; isolated experiments do not pass SP gates.
