# Implementation foundation

## Repository audit

The baseline was [main at 4447975](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/444797563488dfcefa75670d4062ed0dbafe5141). Its 17 tracked files were documentation. GitHub showed one branch, ten open roadmap issues, no closed issues, and no pull requests. The feature and research folders contained index/process documents. The first code therefore establishes the Rust workspace, migration, persistence API, and repeatable checks from scratch.

[Phase Zero #1](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1) remains open. The desktop foundation is merged with a verified Canvas2D/DOM fallback on the hosted Linux and Windows paths. Hardware WebView2 acceleration and recovery remain a separate verification gate. The [persistence ADR](ADRs/phase-1-persistence-slice.md) records choices that affect downstream phases.

## Shared contracts and rollout

| Roadmap phase | Foundation it inherits | First work still required for the phase |
| --- | --- | --- |
| [0 — Platform](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1) | Cargo workspace and Linux/Windows verification workflow | Validate the accelerated path and recovery on GPU-capable Windows hardware |
| [1 — Persistence](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/2) | World-scoped identity and migrations, pinned forks, versioned calendars, typed claims, and sourced event effects | Consume the storage contracts in the Phase Two relationship and event workflow |
| [2 — Relationship and event history](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3) | Historical event identity, source references, atomic multi-object commands, transactional invalidations | Add typed relationships and event/action/effect records, then verify the Northbridge betrayal scenario |
| [3 — Query and discovery](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4) | Explicit read context, stable revisions, typed records, committed invalidation rows | Add one bounded query service, an index consumer and deterministic Northbridge query fixtures |
| [4 — Node, Canvas and Display](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/5) | Stable object references, scope boundary and versioned reads | Bind all three surfaces to the shared query service and verify public projection policies |
| [5 — Genealogy and organizations](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/6) | Stable person identity, typed endpoint validation pattern and historical records | Add distinct kinds for kinship, organizations, offices and memberships, with cycle and chronology validation |
| [6 — Agency and conflict](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/7) | Stable event identity and atomic writes with source provenance | Model actions, affiliations, explicit effects and conflict positions feeding a shared impact query |
| [7 — Evidence and privacy](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/8) | Versioned source records, branch-aware read inputs and a facade that denies public reads | Implement authorization from a trusted host capability, branch overlays, claims/knowledge projections and leak tests |
| [8 — Synthetic genetics](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/9) | Stable person references and transaction-coupled invalidation | Derive synthetic results from versioned accepted biological parentage and versioned simulation seeds |
| [9 — Release and portability](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/10) | Schema identity/version, standalone SQLite backup and referential integrity checks | Add a portable manifest and assets, verify migration and export/import, then measure the release build |

The tests introduce a small Northbridge persistence fixture. Extend it as typed relationships, evidence, query, and publication workflows become executable. Keep canonical state in the same Rust/SQLite model and consume its authorized projections from the desktop and renderer.

## Desktop workspace integration

The Phase One branch builds on the desktop and WASM renderer from Phase Zero. The root workspace has one lockfile for `atlas-desktop`, `atlas-renderer`, and `atlas-store`. The store remains a Rust library. The desktop still opens the read-only platform fixture through Tauri IPC.

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

Phase Two adds the relationship and actor-action workflow on these storage contracts. Calendar discontinuities, branch merging, full archive roundtrip and public projections remain outside this foundation. Phase Zero hardware validation remains tracked in issue #1.
