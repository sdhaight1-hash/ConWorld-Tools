# ConWorld Tools · State of development
**November 21, 2026 · Living History Atlas**

The downloadable source is usable as a developer build for a local author who wants to record a small world's people, relationships, events and disputed accounts, then search how that history looked on a chosen day or author revision. It is still an early product for someone who wants to build family trees, design character pages or publish a navigable atlas. Three of ten roadmap tickets have accepted their scoped deliverables. Phase 3 is active. The six later phases vary greatly in size, so counting tickets would give a misleading estimate of total engineering effort.

## What was audited

The baseline is [main at 4ef30e7](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/4ef30e7cd18b833938ccc54c10c6dafc3e0e2734). The inventory covers 132 tracked files, the desktop application and both library crates, migrations, both CI workflows, the README, canonical Design-Docs, 22 ADRs and the 11 indexed development images. GitHub returned ten roadmap issues, 32 closed PRs and four open PRs. No Git tags were advertised. The live README supersedes the attached copy. The detailed [engineering audit](../Design-Docs/Repository-Audit-2026-11-21.md) contains the acceptance-ID crosswalk and findings A01–A15. Comparing its main baseline with today's main shows only four documentation/media paths changed. Its code findings therefore still apply. Selected high-risk paths were independently checked below.

## Roadmap and user-facing capability

| Phase / issue | State | Available on main | Outstanding delivery gate |
|---|---|---|---|
| 0 · #1 | Accepted | Tauri desktop, keyboard inspector, Canvas2D fallback and wgpu benchmark | Measure the real saved-world graph under load. The earlier physical RTX 2060 run recorded a 16.70 ms median frame interval on a synthetic 2,000-node, 3,910-edge scene. |
| 1 · #2 | Accepted | SQLite world isolation, stable IDs, transactional revisions, historical corrections, branch forks, sourced claims and calendar primitives | Complete author-facing branch/calendar management and restore workflows. |
| 2 · #3 | Accepted | Author a directed relationship, change trust and tags through a sourced event, inspect the actor and compare before/after history | Richer organizations, conflict histories and knowledge models belong to later phases. |
| 3 · #4 | Active | Indexed historical labels and claims, aliases, relationship paths, participant roles, typed compound filters, saved searches, reviewed mention links, backlinks, stable paging and cooperative cancellation | Integrate the four drafts. Prove the complete native workflow, large-world behavior and permission-filtered query reuse across studios. Rich discovery including image references is outstanding. |
| 4 · #5 | Library foundation | Layout validation, reparenting and audience-based subtree filtering | Save, edit and reopen a Canvas entry, bind live data, then publish an authorized Display page. |
| 5 · #6 | Library foundation | Dated parentage categories and cycle checks on supplied records | Persist and edit kinship, marriages, households, dynasties and organization membership. |
| 6 · #7 | Partial | Phase 2 event/action/effect authoring plus functions for supplied causal links and conflict positions | Persist competing positions and event causality, then explore them in the Conflict Atlas. |
| 7 · #8 | Partial | Scoped claim visibility, a public-account preview and historical branches | Enforce publication grants and revocation across page content, links, search and export. |
| 8 · #9 | Experimental | Deterministic fictional segment-contributor example | Durable population history, relative matching, inheritance invalidation and analysis tools. |
| 9 · #10 | Partial | Developer launcher, database backup primitive, independent archive inspection and platform CI | Portable assets and layouts, recovery round trip, packaged offline install and measured 50k-world performance. NPC integration is optional. |

