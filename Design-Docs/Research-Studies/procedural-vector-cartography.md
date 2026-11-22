# Procedural Vector Cartography from NestedLocations Data

**Status:** In Review — architectural direction accepted in ADR-013; original experimental gaps and production verification remain open.  
**Author:** Lola, research/codesign agent. **Date:** November 21 local / November 22 UTC, 2026.  
**Origin:** [Research #66](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/66), companion to [#65](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/65) and [NestedLocations](nested-locations.md).  
**Audited main:** [ea93ada](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/ea93ada3032478edb05fc25a0cbe408a3db61cc9). No production code or contract changes in this research.

**Review addendum (2026-11-22 UTC):** [ADR-013](../ADRs/cross-phase-spatial-cartography.md) and [13](../13-Spatial-and-Nested-Location-Model.md) are the accepted joint direction with #65. Original measured observations and research limitations remain unchanged. Full constraint/unknown, label metrics, native/real branch/security gates remain unpassed.

## 1. Question and method

Investigate deriving attractive editable vector maps from the canonical historical spatial model, including incomplete geometry. Reviewed current repository inventory (211 paths, 66 issue/PR records), root README, research/feature/screenshot policies, design architecture/query/Canvas/ADRs, current graph renderer and browser benchmark, open #62/#63, #65 and exact #66 ticket. Reviewed primary reference documentation [R1–R11]. Built an isolated synthetic Northbridge browser fixture and executed geometry assertions and rendering microbenchmarks using Chromium and Shapely. Compared SVG DOM and Canvas2D on identical polygon feature input. Visually inspected real browser capture. No production Rust compile, native Tauri run, GPU test or actual world DB access was performed. The scope is not replacing GIS or generating entire fictional worlds from nothing.

## 2. Existing architecture

[D-001, D-004, D-009–012, D-016](../12-Decisions-and-Reconciliation.md) establish Rust/SQLite authority, one semantic world, scoped publication, local-first operation and shared query. [Architecture](../02-System-Architecture.md) allows WebGPU via wasm with accessible fallback; [Canvas](../06-Three-Layer-Entries-and-Canvas.md) plans MapView and ConflictMap, typed bindings and component fallbacks. [Query](../07-Search-Query-and-Discovery.md) requires frozen view context and budget. [atlas-renderer](../../crates/atlas-renderer/src/lib.rs) has GraphRenderer with position/color vertex attributes and surface, not a vector path tessellator; [graph-benchmark.ts](../../apps/desktop/src/graph-benchmark.ts) implements the existing graph presentation route. #65 proposes a Place identity, typed temporal containment and optional GeometryVersion; implementation is pending. #62 branch metadata and #63 media/hotspots remain unmerged at baseline. Do not add speculative tables/migration numbers or a parallel map database.

Candidate architecture depends on an accepted #65 Place contract; isolated experiments do not. A map is a derived artifact, not the place or its property owner. A change in style must never change terrain, territory, ownership, names or event facts. A geographic boundary edit is a semantic author command, not a renderer side effect.

## 3. Literature and comparable systems

| Reference | Observed design | Implication / limitation |
|---|---|---|
| SVG 2 [R1] | XML/DOM vector syntax, paths, fills and transforms | Good portable document/export and small accessible UI; use safe subset, no scripting in exported map |
| D3 geoPath [R2] | Geographic paths rendered to SVG path strings or canvas context | One projection/path pipeline can target multiple outputs; don't equate image pixels with lon/lat |
| MapLibre layers [R3] | Data-driven sources/layers, layout, paint, filters and zoom ranges | Separate map style and feature data; full renderer not required; evaluate local non-Earth frames/offline assets before adopting |
| Mapshaper [R4] | Simplification and topology preservation | Shared edges require coordinated simplification; independently simplified polygons can crack or overlap |
| Lyon [R5] | Fill/stroke path tessellation for GPU buffers | Candidate for future wgpu view; text shaping, labels, picking, transforms and rendering still need design |
| TopoJSON [R6] | Shared indexed arcs with reversible orientation | Useful derived topology, stable IDs and coordinated LOD; quantization is a choice, not semantic precision |
| geo Rust [R7] | Planar polygon operations, simplification, topology, spatial indexing | Candidate for Rust-side validation/boolean operations; geodetic and local semantics must be explicit |
| Azgaar [R8] | Seeded maps, multiple layered styles, SVG/JSON/GeoJSON exports; README advocates separate state/generator/editor/renderer | Strong reference for atlas styling and tool ergonomics. Seed alone cannot recover manual edits. Its MIT license is visible; retain notices if reused. |
| Watabou [R9] | City generator evolved from Voronoi diagrams with customization and SVG export; older public code is GPL-3.0 | Good conceptual reference for settlement generation; do not copy GPL implementation into project without license review. It prioritizes pleasing maps, not historical accuracy. |
| Parish and Müller [R10] | Procedural streets informed by environment/population inputs, lots and buildings | Streets/blocks/parcels require rules beyond Voronoi; research separate bounded algorithms. |
| GeoJSON [R11] | Earth WGS84 interchange | Never label fictional coordinate frames as WGS84 by default. |

No external code was copied or dependencies added. SVG and native Canvas2D require no additional library. Candidate dependencies D3, MapLibre, Lyon, geo and optional tooling require exact version pinning, SPDX/transitive review, wasm/native platform build and offline asset audit. No library selection or license compatibility acceptance in this pass.

## 4. Data-completeness modes and generation

A map can combine feature-level origin statuses and an explicit map legend.

- **Authored:** project stored points, lines and polygons with holes into frame. Preserve geometry identity, source and time. Never smooth authoritative coordinates in place. A place without coordinate has an unplaced list record, not a default zero point.
- **Draft:** deterministic, constrained candidate geometry from known extent/anchor/area/adjacency or terrain constraints. Show dashed/hatch style and provenance, author review, unresolved constraints, and dependency revision. No canonical fact before explicit transactional promotion.
- **Schematic:** graph/nesting view in a separate layout coordinate space where location geometry is absent. State not to scale; do not claim travel distance, adjacency or border shape. Generated positions remain presentation only.

Compare algorithms:

| Technique | Appropriate use | Failure condition |
|---|---|---|
| Bounded Voronoi via halfplane clipping | Seed anchored proposal within convex extent, quick candidate regions | Equal Euclidean nearest sites not historical jurisdiction; no fixed area or adjacency guarantee; naive all-pairs unscalable |
| Weighted power diagram | Candidate allocation biased by sizes/weights | Empty cells, no exact area unless iterative optimization; explicit infeasibility |
| Constrained triangulation/planar subdivision | Coastlines, rivers, locked border segments, non-convex bounds and shared faces | Robust numeric predicates and topology validation required; avoid simplistic hand-written production solver |
| Graph embedding/treemap | Semantic-only nesting, overlapping lens-specific views | Not geography; layout edges cannot be used for actual routes or territorial authority |
| Road network + street blocks + lot subdivision | Town plans with intersections, terrain/path constraints, known landmarks | Voronoi alone does not satisfy urban history, traffic, access or parcel rights |
| Rectilinear partition + adjacency graph | Floorplans with known extent, room area, entrances | Rooms fitting visually may violate circulation/doors/structure; never imply access automatically |
| Polygon union/clipping | Reconstruct region from authorized component geometry | Can't expose hidden shape indirectly; union only within compatible frame and explicit lens |

Preferred progression: exact + schematic first, simple candidate planar subdivision second, constrained street/parcel/floorplan generation only after data contracts. For seeded regeneration sort inputs by stable ID, use counter/ID-derived deterministic streams and pin algorithm/config version; do not key on global revision (unrelated edits would scramble maps). Lock precedence: author-approved geometry > explicit constraints > previous candidate preference > generator defaults. Fail with an unsatisfied-constraint report, not fabricated compliance. Keep accepted geometry independent of seed. Candidate IDs stable within recipe and input identity; accepted new Place ID assigned through command.

## 5. Typed proposed pipeline and visual design

    MapSpec { root_place, frame, lens, mode, layers, theme_pin,
              viewport, generator_pin?, seed?, locks?, overrides? }
    LayerSpec { id, query_binding, permitted_feature_kind, visual_role,
                style_rule, zoom_range, priority }
    MapScene { snapshot_signature, render_bounds, primitives, labels,
               legends, visible_hit_targets, omissions, truncation }
    Feature { stable_render_id, authorized_object_ref?, geometry_ref?,
              kind, visual_role, origin_status, style_token, local_path }

Rust resolves and authorizes scoped world records, then generates derived scene. Viewport bounds are parameters, not privilege. Projection and styling may be pure deterministic presentation operations on already-authorized data. Use local frame origin, units and affine transforms only when known; no automatic globe-to-floor conversion. Clip projection at frame/canvas boundary, preserve holes and multipolygons. Earth-compatible geo exports handle antimeridian rather than connect wrapping segments across map; fictional planet projection choice requires defined model. Unknown transform -> independent map navigable by semantic link.

    build_scene(view, spec, budget):
        scoped = query_authorized(view, spec.root_place, spec.layers)
        validate_frames_and_input_caps(scoped)
        draft = generate_only_from_authorized_constraints(scoped, locks, seed)
        compose_layer_geometry(scoped, draft, mode)
        simplify_shared_arcs_once_per_lod()
        place_symbols_then_labels_by_stable_priority()
        return safe_immutable_scene_without_hidden_metadata()

Theme vocabulary: semantic layer roles (water, land, terrain, boundaries, settlements, routes, buildings, labels, event areas) -> typed fill/stroke/width/dash/pattern/icon/typography rules. Symbol library as local versioned vector paths with anchor, viewbox and permitted tint, not arbitrary SVG code. Example symbols: castle/city/hamlet/port, mountains/hills, woods, bridges, gates, walls, stairs; use scale-appropriate generalization and repeated terrain symbols only for known terrain or decorative mode. Colors not the only cue; hatching for disputed territory, dashed proposal outlines, contrasting labels and accessible list. Keep legends based on scoped layers, and scale/north marked only if calibrated. Same scene semantics support physical, political, cultural, institutional, historical conflict, population and transport styles. Layer query for same viewport can change data/time without style duplication.

Label layout: deterministic candidate positions, priority and tie-breaking by stable ID, collision boxes using measured glyph bounds, spatial grid/R-tree, suppress lower priority with accessible list fallback. Track actual overlap and placement rate. Prototype uses naive estimated rectangles; no production claims of typography quality. Store label offsets/overrides in map spec, not canonical geometry. Use local packaged fonts and symbols (license review); SVG portable export should use safe text or optional outlined paths.

Map UI in Node: location explorer + selection opens canonical node; Canvas: map component binds query/style/version; Display: scoped read-only navigable maps + list and historical selector. Stable feature IDs resolve via authorized objects, not map text. A linked raster map from #63 may be underlay; cannot unredact hidden features by toggling layers. Map tile exports are audience-specific release artifacts.

## 6. Historical change, privacy, persistence

Use existing ViewContext including world, branch, revision, date, scope. Keep location boundary, claimed border, legal jurisdiction, actual control and ownership distinct. Switch time generates new snapshot; maintain stable geometry/feature identity where unchanged. Do not interpolate between historical geometries unless labeled as illustrative and approved. Separate event occurrence location from affected territory. Ship interior in local frame can remain identical when its placement changes. Use source-backed facts, don't map political ownership from proximity.

Filter before any generator, topology union, simplification, label collisions, extents, legend classes, statistics, cache and export. A secret anchor must not move public boundaries or labels. Hidden areas in raster or unified polygon geometry require redaction at semantic source/approved derivative; dropping only labels is insufficient. Draft generation sees only authorized constraints, separate author/public seeds/config as necessary, avoid global RNG sequence keyed to hidden input. Cache key includes audience/policy/perspective, world/branch/date/revision, query, frame, style and generation version, view bounds and budgets. For private changes across revisions, public equality should hold for released visible facts; don't expose global revision deltas or budget leakage to readers. Error, cursor and timing side channels require tests. Never send untrusted HTML/JS/XML or external links through SVG serializer; typed allowlist + escaped text & attributes, finite coordinate/complexity caps, no foreignObject, script, external href or event handlers; export stable authorized IDs only when explicitly allowed. No IO or network from renderer.

Persist canonical geometry/provenance/branch per #65. Persist author map recipes, style pins, locks, overrides, generator version and accepted semantic IDs. Candidate proposals separately versioned; derived mesh, topology and label caches are rebuildable. Archive includes map recipes, required local assets and version pins with current archive integrity rules. Exact semantic equality, missing frame/asset and incompatible version diagnostics; raw SVG is export, not the authoritative world file.

## 7. Executed research prototype and observations

Files: [prototype.html](cartography-prototype/prototype.html) and [check.py](cartography-prototype/check.py). Open the HTML in a regular Chromium browser or use the test runner. No network/assets required. The runner uses Python Playwright (system Chromium executable /usr/bin/chromium; change path for another machine), Shapely 2.1.2; run from its directory using Python with those packages installed. The browser test used set_content because this environment blocks file URLs. No installed application or DB involved.

Synthetic fixture: local 640x360 frame with land polygon, two political regions sharing a border, hole, river, road and settlements. Map selector includes regional, candidate, town, manor, ship, schematic; day 99/100 and branch main/fork, author toggle. Region click on Northbridge -> town; manor point -> floorplan; ship placement label changes across dates; schematic marked not to scale. Town and manor use simple synthetic rectangular arrangements, NOT real procedural street or constrained floorplan generation. Geometry array is hard-coded fixture input, not an actual NestedLocations database. Candidate = four seed anchors (three fixed, one pseudo-random) in a rectangle clipped to all bisector halfplanes; seeds 66 and 1 demonstrate fixed anchor. Draft cells never enter any canonical store (there is no store). Authored fixture coverage isn't complete: land base remains visible in gap/hole. Not a full partition constraint on authored regions.

Executed assertions passed after fixing initial environment file-URL access and using SVG textContent instead of innerText:
- identical deterministic draft scene for same seed in same runtime, locked town coordinates invariant for tested alternate seed.
- public SVG byte equality with private point present/absent in fixture; author differs. This is a narrow pre-label filter check, not production security: browser contains all synthetic fixture data and the author checkbox is not real authorization.
- day 100 changes eastern region color with unchanged boundary; fork matches earlier map; these are hardcoded historical fixture logic, not real branch resolver results.
- Shapely confirms both region polygons valid, shared boundary length positive, overlap area within tolerance, point in hole excluded. Candidate polygons valid, non-overlapping area sum vs union and union equals 570×290 bounds within 1e-6 fixture tolerance.
- Playwright clicks both drill-down levels, verifies ship label changes, and confirms schematic warning. UI lacks keyboard feature navigation, true coordinate transforms, manual editing, pan/zoom, separate semantic unknown record, and author approval persistence.
- Collision handling exists in code, but no robust collision metric/assertion was run (label rectangles estimated; one probe computed). No full exact SVG/Canvas raster equality asserted; label outlines differ.

Captured actual running browser, Nov 22 02:58:48 UTC, regional view main/day100, public: [archived screenshot](../../Development-Screenshots/2026-11-22_02-58-48Z_phase-04_cartography_research-66-browser-prototype_01.png). Archived derivative resized to 400×265 and quantized to eight colors (no content fabricated), Git blob SHA 9367fff5fb1082e2b0efeac9911cdb2db15126d9, SHA256 fff2b5d3e7a34b35c94f69a9fb836631754ff9847b8e2dfa43653e34c8e96555. Original 800×530 capture SHA256 09313d74d430e454c4d12b18a5049e1d44dd76da63bd9c001193592223ad36c4; candidate original SHA256 5ee5ab33c0b7fc06e207aae83e68de60e764fac8b9c206fd4f88fea4d86de1a2, both regenerate using check.py (browser/font differences can change image bytes). Only regional derivative committed, candidate screenshot delivered in research conversation. Exact tested source blob IDs: html 63f69e5cff560925866030094512cb8b0b9a8a04, check.py b6ae0bd80337a739d4853ccad06fd576d91ee053. Not native Tauri/GPU screenshot or proof of integration.

## 8. Measured renderer feasibility

Headless Chromium 144.0.7559.96, Linux 6.18.57 x86_64 glibc2.41, Shapely 2.1.2. Identical array of simple four-vertex rectangle polygons (no labels; dense grid; 640×360 canvas). Five runs per size, median independently per metric. SVG serialization measured separately from setting innerHTML into visibility:hidden attached DOM + getBoundingClientRect; Canvas2D draws to detached canvas + getImageData(0,0,1,1) readback. These are different operations, not equivalent visual frame or pan/zoom timing; SVG paint not measured. Results include JS overhead, do not isolate hardware or prove optimal backend. No memory tracking beyond raw output size, no GPU/tessellation run, no production IPC or data generation timing; no raster-link baseline timing. No target Windows hardware SLA.

| Four-vertex polygons | SVG serialization ms | SVG DOM + layout ms | Canvas2D draw/read ms | SVG raw bytes |
|---:|---:|---:|---:|---:|
| 1,000 | 2.1 | 6.2 | 2.4 | 153,103 |
| 10,000 | 18.8 | 64.6 | 22.1 | 1,573,683 |
| 50,000 | 85.8 | 352.4 | 106.5 | 8,071,983 |

Interpretation: SVG is approachable for small maps and export, but DOM growth matters. Canvas2D avoids DOM per feature but still exceeds a single 16.7 ms frame at high feature count in this microbenchmark. Do not infer universal switching threshold. Viewport culling, clipping, zoom-dependent generalization, cached paths, batching, dirty updates and label budgets should precede automatic renderer selection. A later target test compares actual visible pan/zoom, resize, incremental generation, label density, vertex complexity, memory peak, time-to-first-frame and device fallback (1k/10k/50k mixed places/roads/polygons, holes, vertices, labels). Use stable accessible DOM alternative, and only add optional vector tiles or Lyon/wgpu when measured. Vector tiles are derived frame/zoom/audience-scoped chunks, not database truth. The GPU benchmark is still open.

## 9. Proposed integration and verification

| Phase | Scope and suggested validation |
|---|---|
| #2/#3 closed foundation | #65 Place, geometry version, frame, temporal identity edges; preserve phase disposition. |
| #4 query | Map feature binding, scoped viewport/time/branch query, bounded expansion, explicit incomplete geometry, saved query replay. |
| #5 primary | MapSpec/pin, exact/schematic SVG first, canvas fallback, link navigation and layout templates; local style/symbol palette, print/export. Prototype a bounded native vertical slice before advanced generator. |
| #6/#7 | Residences, asset parcels, historical jurisdiction/control, routes/events and migration thematic layers; no inference from style. |
| #8 | Strong equivalence tests for private presence/absence, hidden constraints, geometry leaks, labels, metadata, hit-testing, raster, cache and archives; real branches. |
| #9 | Founder/migration/population layers, avoid genetic ancestry -> cultural inference. |
| #10 | Portable recipes/assets, offline/no-GPU, target hardware, release and optional NPC link. |

Proposed VC gates, not existing official IDs: VC-01 exact shape/hole/shared edge and no unknown default; VC-02 deterministic seed+version+lock and unsatisfied constraints; VC-03 same world snapshot across styles, no world writes; VC-04 semantic-only warning plus unplaced list; VC-05 temporal historical event/control and branch isolation; VC-06 stable feature IDs, nested navigation and local frames/unknown transform; VC-07 topology-aware LOD preserving shared boundaries; VC-08 measured label collision/suppression, keyboard and screen reader; VC-09 secret input noninterference before generator/label/layout/export; VC-10 safe SVG allowlist/escaping; VC-11 archive roundtrip and recipe pins; VC-12 actual target renderer benchmark and device fallback; VC-13 candidate acceptance atomic audit and stale revision rejection; VC-14 native interactive map and cross-studio scope parity. Map these to INV-01–04, 09–10, 12–16, ENT-01, CAN-01/02, CON-02, QRY-01–03, IMP-01 and AT-17–25/27/32/36 in formal review. Isolated tests only partially support VC-01/02/04/05/06/09; no production gate is passed.

## 10. Recommendation and open decisions

Approve #65 spatial semantics first; propose cartography as separate specialist subsection in spatial contract with renderer/MapView details shared with 02/06/07/08/09 and ADR. No new roadmap phase. Stage 1 = authored and schematic map query, SVG renderer, safe link and accessible map list: prove nested region→town→building and date changes in actual application. Stage 2 = local versioned style/symbol system, Canvas2D fallback, native capture and export. Stage 3 = candidate generation with author locks and explicit acceptance; start bounded planar constraints, add real settlement/floor generation only with specific data and acceptance criteria. Stage 4 = topology/LOD/viewport, wgpu only with benchmark; local vector tiles only if needed. Scope geometric edit authoring deliberately; no general CAD/GIS or automatic terrain simulation.

Research deliverable ready for review; unresolved: geometry library/version/license, real branch-aware spatial schema, map frame and projection contract, exact primitive/picking/label shape API, candidate conflict detection and locks, publication-safe geometry derivation, full candidate-generator prototype for streets/rooms, production native GPU/perf and archive tests. Leave #66 open through architecture review. No normative specs changed and no claims of completed roadmap phase.

## References

Retrieved November 21–22, 2026 unless date specified.
- [R1] W3C, [SVG 2 Candidate Recommendation Snapshot, 6 October 2026](https://www.w3.org/TR/2026/CR-SVG2-20261006/); candidate spec, not final Recommendation.
- [R2] Observable, [D3 geoPath](https://d3js.org/d3-geo/path), geographic SVG/Canvas paths.
- [R3] MapLibre, [Style layers](https://maplibre.org/maplibre-style-spec/layers/) and [sources](https://maplibre.org/maplibre-style-spec/sources/).
- [R4] Mapshaper, [Simplification](https://mapshaper.org/docs/guides/simplification.html), [Topology](https://mapshaper.org/docs/guides/topology.html).
- [R5] Lyon, [lyon 1.0.19 documentation](https://docs.rs/lyon/1.0.19/lyon/), path tessellation.
- [R6] Bostock and Metcalf, [TopoJSON specification](https://github.com/topojson/topojson-specification), topology/arcs.
- [R7] Rust Geo, [geo 0.33.1](https://docs.rs/geo/0.33.1/geo/), planar geometry algorithms.
- [R8] Azgaar/Max Haniyeu, [Fantasy Map Generator repository](https://github.com/Azgaar/Fantasy-Map-Generator), [live application](https://azgaar.github.io/Fantasy-Map-Generator/), [quickstart](https://github.com/Azgaar/Fantasy-Map-Generator/blob/master/docs/wiki/Quick-Start-Tutorial.md), [MIT license](https://github.com/Azgaar/Fantasy-Map-Generator/blob/master/LICENSE).
- [R9] Watabou, [Procgen Arcana city](https://watabou.github.io/arcana/city.html), [TownGeneratorOS source (GPL-3.0)](https://github.com/watabou/TownGeneratorOS).
- [R10] Parish, Y. I. H. and Müller, P. (2001). Procedural modeling of cities. SIGGRAPH, pp. 301–308. [DOI 10.1145/383259.383292](https://doi.org/10.1145/383259.383292). Abstract reviewed, not a full paper reconstruction.
- [R11] Butler et al. (2016), [RFC 7946 GeoJSON](https://www.rfc-editor.org/info/rfc7946/).
