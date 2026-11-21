# Phase 3 — Bounded Typed Composite Query Predicates

**Status:** Partial QRY-01 and ADR-008 vertical slice. Audited `main` `abed4daffe5a03f1a8200025b167d6197a21a5c5`, full tree, README, design contracts/ADRs, migrations 0001–0011, Phase 3 issue, recent CI, tests and screenshot index.

## Decision

Extend the shared Rust `QueryFilter` (schema v1 additively) with typed `Kind`, `HasSource(ObjectRef)`, `And`, `Or` and `Not`. These are **author-scope, historical object-snapshot predicates**. `Kind` uses an allowlisted enum covering the actual current `ObjectData` kinds; `HasSource` requires the same world, a visible source identity and exact stable-ID references, not fuzzy source labels. Boolean predicates run against **one shared snapshot** at one world/branch/day/author revision.

Validate the entire tree **before any short-circuit evaluation**. Maximum 4 nested edges, 32 total nodes, 8 children per boolean group; AND/OR need at least 2. Excess depth/width, unsupported leaf filters, empty names and wrong-world IDs produce explicit errors. The query entry point also rejects nonexistent/not-yet-created branches before materializing candidate results, including an empty world. The existing 2,000-visible-object snapshot cap returns a budget error rather than incomplete results. Queries sort by label and stable object ID before bounded truncation (1–500). No author can infer historical relationships or elevate permissions by combining predicates.

Indexed FTS5, traversal, backlinks and participation searches are **not silently composable**: such nested leaves are rejected pending a correct candidate-set planner. No new database schema or separate authoritative state.

The Node author UI adds an accessible typed builder for label, object kind, source, AND/OR and NOT using a new allowlisted Tauri command. All results reuse existing graph fallback, world revision and stale-response handling. The Rust backend independently validates; TypeScript is a convenience view, not a permission gate.

## Evidence and remaining work

Rust fixtures test nested AND/OR/NOT, source-kind intersection, time/branch/revision, permission checks, wrong-world and unsupported leaves even after short-circuiting, plus limit/depth/breadth gates. A browser-mocked native regression checks IPC shape and query result UI. No authentic native screenshot is claimed absent a real running session.

**Outstanding:** saved typed composite AST and parameters, secure Canvas/Display binding, indexed large-world composition, sort cursor/pagination, deadline/cancellation, aliases and scale benchmarks. Keep Phase 3 issue #4 open.
