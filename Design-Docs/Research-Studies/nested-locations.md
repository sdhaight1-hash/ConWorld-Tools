# NestedLocations — Hierarchical, Spatial and Historical Place Modeling

**Status:** Reviewed — investigation completed; architectural direction accepted in ADR-013; not implemented.  
**Research issue:** [#65](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/65)  
**Author:** Lola, research/codesign agent. **Date:** 2026-11-21 local / 2026-11-22 UTC.  
**Inspected baseline:** [main d901dc7](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/d901dc7ec2b7b7621ab8ba8e65eb430311318f33), schema 33. Related open stacked PRs [#62](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/62), [#63](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/63) were not merged as part of this study.

**Review addendum (2026-11-22 UTC):** [ADR-013](../ADRs/cross-phase-spatial-cartography.md) and [13](../13-Spatial-and-Nested-Location-Model.md) reconcile this study with #66. Their normative decisions supersede proposal-only recommendations below; original observations and limitations are preserved. Exact implementation and advanced choices remain gated.

## 1. Research question and method

Determine the smallest local-first representation of historically changing, arbitrarily nested places that supports overlapping spatial and administrative structures, movable interiors, uncertain locations, query/discovery and the three studios. Compare tree, graph and geometric models against the existing contracts; inspect relevant Rust/SQL, primary gazetteer and geometry standards, and run an isolated SQLite feasibility experiment. No new application code, UI run or target-hardware benchmark was performed. No screenshots are applicable to this nonvisual investigation.

The root README, design index, accepted decisions, requirements, roadmap, specialist contracts, feature/study indexes, screenshot index and main inventory (210 tree entries), 64 existing issue/PR records and active development PRs were checked before creating #65. No dedicated overlapping research issue was found. Sources below were retrieved November 21–22, 2026. This is a focused engineering evaluation, not an exhaustive GIS literature review.

## 2. Existing architecture and gap

The [domain model](../03-Domain-and-Temporal-Model.md) already lists Place alongside Asset and Organization. [Genealogy](../04-Genealogy-Relations-and-Genetics.md) calls for dated residence, migration and historical jurisdictions; [events](../05-Events-Actors-and-Conflicts.md) call for locations and changing control; [Canvas](../06-Three-Layer-Entries-and-Canvas.md) specifies a Location template. Enhanced geography currently is P2 in [requirements](../01-Requirements-and-Invariants.md). There is no normative spatial predicate family or containment contract.

Observed source at the baseline:
- [lib.rs](../../crates/atlas-store/src/lib.rs): ObjectData and ViewContext; Place is absent in ObjectData. Context already carries world, branch, revision, day, scope.
- [network.rs](../../crates/atlas-store/src/network.rs): IdentityKind and NetworkKind lack Place and containment; NetworkInput already provides stable endpoint references, source and exact effective interval.
- [0032.sql](../../crates/atlas-store/migrations/0032.sql): registry/ghost/network kind CHECK allowlists require migration; prior records must retain IDs. Network subject/target indexes provide a useful pattern.
- [store.rs](../../crates/atlas-store/src/store.rs): resolve_object, inherited_metadata, indexed_relationship_path_at and query controls are existing building blocks. Some alias/discovery reads remain main-only.
- [store/family.rs](../../crates/atlas-store/src/store/family.rs) uses interval-intersection recursive ancestry checks; reuse the validated design pattern, not a second pedigree implementation. [store/network.rs](../../crates/atlas-store/src/store/network.rs) has branch restrictions and typed endpoint validation; #62 brings related branch overlays.
- [query.rs](../../crates/atlas-store/src/query.rs): QueryFilter, ObjectKindFilter, PredicateInput, saved parameters, cursor and limits need place extensions. [store/entries.rs](../../crates/atlas-store/src/store/entries.rs) and [layout.rs](../../crates/atlas-store/src/layout.rs) constrain authorized publication and dependency indexing. [access.rs](../../crates/atlas-store/src/access.rs) implements simple audience intersection; do not assume this alone handles spatial inference.
- [Cargo.toml](../../crates/atlas-store/Cargo.toml) uses rusqlite 0.37 with bundled/hooks; availability of R*Tree in the actual shipped build was not checked. #63 proposes raster assets and independent hotspots; maps should reuse that system after integration.

Conclusion: add a real Place identity through the existing registry; do not fake place as an organization, text label, layout component or asset. Retain one world database and existing temporal/policy machinery per D-001, D-004–007, D-009–013 and D-016.

## 3. Primary-source findings

| Source | Observed useful concept | Design consequence / limitation |
|---|---|---|
| Linked Places Format [R3] | Separately dated names, classifications, geometry and inter-place attestations; its Abingdon example changes administrative parent in 1974 | Separate stable place from changing statements; not an internal schema mandate, because Atlas must support unlocated fictional places, private branches and local coordinate frames |
| CIDOC CRM 7.1.3 [R4] | E53 Place, P89 spatial containment, P121 overlap, P122 border, P157 reference space and E9 movement; P89 is transitive and reflexive in that ontology | Distinguish place from occupying actor/asset; adopt a stricter irreflexive structural edge for authored hierarchy, with reflexive membership only in query semantics |
| GeoSPARQL 1.1 [R5] | Separate topological predicates such as within, touches, overlaps and geometry evaluation | Borrow vocabulary and boundary tests; do not adopt RDF/SPARQL infrastructure or assume administrative membership follows from coordinates |
| IndoorGML [R6] | Indoor navigation is a distinct spatial-modeling problem | Separate navigable connections from containment; walls/doors/floors do not imply access; no XML implementation required |
| SQLite recursive CTE [R1], R*Tree [R2] | Graph recursion and rectangle candidate lookup | Use bounded adjacency with appropriate indexes first; only add geometry index once needed. Bounding boxes do not prove exact containment; handle rounding and frame IDs |
| RFC 7946 [R7] | GeoJSON uses Earth WGS84 and longitude/latitude; antimeridian handling and uncertainty are explicit concerns | Use GeoJSON for explicitly compatible Earth data or an adapter, never assert fictional worlds or pixel plans are WGS84 |

The standards differ in semantics: CIDOC's inclusive spatial relation is not our strict structural-parent relation; a geometrical overlap does not imply administrative authority or an accessible route. Linked Places is an interchange profile, not a replacement for Atlas provenance or branch semantics. These distinctions are material, not merely naming preferences.

## 4. Alternatives and recommendation

| Approach | Strength | Failure / tradeoff | Decision proposed |
|---|---|---|---|
| Single parent column | Simple tree UI and breadcrumb | Cannot express overlapping classifications; moving/reparenting erases history unless versioned | UI projection only |
| Materialized path / nested sets | Fast subtree reads in a static tree | Expensive reparenting and unsuitable independent lenses/forks | Not canonical |
| Temporal closure table | Fast ancestor/descendant lookup | Quadratic worst case on a chain; interval, branch and permission path invalidation; hidden intermediate leakage | Optional derived cache only after profiling |
| Typed temporal adjacency | Supports multiple parents, independent meanings, unlocated places and history | Requires bounded traversal, cycle validation and indexes | Recommended foundation |
| Geometry-first GIS | Accurate spatial computations with known geometry | Can't author unknown places; coordinates alone cannot prove ownership, legal authority or semantic containment | Optional enhancement |
| External graph/GIS DB | Specialized tooling | Adds synchronization, distribution and offline packaging cost | Reject unless measured evidence overturns SQLite baseline |

**Recommendation:** versioned typed adjacency in the existing SQLite world store, strict containment DAG per selected lens (not one global DAG over all relations), geometry optional and independently versioned. One place can be found by several paths; result sets deduplicate canonical IDs. Navigation may show more than one breadcrumb and a chosen presentation parent can be scoped to an entry/view, not treated as the only real parent.

## 5. Proposed domain contract (illustrative, not accepted)

    Place { id: ObjectRef, place_type: PlaceTypeId }
    PlaceEdge { id, child: ObjectRef, parent: ObjectRef,
                kind: SpatialRelation, lens: optional validated hierarchy ID,
                interval, source, provenance, audience }
    GeometryVersion { id, place: ObjectRef, frame_id,
                      geometry, uncertainty, interval, provenance, audience }
    LocatedAt { id, subject: ObjectRef, place: ObjectRef,
                role: LocationRole, interval, provenance, audience }

All mutate/read through existing command and ViewContext. IDs, revision/branch records and names are shared, not copied. Place types are extensible labels (settlement, valley, room, route, orbit, region, building) with typed rules where needed; do not require fixed levels or world geometry. Type changes can be historical. Place is spatial identity; an estate building as property can also have a separate existing Asset reference, linked explicitly; do not duplicate ownership under Place. A polity's institution and territory are distinct entities.

| Predicate | Rule |
|---|---|
| ProperPartWithin (Place -> Place) | Strict, transitive within same compatible spatial lens/frame context; forbid active self/cycle. Multiple parents allowed where meaningful. |
| AdminPartOf (Place -> Place) | Transitive only within declared administrative hierarchy; no cross-jurisdiction inference. |
| Overlaps / AdjacentTo (Place <-> Place) | Symmetric, not transitive; no automatic containment. |
| ConnectedTo / AccessibleVia | Directed route/access link with optional direction, cost, validity, policy; cycles allowed; connection is not permission. |
| LocatedAt | Typed subject/role (birth, death, residence, presence, headquarters, event site, asset location, population origin); no generic temporal implication between roles. |
| Jurisdiction / Control / Claim | Institution/faction -> Place; separate legal governance, effective control and competing claims; no automatic equivalence. |
| Ownership | Reuse existing network ownership over asset; resolve asset's place through explicit link. |
| Successor / Split / Merge | Explicit historical event and effect, not a containment edge or automatic identity redirect. |

A room can be in a ship frame; a ship has a changing location through LocatedAt. Infer room's broad geographical context only when both historical placement and containment are authorized and compatible. A ship is a physical asset; an onboard interior Place may be tied to it by stable reference. Moving it never changes cabin IDs. Multiple floors may share XY positions but have different frames/elevations; connecting stairs are explicit.

Identity rules: rename and boundary correction keep identity by default; author decides whether a split/merger continues an old identity or creates successors. Unknown, destroyed, abandoned or mythical places retain record identity and appropriate status; do not equate an unknown coordinate with zero. A historical claim of Atlantis can be represented without asserting it canonically exists. Distinguish place lifetime, fact validity and author revision. Names/aliases, geometries, classification and memberships are independent revisions. Contradictory claims remain claims, not simultaneously asserted canonical exclusive parents.

## 6. Time, branches and invariants

Use half-open exact intervals and existing world-day/calendar abstraction. Never derive from edges that are only true at disjoint dates. Historical interval traversal intersects every edge's effective range. An exact chain is supported only over the nonempty intersection; for uncertain edges propagate definitely/possibly/not established through the existing uncertainty contract, with potentially correlated uncertainties not promoted by naive independence. Initially accept exact canonical intervals and keep uncertain facts as sourced candidate/claim records until typed uncertain containment has an approved gate.

On create/edit, resolve relevant branch overlay at expected revision; test affected intervals, including inherited facts and all changes staged in the same transaction. For proposed child -> parent [lo,hi), search from parent following the same strict edge/lens toward child while carrying interval intersection; reject if child reachable over any nonempty interval. Bound visited (node, interval) states and work; budget exhaustion rejects mutation rather than treating it as safe. Clip inherited history according to accepted fork rules; a later main revision must not alter a pinned fork. Branch-local removal hides only that fact in its branch. No global ancestor closure or current-parent cache as authority.

Pseudocode:

    pending = [(parent, proposed_interval)]
    while pending:
        node, span = pop(pending)
        check_budget()
        if node == child: reject(Cycle, span)
        for edge in effective_outgoing(node, kind, lens, branch, revision):
            overlap = intersect(span, edge.interval)
            if overlap and not already_covered(edge.parent, overlap):
                push(edge.parent, overlap)
    commit all facts, explicit effects and derived invalidations atomically

Validation is performed without reader filtering in authorized author mutation context, so hiding an edge cannot disguise a canonical cycle. Public queries require the opposite: filter root, nodes, edges, frames, geometries, provenance and assets BEFORE traversal and aggregate. If hidden intermediate connects two visible nodes, do not reveal the path or inferred ancestry by default. Only a separately released broader fact can disclose their association. Unauthorized and nonexistent roots use indistinguishable response; avoid per-secret counts, budget signals, cursor and error details. Timing side channels need adversarial tests, not assumed solved by row filtering.

## 7. Geometry and map authoring

Stage A works with no coordinates. Stage B supports independent raster map/floorplan assets, normalized hotspot coordinates bound to a specific version, an optional map extent/frame and semantic links. A hotspot is a link, not canonical geometry or containment. Map display may intentionally exaggerate or schematize.

Stage C introduces typed versioned geometry: point, line, polygon/multipolygon and optional z/floor; geometry role (representative point, boundary, footprint, route, uncertain extent) distinct. Frame includes explicit units/origin/axis convention, parent transform only when calibrated and appropriate; separate geodetic planetary frames from local planar/image frames. Frame transform is not universally available; reject cross-frame geometry comparison unless valid transform exists. For fictional planets establish radius/ellipsoid/longitude convention explicitly before calculating area or distance; use local planar model initially. Preserve old maps and geometries as historical assets; use controlled simplification for rendering only. Validate finite coordinates, size/vertex caps, polygon holes and ring validity using a vetted geometry library selected in a later bounded spike, not hand-built full GIS.

Optional R*Tree stores integer surrogate for geometry-version/fragment, bounding boxes and separate join to world/frame/version; candidates filtered by valid time, revision, scope and exact predicates in correct coordinate space. Beware float rounding and antimeridian/polar crossings; split wrapped extents or use suitable frame, then dedup. No exact containment based solely on bounding box. R*Tree is derived, rebuildable; feature-check bundled SQLite on all target platforms before enabling.

## 8. Query and studio workflows

Extend existing QuerySpec (version/pin compatibility and parameter types) with typed predicates conceptually:

    WithinPlace(root, lens, include_self, max_depth)
    LocatedIn(root, location_role, lens, include_descendants)
    SpatialRelationTo(place, predicate, geometry_role)
    PlaceName(name, optional_context_place)

Return scoped stable IDs, matched role, defensible certainty, provenance and truncation; deterministic pagination bound to view and query signature. Traversal uses indexed adjacency in world/kind/lens/child or parent direction, visited IDs at snapshot; interval queries carry spans. Cap depth, rows visited, expansions, memory, deadline and output size independently; cancellation and errors use current QueryControl. Composite filters do not broaden authorization. Index place aliases/titles and contextual names (two Springfields remain candidates). FTS indexes only authorized released strings; a private map feature must not leak through metadata or geospatial search. Invalidation includes prior/new ancestors, descendants and affected saved bindings; use revision-aware derived dependency indexes after measuring, not a precomputed full closure.

Node: create location with type/name and optional parent(s), switch lens, inspect alternate parents and sources, move/reclassify via dated command with effect preview. Canvas: reuse versioned templates and existing links, map/hotspot component, directory, breadcrumb and historical-bound query. Display: only authorized rendered scene and links, accessible list alternative; no hidden raw image regions (a secret room drawn on a public raster requires redacted derivative or withheld image). Historical selector changes all studios coherently; no frontend geometry as authoritative fact. Use current asset handling and typed IPC; do not add new application or renderer backend.

Example fixture: Northbridge fictional world: Valley -> town -> estate -> manor -> floor -> study. Town belongs to Kingdom A before day 100 and Kingdom B after; it also lies in a watershed and diocese. A secret vault has a public-sounding alias but no public edge. A manor transfer changes ownership, not location. A ship cabin moves between ports independently of interior location. A fork before conquest retains A until branch-local change. Birth at day 99 displays A, event at day 100 displays B; author correction preserves previous recorded revision. Distinct lens results and archived assets survive rename and restore.

## 9. Executed SQLite feasibility experiment

An in-memory Python sqlite3 experiment executed in the research environment (SQLite 3.46.1), not the application or target hardware. Nine assertions passed: exact half-open date boundaries, administrative vs physical lens, hidden-path pruning, world/revision filtering, deduplication, overlapping vs disjoint interval cycle. An R*Tree rectangle accepted point (9,9) for bounding box [0,10]² of triangle x+y<=10; actual triangle excludes it. This confirms candidate-vs-exact distinction. The initial unforced recursive query timed out in the tool; no duration or cause inferred. Revised explicit CROSS JOIN/INDEXED BY query used indexed (w,kind,parent,lo) lookups per EXPLAIN QUERY PLAN.

50,000-node synthetic ten-way tree, in-memory, seven runs each, median wall milliseconds including SQL result materialization and sorting, no branch overlay, revision withdrawals, per-node policy, geometries or production service:

| root | result count incl. root | median ms |
|---|---:|---:|
| 0 | 50,000 | 62.113 |
| 1 | 11,111 | 13.187 |
| 100 | 111 | 0.115 |

These are feasibility measurements, not an app SLA. Dense DAGs, interval explosion, expensive authorization, cold disk, deep chains, branch lineage and target hardware can differ substantially. No production regression was executed. The fixture deliberately filters only edge visibility (not node identity or secret timing); production must add full policy filtering.

Reproduction (Python standard library; no dependencies or app DB):

~~~python
import sqlite3, time, statistics
c = sqlite3.connect(':memory:')
print(sqlite3.sqlite_version)
c.executescript('''
CREATE TABLE edge(w INT, child INT,parent INT,kind TEXT,lo INT,hi INT,rev INT,public INT);
CREATE INDEX ep ON edge(w,kind,parent,lo,hi);
CREATE INDEX ec ON edge(w,kind,child,lo,hi);
''')
c.executemany('INSERT INTO edge VALUES(?,?,?,?,?,?,?,?)', [
    (1,1,2,'physical',0,100,1,1),(1,1,3,'physical',100,200,1,1),
    (1,2,4,'physical',0,200,1,1),(1,3,4,'physical',0,200,1,1),
    (1,1,5,'admin',0,200,1,1),(1,6,2,'physical',0,200,1,0),
    (1,7,6,'physical',0,200,1,1),(2,8,2,'physical',0,200,1,1),
    (1,9,2,'physical',0,200,2,1)])
q = '''WITH RECURSIVE d(id) AS (
 SELECT ? UNION SELECT e.child FROM d CROSS JOIN edge e INDEXED BY ep
 WHERE e.parent=d.id AND e.w=? AND e.kind=? AND e.lo<=? AND ?<e.hi
 AND e.rev<=? AND (?=1 OR e.public=1))
 SELECT id FROM d ORDER BY id'''
def desc(root,day,kind='physical',rev=1,author=0):
    return [r[0] for r in c.execute(q,(root,1,kind,day,day,rev,author))]
assert desc(2,99)==[1,2]
assert desc(2,100)==[2]
assert desc(3,100)==[1,3]
assert desc(5,50,'admin')==[1,5]
assert desc(2,50,author=1)==[1,2,6,7]
assert desc(2,50,rev=2)==[1,2,9]
assert desc(4,50)==[1,2,3,4]
cyc = """WITH RECURSIVE walk(id,lo,hi) AS (
 SELECT ?,?,? UNION SELECT e.parent,max(walk.lo,e.lo),min(walk.hi,e.hi)
 FROM walk CROSS JOIN edge e INDEXED BY ec
 WHERE e.child=walk.id AND e.w=1 AND e.kind='physical'
 AND max(walk.lo,e.lo)<min(walk.hi,e.hi))
 SELECT EXISTS(SELECT 1 FROM walk WHERE id=?)"""
def cycle(child,parent,lo,hi):
    return c.execute(cyc,(parent,lo,hi,child)).fetchone()[0]
assert cycle(2,1,20,40)==1
assert cycle(2,1,100,120)==0
c.execute('CREATE VIRTUAL TABLE bbox USING rtree(id,xmin,xmax,ymin,ymax)')
c.execute('INSERT INTO bbox VALUES(1,0,10,0,10)')
assert c.execute('SELECT id FROM bbox WHERE xmin<=9 AND xmax>=9 AND ymin<=9 AND ymax>=9').fetchone()==(1,)
assert 9+9>10
c.execute('DELETE FROM edge')
c.executemany("INSERT INTO edge VALUES(1,?,?,'physical',0,200,1,1)",
              ((i,(i-1)//10) for i in range(1,50000)))
c.commit()
for root in [0,1,100]:
    times=[]
    for _ in range(7):
        t=time.perf_counter()
        out=desc(root,50)
        times.append((time.perf_counter()-t)*1000)
    print(root,len(out),statistics.median(times))
print(c.execute('EXPLAIN QUERY PLAN '+q,(0,1,'physical',50,50,1,0)).fetchall())
~~~

## 10. Phase integration and acceptance plan

No new phase. Phases 0–2 remain officially closed unless owner accepts evidence requiring reassessment. Basic spatial identity/containment should be proposed as foundation scope; enhanced geography remains P2 until explicit reprioritization.

| Phase/issue | Deliverable proposal and gate |
|---|---|
| 0–1 [#1](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1), [#2](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/2) | ADR, identity, frame semantics, migration and typed place registry; preserve IDs, reject cross-world and stale writes (AT-01/14–16/26/33). |
| 2 [#3](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3) | Source-backed interval edges and event effects; temporal cycle and rollback tests (AT-04/05/13). |
| 3 [#4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4) | Bounded snapshot/interval query, contextual alias, saved query replay, index rebuild parity (AT-21–25/35). |
| 4 [#5](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/5) | Native create/edit/list workflow, template, raster map/hotspots, keyboard list, historic reader; no leaks (AT-17–20/22/34). |
| 5 [#6](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/6) | Residence, estate assets, migration and institution seats/jurisdiction; role distinction (AT-02/06/07/12). |
| 6 [#7](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/7) | Multiple event sites, conflict control/claim histories and explicit effects (AT-08–12). |
| 7 [#8](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/8) | Branch overlay, unknown/disputed places, hidden intermediate, redacted map, search/count/cursor/scene/export leakage tests (AT-13–16/20/25/34). |
| 8 [#9](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/9) | Dated population origins, founder migration relations, no ancestry-based cultural inference (AT-28–30 plus spatial fixtures). |
| 9 [#10](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/10) | Semantic maps/assets archive roundtrip, missing-frame/asset diagnostics, offline no-GPU navigation and dense workload (AT-23/24/27/31/32/36). |

Proposed spatial-specific tests (not official AT numbers pending approval): NL-01 unlocated create/reopen/rename; NL-02 overlap/no mixed-lens composition; NL-03 date boundary and author correction; NL-04 overlapping cycle rejection versus disjoint acceptance including batch; NL-05 cross-world/wrong-kind rejection; NL-06 moving vessel interior; NL-07 branch isolation including withdrawals and nested forks; NL-08 multiple event roles/sites and ownership separation; NL-09 same-coordinate different floor/frame and geometry candidate false positives; NL-10 secret ancestor/path/count/map/link/asset leakage; NL-11 saved query and index rebuild parity; NL-12 archive migration roundtrip and incompatible schema rejection; NL-13 cancellation, bounded graph expansion and dense 1k/10k/50k fixtures; NL-14 native Node/Canvas/Display historical coherence and accessibility. No pass claims for these production gates.

## 11. Ordered next implementation proposal and open decisions

1. Review this study and record acceptance/deferred boundaries in ADR; clarify strict containment vs geometric inclusive predicates, identity/type versioning and lens rules. Reconcile 01/02/03/08/09/10/12 and relevant specialist docs, optionally add 13-Spatial-and-Nested-Location-Model.md.
2. Integrate current #62/#63 first and re-audit exact baseline; do not assign a migration number in advance. Add Place through registry, allowlists, ghost/discovery and query kind filters. Reuse branch-aware typed metadata mechanism if it satisfies interval/cycle invariants; otherwise one narrowly typed edge table sharing revision/branch machinery. Prove single native vertical slice with no geometry: place hierarchy, event location, dated reparent, lookup, Canvas and public page.
3. Add richer location roles, institution and conflict links, author location browser and map links. Only after safe native projection, add optional coordinate frames and exact geometry backed by mature tested libraries. Keep maps usable offline and absent.
4. Profile real workloads, including high fan-out and overlapping historical DAG, before adding closure/materialized cache, GIS extension or GPU-specific code. Later study routing/3D/planning and planetary transforms only if needed.

Open decisions: precise place-vs-asset identity policy, uncertain canonical containment representation, lens and hierarchy cardinality metadata, audience defaults for location identity, geometry library and licensing, exact transform/planet coordinate conventions, branch migration and public query dependency leakage. R*Tree support in bundled shipping SQLite and renderer interaction not validated. Prototype does not test fork semantics or real native app.

**Conclusion:** a real spatial dimension is feasible without replacing Rust/SQLite or forcing full GIS. Strongest value is stable, historical, typed location relationships integrated with existing world facts. Research is ready for design review; no normative docs, phase statuses, production schemas or application code changed in this pass.

## References

- [R1] SQLite, [The WITH Clause, sections 3.2–3.4](https://sqlite.org/lang_with.html), living documentation; recursive graph queries and traversal ordering.
- [R2] SQLite, [The R*Tree Module, sections 2, 3.4, 4](https://sqlite.org/rtree.html), living documentation; candidate range search, build option and precision.
- [R3] LinkedPasts, [Linked Places Format README](https://github.com/LinkedPasts/linked-places-format), source documentation retrieved Nov 21–22, 2026; sections on when, names, geometry, relations.
- [R4] CIDOC CRM SIG, [CIDOC CRM v7.1.3](https://cidoc-crm.org/html/cidoc_crm_v7.1.3.html), E53, P89, P121, P122, P157 and E9/P26/P27.
- [R5] OGC, Car et al. (eds.), [GeoSPARQL 1.1, OGC 22-047r1](https://docs.ogc.org/is/22-047r1/22-047r1.html), published 2024-01-29; topological relations and geometry querying.
- [R6] OGC, [IndoorGML standards catalog](https://www.ogc.org/standards/indoorgml/), 2.0 conceptual model / 1.1; indoor navigation design motivation.
- [R7] Butler et al., [RFC 7946: The GeoJSON Format](https://www.rfc-editor.org/info/rfc7946/), August 2016, sections 3.1.9–3.1.10 and 4; antimeridian, precision/uncertainty and WGS84.
- Internal [requirements](../01-Requirements-and-Invariants.md), [architecture](../02-System-Architecture.md), [domain](../03-Domain-and-Temporal-Model.md), [query](../07-Search-Query-and-Discovery.md), [storage](../08-Storage-and-API-Contracts.md), [acceptance](../09-Acceptance-and-Verification.md), [roadmap](../10-Delivery-Roadmap.md), [decisions](../12-Decisions-and-Reconciliation.md).
