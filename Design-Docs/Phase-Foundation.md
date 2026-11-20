# Implementation foundation

## Repository audit

The baseline was [main at 4447975](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/444797563488dfcefa75670d4062ed0dbafe5141). Its 17 tracked files were documentation. GitHub showed one branch, ten open roadmap issues, no closed issues, and no pull requests. The feature and research folders contained index/process documents. The first code therefore establishes the Rust workspace, migration, persistence API, and repeatable checks from scratch.

[Phase Zero #1](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1) remains open. The headless persistence slice can be verified independently of the Windows desktop shell and WebGPU/fallback experiment. The [persistence ADR](ADRs/phase-1-persistence-slice.md) records choices that affect downstream phases.

## Shared contracts and rollout

| Roadmap phase | Foundation it inherits | First work still required for the phase |
| --- | --- | --- |
| [0 — Platform](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1) | Cargo workspace and Linux/Windows verification workflow | Run a Tauri desktop shell and observe the WebGPU and accessible fallback paths on Windows |
| [1 — Persistence](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/2) | World-scoped IDs, root branch, recorded versions, valid-time reads, date uncertainty, and SQLite migrations | Add typed claims and expand provenance, including original calendar references on events |
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

On Linux, workspace formatting, strict Clippy, the desktop scene test, and the persistence integration tests passed together at workspace integration. The pinned-fork update then passed eleven store integration tests locally. The dedicated persistence workflow builds only `atlas-store` so it can also run without native desktop libraries.

## Verified in the first slice

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
| AT-33, persistence batch | Rolled back identities, versions, audit and invalidation rows. Retried successfully and rejected duplicate and stale commands |
| AT-14, exact-day forks | A child retained its pinned parent history, diverged after the cut, and reopened from a backup. Nested forks kept sibling evidence isolated, and invalid pre-fork edits rolled back. |
| Calendar registration and conversion | Signed-year and leap-cycle conversions, inclusive partial-date bounds, world isolation, rule lookup after reopen, and the stability of already-recorded event dates were verified. |
| Successive fictional states | A later fictional name preserves the earlier interval at the same author revision. Corrections and schema-v1 migration retain prior author views. |
| Migration guard | Refused a newer schema without rewriting its version |

AT-14 needs a parent/child fork implementation. AT-33's linked historical effects need the typed event/effect workflow. Windows desktop runtime, the WebGPU and accessible fallback paths, full archive roundtrip, and public projections are still open.
