# Implementation foundation

## Repository audit

The baseline was [main at 4447975](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/444797563488dfcefa75670d4062ed0dbafe5141). Its 17 tracked files were documentation. GitHub showed one branch, ten open roadmap issues, no closed issues, and no pull requests. The feature and research folders contained index/process documents. The first code therefore establishes the Rust workspace, migration, persistence API, and repeatable checks from scratch.

[Phase Zero #1](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1) tracks the platform acceptance evidence. The desktop foundation is merged with a verified Canvas2D/DOM fallback on the hosted Linux and Windows paths. Physical Windows WebView2 acceleration and recovery are verified in the [Phase Zero log](Phase-0-Verification.md). The [persistence ADR](ADRs/phase-1-persistence-slice.md) records choices that affect downstream phases.

## Phase handoffs

The first fixture in each row is the smallest dependency-ready slice. Keep stable IDs, the Rust write path and explicit read context across phases. The roadmap and issue acceptance criteria remain authoritative.

| Phase | Inherited implementation | First fixture and gate |
| --- | --- | --- |
| [0 — Platform](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1) | Cargo workspace, Tauri desktop, Canvas2D and an accessible inspector | The 2,000-node benchmark verified zoom, pan and device-loss recovery on an RTX 2060 (AT-32). |
| [1 — Persistence](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/2) | World-scoped SQLite migrations, pinned forks, versioned calendars, sourced claims and atomic effects | Reuse the migration/reopen and failed-command suites when introducing new record kinds (AT-01, AT-14–16, AT-26, AT-33). |
| [2 — Relationships and event history](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3) | Saved directed relationships, action-specific effects and a day/revision graph inspector | Extend the Northbridge letter disclosure with configured definitions, sourced claims and viewer-specific evidence. Prove a failed action remains failed even when a witness says otherwise (AT-03–05, AT-08–09, AT-13 core). |
| [3 — Query and discovery](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4) | Explicit read context, stable revisions, typed records and committed invalidations | Add a bounded, versioned Rust QuerySpec. First fixture combines an event-role filter, dated graph traversal and ambiguous name discovery. Compare incremental and rebuilt results (AT-21, AT-23–24, AT-35). |
| [4 — Node, Canvas and Display](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/5) | Stable object references and the Phase 3 query service | Bind a relationship timeline to a versioned layout element. Reparent it without changing the binding, then verify public compilation excludes a hidden nested query (AT-17–20, AT-34). |
| [5 — Genealogy and organizations](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/6) | Stable person identity, typed role validation, branch/day/revision reads and shared queries | Create one biological and one adoptive link for the same child. Validate cycles on accepted biological edges and keep public disputes separately queryable (AT-02). |
| [6 — Agency and conflict](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/7) | Event roles, actor actions, explicit effects and stable memberships from Phase 5 | Model a three-sided strike with one dated defection and one independently authored causal edge. Bound impact traversal and require explicit attribution (AT-10–12). |
| [7 — Evidence and privacy](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/8) | Source-backed records, pinned forks, baseline evidence projections and publication/query entry points | Exercise one secret across branch, date, viewer, graph paths, snippets, aggregates and cache refresh. Rust must exclude it before serialization (AT-13, AT-20, AT-25, AT-34). |
| [8 — Synthetic genetics](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/9) | Accepted biological pedigree and revision-coupled invalidation | Pass deterministic, explicitly synthetic fixture inputs to a versioned derived model. Verify a legal-record correction leaves that model untouched and a biological correction invalidates it (AT-28–30). |
| [9 — Release and portability](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/10) | Schema guard, SQLite backup, referential integrity and desktop fallback | Roundtrip a world with historical records and assets. Record checksums, offline startup and bounded traversal on the target Windows hardware (AT-27, AT-31–32, AT-36). |

The Northbridge fixture is the common scenario. Extend it with each delivered slice and reuse canonical Rust data across desktop, renderer, query and publication surfaces.

## Desktop workspace integration

The Phase One branch builds on the desktop and WASM renderer from Phase Zero. The root workspace has one lockfile for `atlas-desktop`, `atlas-renderer`, and `atlas-store`. The store remains a Rust library. The desktop opens the Northbridge platform fixture and supports persistent author worlds through Tauri IPC.

On Linux, workspace formatting, strict Clippy, the desktop scene test, and the persistence integration tests passed together at workspace integration. The foundation suite has seventeen store tests across persistence, calendars and evidence. The dedicated persistence workflow builds only `atlas-store` so it can also run without native desktop libraries.

## Verified Phase One behavior

Run from the repository root with stable Rust and a C compiler for bundled SQLite.

```sh
cargo fmt --all -- --check
cargo clippy -p atlas-store --all-targets --locked -- -D warnings
cargo test -p atlas-store --locked
```

| Scenario | Observed result on Linux, Rust 1.100.0 |
| --- | --- |
| AT-01 | Created world and typed objects, corrected a person label, reopened original and backup, recovered the original label and source reference |
| AT-15 | Corrected a historical event's day without changing its ID. Old revision remained readable. The old day disappeared from the corrected view |
| AT-16, occurrence slice | Preserved normalized uncertainty bounds and the original label. Distinguished possible, definite and unestablished days |
| AT-26 | Rejected cross-world references and wrong-kind sources, and denied public scope before reads |
| AT-33 | A bad second event effect rolled back the event, the first effect, versions, audit and invalidation rows. A valid retry committed sourced effects with exact before and after versions. |
| AT-14 | Exact-day and event-anchored forks preserved pinned parent history after parent corrections. Nested forks kept sibling evidence isolated, and invalid pre-fork edits rolled back. |
| Calendars | Signed years, named eras, leap cycles, partial-date bounds, and pinned exact-date provenance survived reopening and correction. |
| Sourced claims | Statement corrections retained prior revisions and preserved the canonical subject. Missing source was rejected. |
| Successive fictional states | A later fictional name preserves the earlier interval at the same author revision. Corrections and schema-v1 migration retain prior author views. |
| Migration guard | Refused a newer schema without rewriting its version |

Phase Two adds the relationship and actor-action workflow on these storage contracts. Calendar discontinuities, branch merging, full archive roundtrip and public projections remain outside this foundation. Phase Zero hardware results are recorded in the [platform verification log](Phase-0-Verification.md).
