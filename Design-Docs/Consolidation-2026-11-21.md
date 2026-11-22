# Consolidation and integration record — 2026-11-21 UTC

Status: merged baseline, scoped CI-verified implementation. All ten development PRs merged into main; no open PRs immediately following consolidation. Phase 3–9 roadmap issues remain open. Main consolidation commit [a7a943f](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/a7a943fb0d9a090602298c1b30acd61ec04e3eff) preserves all original commit ancestry. The first five PRs were merged individually; the remaining stack was merged together through #52 after resolving the only conflict, concurrent append-only screenshot index entries. The independent main audit commit 4ef30e7 and its image were preserved. The GitHub compare from verified application tip d2aafb7d to the merged baseline shows only four changed paths: the design README, audit report, discovery PNG, and screenshot index. All production code, tests, migrations, dependency manifests and workflows are identical to the tested tip. No force push, squash, rebase or destructive branch cleanup.

## Verified development slices

Both CI workflows passed at every listed exact original head on Linux and Windows. This means formatter, strict Clippy, locked Rust tests/builds, frontend build and live Vite smoke, Linux browser regression and Windows WebView2 platform smoke; not complete roadmap acceptance or a real GPU benchmark.

| PR | Slice | Schema | Checked head | Persistence | Desktop |
|---|---|---:|---|---|---|
| [#41](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/41) | Typed parameters | 19 | `3f8e82ac` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940145779) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940145843) |
| [#42](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/42) | Focused discovery and revision deltas | 20 | `08f90ef8` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940322223) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940322245) |
| [#44](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/44) | Indexed historical names | 21 | `9795ec7f` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940354929) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940355016) |
| [#45](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/45) | Complete preview refresh | 21 | `2741a156` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940409953) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940479406) |
| [#47](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/47) | Exact review span and bounded refresh | 21 | `8d0af9dc` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44943922155) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44943922042) |
| [#48](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/48) | Persistent pages, focused history and recovery | 22 | `e36ca50b` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44944908463) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44944908294) |
| [#49](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/49) | Dated parentage and fictional comparison | 23 | `e48368f2` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44947503686) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44947503711) |
| [#50](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/50) | Indexed public text | 24 | `23788aca` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44959192584) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44959192573) |
| [#51](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/51) | Private saved-query layout editor | 25 | `7234d671` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44959899901) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44959899805) |
| [#52](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/52) | Authored links and indexed backlinks | 26 | `d2aafb7d` | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44960037600) | [Pass](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44960037608) |

## Integration boundaries

- One SQLite world store with append-only history, world-scoped identities and explicit main/fork constraints. Schema 26; migrations 19–24 and 26 present. Schema 25 (like 15) intentionally has no DDL. Fresh and older database paths run ordered migration checks; archive compatibility is guarded by application ID and schema version. Back up before upgrade; older binaries reject newer schema.
- Author Node search and Canvas private bindings share typed QuerySpec, exact saved definition revisions, current historical data view, cancellation and stable IDs. Public text search compiles authorized released layouts and uses a separate derived FTS candidate index; revocation changes it atomically. Author-only queries do not execute in reader projection.
- Explicit page references index source/element/target revisions and do not create semantic relationships. The destination must be released and in scope for reader links; source and target share the read snapshot. Author backlinks remain distinct from semantic and reviewed mention references. The core backlink API is implemented; authoring/navigation UI and link-caption FTS remain open.
- Dated accepted kinship and synthetic immediate-contributor comparison read the selected day and author revision; causal impact supports focused canonical reads. Native page and family captures are committed. Verified recovery copies, validates and migrates to a fresh destination; full media-inclusive export remains unfinished.
- Static native IPC audit: all 35 command handlers, generated manifest commands and permission entries match exactly. No added capability or independent database was introduced in the merge.
- Combined fixtures in [roadmap_foundations.rs](../crates/atlas-store/tests/roadmap_foundations.rs) cover query context, hidden subtrees, reparenting, scoped evidence, causal impact, persistence, recovery, parentage, published search and backlinks. CI includes the character editor mock-native test; this does not substitute for full native E2E reproduction.

## Remaining roadmap gates

| Issue | Remaining priorities |
|---|---|
| [#4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4) | Link editor and backlink inspector, draft ghost proposals/review, hotspot discovery, safe character/reader queries, full scale and AT-21–25/35 |
| [#5](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/5) | General Canvas layout/templates, Grid/Panel, reader-safe bound timeline/table, image assets/hotspots, standalone Display navigation |
| [#6](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/6) | Households, marriages, organizations, memberships, succession, family-tree UI, fork-aware kinship |
| [#7](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/7) | Persistent causal links, conflict editor, faction histories |
| [#8](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/8) | Character knowledge, alternate branch editing, stronger field/asset policy and adversarial privacy |
| [#9](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/9) | Founder tracing, multigenerational populations, reproducible match reports |
| [#10](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/10) | Full portable archives, recovery UI, 50k target-hardware/renderer performance and release acceptance |

Do not close these roadmap issues based on this consolidation. Next implement end-to-end hyperlink authoring, scoped reverse-reference inspection and reader navigation using the existing ObjectLink/index APIs, then continue composable Canvas and reader-safe components.

## Fresh main validation

Consolidation-triggered CI: [persistence](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44962400367) and [desktop](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44962400350) on a7a943f were queued when inspected. The exact tested application content is unchanged; fresh merge-commit results are separately tracked. Visual evidence: see [media index](../Development-Screenshots/Dev-Screenshots.md). No new native runtime capture claimed in this consolidation.

## Follow-up integration

- [#53](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/53) merged into main as `e3e49edd`. Exact head `1d74f649` passed [persistence](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44963245194) and [desktop](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44963245219) on Windows and Linux. End-to-end editor links and reader navigation are now available in main; the [reader capture](../Development-Screenshots/Dev-Screenshots.md#additional-ci-visual-evidence-artifact-archive) is browser mock-native evidence, not native runtime proof.
- [#54](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/54) schema-27 caption search passed [persistence](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44963708953) and [desktop](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44963708958) at standalone head `27fe66d0`. Integrated head `256d36ea` preserves the target registry validation from #53 and its failure regression; [combined persistence](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44967685388) and [desktop](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44967685373) checks are pending at this record. Do not call schema 27 merged yet.
- [#55](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/55) head `83cbdfb1` extends the existing editor to event/relationship nodes and author query results, with a source-entry mock-native regression and capture configuration. [Desktop verification](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44967727595) pending. No schema change. General compositional layout, ghost review, and reader-safe bound components remain open.
