# 07 — Universal Search, Typed Queries, Backlinks and Discovery

**Status:** Normative shared query-service contract; search is a foundation, not a late polish phase.

## 1. One engine, three search experiences

**Global search:** names, IDs, aliases, tags, text, sources, media descriptions, organizations, actor roles, conflicts, page content and saved queries; ranking/filters, disambiguation, result provenance and jump-to. Keep current world, branch, date, author revision and perspective.

**Node Studio:** filter objects, edge types, directionality, path/depth and neighbor count; ancestry/common ancestor, event impact, faction intersections and "how are A and B connected?". Discovered names may expand authorized graph neighbors or create *proposals*, never indiscriminately add canon.

**Canvas Studio:** search component/primitive palette; picker for objects and media; find within page; query builder with preview/binding; dependency inspector for components, templates and inline references.

**Display Studio:** search the **published authorized projection**, including permitted content and linked component data. Query, snippet, facet, result counts, link previews, backtracking paths, HTML exports and image annotations must all obey identical policies.

## 2. Versioned typed query AST

Saved/inline queries share an AST that Rust validates, normalizes and executes. No arbitrary SQL or JavaScript embedded in Canvas components.

~~~rust
struct QueryContext {
    view: ViewContext, deadline_ms: u64, max_results: u32
}
struct QuerySpec {
    schema_version: u32,
    source: QuerySource,
    predicate: Option<Predicate>,
    traverse: Option<TraversalSpec>,
    project: Vec<FieldRef>,
    order_by: Vec<SortKey>,
    pagination: PageSpec,
    output: OutputShape
}
enum QuerySource {
    Objects(ObjectKind), Relationships, Memberships,
    EventInvolvements, ActorActions, Conflicts,
    Claims, AuthoredBlocks, PublishedBlocks, Links
}
enum Predicate {
    And(Vec<Predicate>), Or(Vec<Predicate>), Not(Box<Predicate>),
    FullText(SearchText), HasTag(TagId),
    HasRelationship { kind: RelationshipKindId, target: ObjectId },
    InInterval(TimeSpan), HasSource(SourceId),
    FieldCompare { field: FieldRef, op: Comparison, value: Scalar }
}
struct TraversalSpec {
    start: Vec<ObjectId>, relation_types: Vec<RelationshipKindId>,
    direction: Direction, max_depth: u8, max_visited: u32
}
enum OutputShape { ObjectRows, Edges, Paths, Aggregate, TimelineSeries }
struct QueryResult {
    rows: Vec<TypedRow>, next_cursor: Option<OpaqueCursor>,
    source_revision: AuthorRevision, truncated: bool,
    warnings: Vec<QueryWarning>
}
~~~

These are contracts to refine, not definitions of an implemented API. World IDs, field descriptors, enum names and serialization versions must be validated together before code generation.

## 3. Planner and execution

1. Parse natural search syntax or receive a serialized typed query.
2. Resolve label/alias to IDs with ambiguous candidate results; never guess that "the guild" means one organization.
3. Validate AST schema and allowed fields/operators for source kinds.
4. Freeze ViewContext: world, branch, fictional time, author revision, audience, perspective and capability.
5. Derive authorized object/field/source sets **before scoring, aggregation or path traversal**.
6. Plan parameterized SQL for typed predicates, FTS5 indexes for text, bounded recursive SQL/Rust traversal for graph paths, and projected temporal snapshots.
7. Return stable pagination, explicit uncertainty semantics, provenance, data revision, counts only over authorized rows and truncated indicators.
8. Track dependency signature; schedule affected read-component refreshes after writes.

Start with explicit query builders and a modest human-readable filter syntax. Optional natural-language-to-query by an AI service is a suggestion layer and must show its parsed query for approval.

## 4. Query semantics

- **Temporal:** at time T, during interval I, before/after event E, overlap/not overlap, current as of recorded revision R. Unknown date can be definitely true, possibly true or not established; clients must not silently force possible to true.
- **Graph:** direction, relationship kinds, traversal depth, visited-node budget, path length, shortest authenticated path and event causal impact. Traversals check visibility at every step; hidden intermediate nodes cannot be exposed by public "connected" counts.
- **Genealogical:** biological parent vs legal parent vs household membership; ancestry is derived from accepted parentage edges and validity intervals.
- **Relational:** actor within event, group participation, conflict position per issue, co-membership, ownership transfers, provenance.
- **Text:** full-text phrase, aliases, contextual mention, title, source, captions and field-filtered search. Suggestions are distinct from accepted relationships.
- **Aggregate:** histogram, count, time series, group-by, never across hidden records or ambiguous time contexts.

