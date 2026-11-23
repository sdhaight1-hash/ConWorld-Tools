# 02 — System Architecture: Rust, HTML and wgpu

**Status:** Selected baseline, subject to explicit platform feasibility gates.

## 1. Logical architecture

~~~text
Tauri 2 desktop shell
  HTML/CSS/TypeScript UI (React is a view-layer choice)
    World Bible directory / entry shell / authorized reader
    Semantic controls + Canvas composition | contextual connections/graph
    Hierarchy | Genealogy | Organization | Event | Conflict | Map | Evidence
       |
       | typed IPC commands and read-only queries; no direct SQL
       v
Rust application/use-case facade
  validation + transactions + event notification + authorization
       |
       +-- Domain engine (typed invariants, no GUI/storage dependency)
       +-- Query service (temporal snapshots, graph, FTS, knowledge)
       +-- Canvas compiler (bindings, templates, publication)
       +-- Evidence/discovery/asset services
       +-- Optional genetics/generation/NPC adapters
       |
       v
SQLite world store + versioned attachments + rebuildable indexes

Native app --> scoped immutable Scene DTO --> wgpu-WASM WebGPU canvas
                                 \--> Canvas2D/SVG/DOM fallback
~~~

**Source of truth:** native Rust and SQLite. Frontend caches are temporary. wgpu view is a renderer and interaction hit-testing aid, never a historical logic engine.

The entry shell coordinates existing commands and scoped queries under one supported selection/context. Semantic controls explicitly mutate canonical facts via Rust; layout elements bind those facts without owning them. Graph coordinates and hierarchy presentation remain view metadata. Article-only editing must not depend on loading the whole graph. This is a UI reconciliation, not an additional storage representation or native service.

## 2. Rust workspace organization

Start with a modest workspace, extract optional crates only when stable:

- **atlas-types:** WorldId/ObjectId/BranchId/Revision/WorldDay, date uncertainty, ViewContext, typed errors, command/response DTOs.
- **atlas-domain:** people, lineage, relations, tags, organizations, events, actions, conflicts, claims; pure invariants.
- **atlas-store:** SQLite migrations, FK checks, typed repositories, revision transactions, object/version persistence and projection invalidation.
- **atlas-app:** command bus, authorization, UoW transactions, query facade and project lifecycle.
- **atlas-query:** typed QuerySpec planner, FTS5 adapter, bounded graph traversal, pedigree, temporal snapshots, aggregate results.
- **atlas-canvas:** layout schema, template version pinning, binding evaluation and publication compiler.
- **atlas-renderer:** Rust/wgpu scene consumer compiled for wasm32; window-independent scene DTO.
- **Later:** atlas-genetics, atlas-generation, atlas-npc-adapter as separate features; avoid premature micro-crates.
- **apps/desktop:** Tauri command handlers, WebView capability checks, HTML/CSS/TypeScript UI, theme and accessibility behavior.

Domain depends on types only; persistence depends on domain contracts; renderer depends only on read-only scene structures. No Tauri, SQL, browser APIs or wgpu imports inside domain rules. No world-state mutation from renderer hit tests.

## 3. Browser vs GPU boundary

Native wgpu cannot draw directly into a Tauri HTML DOM element. An integrated canvas requires a Rust-to-WASM build of the renderer using WebGPU, tested in the embedded WebView2 runtime. **Feasibility gate:** render 2k labeled nodes, selection, zoom/pan and device-loss recovery inside a Tauri window. Evaluate fallback: DOM list + HTML/CSS/SVG/Canvas2D scoped view; app MUST remain usable without WebGPU. A separate native wgpu window is an alternative, not a simultaneous default implementation.

Only graph/genealogy/map/historical-density views warrant GPU acceleration. Text, editors, tables, rich fields, page components and search results belong in accessible HTML.

GPU scene DTOs include nodes, edges, styling tokens, labels and authorized hit-test IDs **already filtered** by Rust. Never send hidden metadata and rely on shader clipping.

## 4. Mutation and read flow

~~~text
User edits a relationship
-> Tauri typed command {world, branch, expected_revision, fields}
-> Rust validates identity, semantics, permissions, chronology
-> single SQLite transaction:
     register version / typed payload / causal effect / audit
     enqueue index & component invalidations
-> commit -> version notification
-> QueryService(ViewContext) computes authorized refreshed projection
-> Canvas rebind + search update + graph scene refresh
~~~

All writes are idempotent by command ID or fail clearly on conflict; they never silently overwrite stale edits. An authored event that explains a change is different from an internal WriteAudit entry. An authored correction may change fictional history while preserving previous author revisions.

For reads: validate ViewContext (world, branch, fictional instant/interval, author revision, perspective/audience, capability); calculate authoritative state; apply policy **before** results leave Rust; apply ordering/pagination/scene budget; return source revision and truncation warnings.

## 5. Storage, migration and packaging

- SQLite with foreign_keys=ON, WAL when safe, STRICT tables where applicable, unique world-scoped keys and indexed time ranges.
- Relational typed tables for core semantics; JSON only for explicitly versioned extensible component props and world-defined metadata.
- Rebuildable FTS5, backlink, adjacency, publication and dependency indexes; maintain via transactional outbox or atomic invalidations.
- Worlds portable in a manifest-versioned archive containing DB, assets, rulesets/seed, template pins and validation checksums. No implicit references to another world.
- Autosave, backups, migration dry run, downgrade refusal with actionable error. Never run partial destructive migrations on open.

## 6. Failure containment

- Unsupported WebGPU -> Canvas2D/DOM; GPU device loss -> reconstruct scene without data loss.
- Invalid partial date -> unknown/range value, never silently pick January 1.
- Stale UI revision -> structured conflict response; reload/diff; no arbitrary last-write-wins.
- Broken image/link -> preserve reference, show missing asset, create diagnostic.
- Query too broad -> bounded partial result and explanation, not process freeze.
- Secret content -> filtered before counts, result excerpts, assets, scene DTO and cache.
- Genetics unavailable -> ordinary genealogy and publication continue.
- External AI/NPC unavailable -> normal world authoring continues.

## 7. Performance and accessibility gates

Use 1k/10k/50k seeded object fixtures. Measure time-to-open, 1-hop graph focus, FTS latency, memory, scene build duration, canvas rebind count and index-rebuild time. Do not assert a target percentile without hardware benchmarks: set budgets in an ADR after an instrumented smoke test. Always provide keyboard-operable DOM alternatives to rendered graphs, respecting high contrast and reduced motion.

## 8. Shipping boundary

The entire application need not exist to demonstrate correctness. A first useful vertical slice contains two characters, one reified relationship, a dated betrayal event with actors and claim/source, a valid-time change, a basic Node graph, one Canvas bound timeline component and public Display search. That slice exercises all crucial boundaries; build it before broadening the plugin catalog.

## 9. Shared spatial and cartographic pipeline

[D-017–D-020 and 13](13-Spatial-and-Nested-Location-Model.md) extend the existing domain/store/query facade: canonical Place and spatial facts/optional geometry → scoped query → derived schematic/exact/draft scene → SVG/Canvas2D/optional wgpu. No separate database, renderer authority or compulsory GIS; SVG first for bounded maps and accessible list, target measurements gate acceleration. Apply authorization before any procedural derivation or labeling; assets and recipes use existing persistence/archives.