Sources: [roadmap](../Design-Docs/10-Delivery-Roadmap.md), [acceptance contracts](../Design-Docs/09-Acceptance-and-Verification.md), [store modules](../crates/atlas-store/src), [native workspace](../apps/desktop/src-tauri/src/workspace.rs), [physical renderer record](../Design-Docs/Phase-0-Verification.md), [roadmap tickets](https://github.com/sdhaight1-hash/ConWorld-Tools/issues?q=is%3Aissue).

## Changes awaiting integration

| Draft PR | Head | Base | Addition |
|---|---|---|---|
| [#41](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/41) | 3f8e82ac | main | Versioned typed saved-query defaults and overrides, schema 19 |
| [#42](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/42) | 08f90ef8 | #41 | Focused claim discovery and revision invalidation, schema 20 |
| [#44](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/44) | 9795ec7f | #42 | Indexed mention-name candidates, schema 21 |
| [#45](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/45) | 2741a156 | #44 | Dependency-driven reuse and refresh of discovery previews |

Main still opens schema 18. The saved-parameter and focused-claim screenshots on main came from development builds using synthetic IPC. GitHub returned mergeable=false for draft #41 with an older base SHA in its metadata. A fresh local Git three-way merge simulation against 4ef30e7 found no textual conflicts for #41 or for the top of the stack. Merge readiness still requires checking the resulting tree and CI, including retention of the audit and images added by #46.

## Findings that change development priorities

| Priority | Finding and user impact | Verification gate |
|---|---|---|
| Release blocker for readers | Author search explicitly requires author scope. The layout compiler can prune hidden descendants, but there is no persisted grant proving an entry and its fields may be published. A scoped public rumor preview is the extent of the demonstrated reader workflow. | Publish one character with one visible claim and one private claim. Verify that links, counts, nested bindings, search and export reveal only authorized data, including after revocation. [Query](../crates/atlas-store/src/query.rs) · [Layout](../crates/atlas-store/src/layout.rs) |
| Phase 3 correctness | Review resolves aliases through a world-wide list capped at 2,000 before narrowing to the target. An indexed discovery result on the staged build can therefore be discoverable yet impossible to approve. Delta refresh also resets comparison work across affected claims. | Approve a valid match among more than 2,000 unrelated aliases. Compare cold and incremental results and documented budget behavior across several changed claims, including cancellation. [Store](../crates/atlas-store/src/store.rs) · [Staged discovery](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/2741a15616317e20484340c845271b6167ea043f/crates/atlas-store/src/discovery.rs) |
| Authoring consistency | Person and relationship creation reread the current revision. Other edits accept the revision the author actually saw. A stale creation can be recorded against a newer world state. | Run two clients against one world. Require a documented conflict or explicit rebase on stale creation and prove retry behavior. [Native workspace](../apps/desktop/src-tauri/src/workspace.rs) |
| Scale and responsiveness | Above the 2,000-object scene budget, the workspace falls back to search-only. Paging reruns the bounded query with a 10,000-candidate ceiling. An indexed hit does not guarantee that the author can edit its neighborhood. | Use 1k/10k/50k fixtures to time first and later pages, cold and warm searches, cancellation and opening one editable neighborhood. Record fixture, hardware, latency, memory and truncation. [Workspace](../apps/desktop/src-tauri/src/workspace.rs) · [Paging design](../Design-Docs/ADRs/phase-3-query-pages.md) |
| Native integration evidence | Browser tests drive the real frontend with simulated native replies. The Windows smoke exercises the shell and renderer. Neither proves that a Phase 3 author can save, close, reopen and review a claim through the native UI. | Record the full native SQLite-backed workflow with a pinned revision, a cancellation and an approved backlink. Archive the capture with its fixture and commit. [Media index](../Development-Screenshots/Dev-Screenshots.md) |
| Documentation and CI evidence | The README still names pagination among future items. The CI artifact glob also matches the already tracked pagination PNG. An upload can find a file even when a run did not generate a new one. | Link a single dated main/PR status table. Stage only the current run's output for upload and verify its identity. [README](../README.md) · [Workflow](../.github/workflows/desktop.yml) |

## Verification record

| Check | Evidence obtained in this audit |
|---|---|
| Frontend from a clean checkout | Locked npm install passed. The production build passed all four launcher/configuration checks, TypeScript checking and Vite bundling. The live Vite smoke served the expected frontend at 127.0.0.1:1420. |
| Database DDL | Applied all 17 main SQL files in order to an empty SQLite 3.53.1 database with foreign keys on. Integrity check returned ok and foreign-key check returned no rows. Version 15 is a format compatibility bump. This check covers empty-schema DDL only. Populated migrations, Rust backfills, transaction behavior and reopen are separate tests. |
| Browser and native execution | The pinned Chromium download returned HTTP 403. System Chromium was present but the host denied its startup socket before page creation. No fresh UI capture or local Playwright pass resulted. Cargo was unavailable here. The 60 main store integration test functions were inventoried, without a local Rust execution claim. |
| GitHub CI | Desktop workflow succeeded on the #46 PR head. Desktop and persistence workflows succeeded on the staged #45 head, including Windows and Ubuntu jobs. The PR-filtered workflow lookup returned no run for main's merge SHA. Evidence is attributed to the tested commits. [#46 desktop](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44942048842) · [#45 desktop](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940410058) · [#45 store](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940409953) |

## Development sequence

1. Reconcile the Phase 3 stack with current main, reproduce the alias-review and refresh-budget cases, and run CI on the integrated tree.
2. Exercise the native author workflow from world creation through reopen and reviewed backlink. Pair that with measurements of the search-only transition and large-world paging.
3. Implement the smallest reader-authorized workflow that spans all three studios. An author should be able to create one person in Node, save a Canvas page containing a bound historical query, and open that page in Display with private material omitted. Use this as the next product milestone.
4. Continue persisted genealogy and organizations once that workflow can display real data. Use the 36 acceptance scenarios and existing phase issues as release gates.

This report is a repository-wide development assessment. It provides source, workflow and runtime evidence at the stated baselines. Formal security assurance, hardware retesting and release certification remain separate activities.