## 5. Persistence and indexes

SQLite typed tables remain canonical. Build FTS5 for names, aliases, descriptions, source text and permitted authored blocks. Use separate derived author and publication indexes or policy-aware prefiltering where proven safe; never publish raw author index then hope to filter snippets afterward.

Maintain materialized adjacency for fast neighbor lookup; link/backlink indexes keyed by source block and target object; alias/mention candidates with confidence and source ranges; component dependency metadata; publication caches keyed by world/branch/date/revision/audience/perspective/policy version.

All derived indexes are disposable and rebuildable. On edit, mark affected object IDs and source spans; on policy changes invalidate publication index/caches **before** serving. Use bounded work queues, throttled incremental re-indexing and cancellation; late async UI replies must not overwrite newer query results.

An alternate text index (e.g., Tantivy) is an ADR after measurements, not a second source of truth.

## 6. Discovery, auto-linking and node proposals

Discovery pipeline:

1. Extract explicit text links, image hotspots, names and aliases with location and field context.
2. Normalize tokens; match existing IDs with candidate ranking and collision list.
3. Evaluate hints from typed data (e.g., co-attendance at event) without declaring unproven relationships.
4. Suggest link insertion, relation evidence, new draft ghost node or duplicate-merge review.
5. Let author confirm/decline/snooze; record decision and source. Canonical edits go through the same Rust command validation as manual edits.
6. Reprocess only affected paragraphs/images/aliases on change, not the whole world.

A sentence "Eleanor visited the Guild" may warrant link to the guild and event proposal. It does **not** prove she was a member. A recognized portrait hotspot only creates a reference, not a biological or social bond.

## 7. Saved queries and Canvas bindings

SavedQuery stores stable ID, AST schema, author revision, parameters/types/defaults, owner/scope/visibility, description, output schema and history. A component binds either inline spec or saved ID plus typed parameters. Reuse the same source result as a Node graph highlight, Canvas table, Display list or timeline series, with visibility and row-budget constraints applied on **each execution**. Saved-query permissions never exceed the invoker's capabilities.

Example author query: "descendants of House Vale who belonged to the merchant guild 1850–1900, participated in a conflict, and experienced event-attributed relationship change." Planner combines kinship, membership intervals, event involvement, historical effects and evidence. Author can inspect the logical plan and uncertainties; public output may be smaller.

## 8. Safety, resilience and acceptance

- Explicit permission and context checks for every nested query, graph scene, asset, citation, snippet and backlink.
- Bound max depth/visited/results/bytes/compute; return cursor/truncated flags and a suggested narrower filter.
- Parameterized SQL; allowlisted field registry; no raw SQL in user queries; no dynamic code execution inside pages.
- Index rebuild reproduces semantically equivalent authorized results; rename/alias does not orphan backlinks.
- Query cached under one audience cannot be read by another; policy change invalidates existing outputs.
- Two views of a dated query at different time/revision coordinates produce appropriately different results.
- Text mentions can propose links and draft nodes; membership or causality requires explicit structured approval.
- Keyboard-accessible result lists serve as graph fallback; results retain navigation context across studios.

## 9. Spatial query extensions

[13](13-Spatial-and-Nested-Location-Model.md) defines WithinPlace, LocatedIn, SpatialRelationTo, contextual PlaceName and scoped MapFeatures projection as extensions of the existing versioned QuerySpec and saved-query bindings. Typed/lens-specific bounded adjacency; intersect effective intervals along historical paths, retain uncertainty, filter hidden paths before traversal/aggregation/map derivation. Viewport is not privilege. Stable IDs/cursors, authorized labels and explicit incomplete geometry. Derived spatial indexes/caches rebuildable; invalidate prior/new ancestor, geometry, frame, recipe and policy dependencies. R*Tree is optional candidate lookup, never exact truth.

## Research #69 disposition (scope clarification)

[Reviewed Transform study](Research-Studies/maltego-inspired-transforms.md) and [closure record](ADRs/phase-3-contextual-exploration-review.md) clarify the existing Node exploration/query entry points in sections 1, 3 and 7: selection-bound contextual actions may reuse validated query/domain services, subject to existing phase gates. No separate executable engine, independent canonical graph, new feature scope or reader privilege. Reusable typed multi-stage recipe definitions and dedicated lore workbench are deferred pending distinct author value and explicit New Feature approval. This note adds no original roadmap obligation or production acceptance gate.
