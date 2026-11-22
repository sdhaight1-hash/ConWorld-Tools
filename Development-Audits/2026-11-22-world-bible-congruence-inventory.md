# World Bible congruence audit — frozen repository inventory

Date: 2026-11-22 UTC. Inventory before audit publication; main history enumerated through 235 commits (100+100+35; following page empty). All 73 branch refs, 60 PRs and 20 non-PR issues were enumerated; PR metadata and ancestry are distinct from patch parity. No branch deletion, issue closure, merge or workflow pass implied. Historical squash/rebase merges can have differing commit identities. See [audit](2026-11-22-world-bible-congruence-audit.md).

## Branch register

| Branch | Pinned head | History / recommendation |
| --- | --- | --- |
| `audit/bug-hunt-performance-2026-11-22` | [65771fd6c](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/65771fd6c5a8bb439a99c2d2f3556bb2837af99d) | Preserve BH-01–06 report via recovery branch; no independent code |
| `audit/state-of-development-2026-11-21` | [4fa8c1b97](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/4fa8c1b9717c192f8bf4a1a982f2ee02e5ef4503) | Historical snapshot; retain, not current authority |
| `audit/state-of-development-2026-11-22` | [4b274360f](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/4b274360f38186ce02b41a26bd8fe12b38b0ec6f) | Preserve dated source audit; update disposition, not historical facts |
| `fix/desktop-startup-assets` | [442765397](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/4427653970f5fb132e6d99b3943519f9f68cd76b) | #13 closed unmerged; candidate already has custom-protocol default, loopback and build wiring; compare residual patch before archive |
| `fix/dev-server-smoke` | [615a5b8a7](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/615a5b8a70e47d05dbcb9ea13c119dc3ef84a7f9) | PR #14 merged; head not literal main ancestor; retain until patch/cleanup review |
| `fix/windows-smoke-exit-code` | [6a8f41b6c](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/6a8f41b6cf18f938277b0dea913d1cb39047319e) | Exact head in main history; retain until cleanup review |
| `main` | [5777f0fae](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/5777f0faea7dd5c4aa93908d35d3cfd7cfea773f) | Released development baseline; schema 36 |
| `research/cartography-label-index` | [8e6bc390d](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/8e6bc390d3d7c473785fea4c683adcc6600bc79b) | Unique supplemental #66/#68 research, carry report/script/results and index |
| `review/ui-ux-audit` | [e2947c3e4](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e2947c3e4f2c4e3c12b7a9c121423daff06c202c) | Design/prototype/research + #77/#78 documents; integrate selectively with latest verified code |
| `work/ae01-integration` | [680c52629](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/680c52629e72b9d166ff902af25e45ab8a11ec6f) | #73 ancestor of #80, 46 behind; superseded integration candidate |
| `work/archive-verified-captures` | [d2f007890](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/d2f0078909727f6b459230e047bf4809694b27e2) | Exact head in main history; retain until cleanup review |
| `work/branch-metadata-and-succession` | [24e09b307](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/24e09b307307bde201268615cb786985a649ab1b) | Exact head in main history; retain until cleanup review |
| `work/canvas-media-portable-archives` | [776198393](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/776198393e13b2227881acb154bab1efd9f0eeb1) | Exact head in main history; retain until cleanup review |
| `work/cross-phase-custom-templates` | [1f8c8c9f0](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/1f8c8c9f047ae31751b90c7c9fc1dfb2c63659d6) | Exact head in main history; retain until cleanup review |
| `work/cross-phase-entry-history` | [e36ca50b8](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e36ca50b806d4830af622a8ed52b4b59d3e5b602) | Exact head in main history; retain until cleanup review |
| `work/cross-phase-institution-discovery-publication` | [43629ca71](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/43629ca711b205fdb2981a993a769d157e64e9c9) | Exact head in main history; retain until cleanup review |
| `work/cross-phase-lineage-discovery-genetics` | [82052bb1f](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/82052bb1f6e8d7f3e9f4779089276625381c54ca) | Exact head in main history; retain until cleanup review |
| `work/dated-family-history` | [e48368f25](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e48368f25bf75162c8647ffefac5599e71038567) | Exact head in main history; retain until cleanup review |
| `work/integration-world-fixture` | [8a451acad](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/8a451acadc19f96ad4f0f8877ef797678d934a5d) | Verified original fixture; prefer expanded workflow branch after exact validation |
| `work/integration-world-workflows` | [91b1e182f](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/91b1e182fb96824908fe80f417e08ebd27b59e4b) | Expanded Alder Basin fixture, staged skip-ci, integrate and verify |
| `work/linked-encyclopedia` | [1d74f6492](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/1d74f6492ab6ab8b15940fc5c4328a20a6de05be) | Exact head in main history; retain until cleanup review |
| `work/ontology-boolean-pagination` | [6bf9739d9](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/6bf9739d944754c54ced22d1d38f483820f603ae) | Ancestor of #80, 16 behind |
| `work/ontology-classification-pagination` | [c8e765c30](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/c8e765c3094159efe4b7caf7a77c10783f309217) | Ancestor of #80, 17 behind |
| `work/ontology-consolidation` | [30b6710a5](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/30b6710a583550df68e8ba88cb26c6ad01b26e56) | #75 frozen base of #80, 6 behind; integration base, not independently final |
| `work/ontology-final-evidence` | [1aa9b0b6f](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/1aa9b0b6fa6a4f189e5ee0bb5d88ca4a9129aa0d) | #80 primary code candidate; complete verification |
| `work/ontology-spatial-reconciliation` | [b6d4cfaa6](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/b6d4cfaa6993ef6b9696a1e89af4775adcc31409) | #74 closed unmerged, ancestor of #80, 23 behind |
| `work/ontology-verification-fixes` | [27daf6c2e](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/27daf6c2e3876d9f3f9becb378a1c0aeca7af2ce) | Ancestor of #80, 8 behind |
| `work/phase-0-acceptance` | [174b9431d](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/174b9431d69b526fbacc57e8de205f8814765e2b) | Exact head in main history; retain until cleanup review |
| `work/phase-0-platform` | [dfe42f4b3](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/dfe42f4b3b7646e9f4aea3de57daa41aa969e9aa) | Exact head in main history; retain until cleanup review |
| `work/phase-1-persistence` | [208244c21](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/208244c211337dbf8e27cc3e2ad357caf7581b9d) | Exact head in main history; retain until cleanup review |
| `work/phase-2-actors-inspector` | [ca21b6683](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/ca21b66832a39e972fc645edf9e279c8739280cf) | Exact head in main history; retain until cleanup review |
| `work/phase-2-relationships-agency` | [254ee89dd](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/254ee89dd50ad721261586d5d74fdc5211cb1d6b) | PR #15 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-authored-entry-links` | [ab71fe330](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/ab71fe3305042833260afa1db9d9110153c23c5a) | Exact head in main history; retain until cleanup review |
| `work/phase-3-authorized-caption-search` | [256d36ea7](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/256d36ea70734c5aef276f48bbe7632460a721da) | Exact head in main history; retain until cleanup review |
| `work/phase-3-authorized-entry-search` | [23788aca6](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/23788aca64cc6d79ff9788a9977349f590af1d27) | Exact head in main history; retain until cleanup review |
| `work/phase-3-claim-fts` | [c02fe315d](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/c02fe315d998bec80392cd73b744c96e1112a303) | PR #26 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-composite-query` | [48437f9fd](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/48437f9fd2dfd7bb0009fe03005280464dbd35f4) | PR #28 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-discovery-refresh` | [2741a1561](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/2741a15616317e20484340c845271b6167ea043f) | Exact head in main history; retain until cleanup review |
| `work/phase-3-focused-evidence` | [5da6776e0](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/5da6776e001f572903015405ebd6d698cc65127d) | Compare exact PNG and index against existing main archive; preserve any unique evidence |
| `work/phase-3-fts5-historical-labels` | [f93e61640](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/f93e6164052bfac62ad6a2318fcd69fd0a44760a) | PR #22 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-incremental-discovery` | [08f90ef88](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/08f90ef8829b60bcb1f13eb2a0467c1b3a9653de) | Exact head in main history; retain until cleanup review |
| `work/phase-3-indexed-aliases` | [303fb26af](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/303fb26af195a7f53a4ca3c7aa0ce495937ad355) | Exact head in main history; retain until cleanup review |
| `work/phase-3-indexed-backlinks` | [e8583742b](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e8583742bd149255796be58e35b9b9d51ea5c583) | PR #36 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-indexed-composite` | [d4422d85e](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/d4422d85e61b01aa5f574e2fdbc85e96ffcb1073) | Exact head in main history; retain until cleanup review |
| `work/phase-3-indexed-participants` | [dc59db3e0](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/dc59db3e000da05c4b1d7bd531f2bd596acd13b9) | PR #30 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-indexed-paths` | [07a4a067e](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/07a4a067e46455bbe6bee5fcea51a9c373a86e49) | Exact head in main history; retain until cleanup review |
| `work/phase-3-lineage-candidates` | [ee08ba008](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/ee08ba008317c96ba2109ee4106d25e342cbbd53) | Exact head in main history; retain until cleanup review |
| `work/phase-3-mention-candidates` | [d2d6c498d](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/d2d6c498dd3130049f70edca4302f012f115f2c5) | PR #24 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-mention-name-index` | [9795ec7f0](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/9795ec7f036bd9152a66bac51c74d1e5abb4ddc5) | Exact head in main history; retain until cleanup review |
| `work/phase-3-pagination-evidence` | [241f74349](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/241f74349925b685468199a38bfe1254c39f6165) | PR #39 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-parameter-evidence` | [b7302d8af](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/b7302d8af9ec66ea680188738b478ed82865cdc8) | PR #43 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-participant-events` | [1de8b657a](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/1de8b657a7352280d3f97268597526c7be7995ee) | PR #27 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-person-aliases` | [b95f26e14](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/b95f26e14b946b0355be7afdf75fca2a2356cc4d) | PR #31 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-query-control` | [3ca816230](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/3ca816230872c8339678f2d923e1436add22f29a) | Exact head in main history; retain until cleanup review |
| `work/phase-3-query-pages` | [9ed185f30](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/9ed185f30f82de9cf3a7d4dce289a2c58f7487c4) | Exact head in main history; retain until cleanup review |
| `work/phase-3-review-and-refresh-bounds` | [8d0af9dcd](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/8d0af9dcd32b927babfc325372f5ea1ca89a7a1a) | Exact head in main history; retain until cleanup review |
| `work/phase-3-reviewed-mentions` | [e8509f469](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e8509f46999a9dcd1b4bba781c564a05748ee0a1) | PR #25 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-saved-composite` | [c68e2fae1](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/c68e2fae17d2b1d9db0eafdcb42cd88f60dcd913) | PR #29 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-saved-label-queries` | [2cbc8a7ea](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/2cbc8a7eaee2b3fae96d345f994ffb5a8ca71ef9) | PR #23 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-saved-layout-bindings` | [7234d6713](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/7234d67138b0ee1392a8b95af8035acaa8aaab22) | Exact head in main history; retain until cleanup review |
| `work/phase-3-saved-parameters` | [3f8e82acb](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/3f8e82acb5d5647cfe02de191825598c4af99f5e) | Exact head in main history; retain until cleanup review |
| `work/phase-3-search` | [5c27b1989](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/5c27b198941f5cccceadc9781759d8ca7599a32c) | PR #20 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-search-context` | [f117ed07b](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/f117ed07bc762025539731762459840825a68674) | PR #33 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-3-traversal` | [8fca4acee](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/8fca4acee95d2d27c5c485cd02e44a20217eeab3) | PR #21 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/phase-4-canvas-tree` | [4ce94821e](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/4ce94821e90744b22815e023c8ceb337f85e71f5) | Exact head in main history; retain until cleanup review |
| `work/phase-4-general-entry-authoring` | [eb55a8899](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/eb55a889962a21b77e18abcb705d6706194e26e1) | Exact head in main history; retain until cleanup review |
| `work/phase-4-layout-containers` | [bfd0ee002](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/bfd0ee002b92a1fc8f75adb9614445ab6a5145d0) | Exact head in main history; retain until cleanup review |
| `work/phase-4-template-query-studio` | [cd239c153](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/cd239c153604f0ad96c0370c5144d8693a41ca95) | Exact head in main history; retain until cleanup review |
| `work/recovery-network-hardening` | [3ecd5bfab](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/3ecd5bfab683ee1b7694788d6985d7c13bd243fd) | Verified unique sibling changes BH-01–04; adapt/merge and test #80 |
| `work/repository-audit-2026-11-21` | [30455a278](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/30455a2781f00c181eaf33655b304323fde719fd) | PR #46 merged; head not literal main ancestor; retain until patch/cleanup review |
| `work/roadmap-code-foundations` | [a9e53b306](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/a9e53b3068033f5125b3ac360566b0e42b8c7a03) | Exact head in main history; retain until cleanup review |
| `work/spatial-foundation` | [31038560d](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/31038560d0e49a93f73aee42195bc8d654e2a9ed) | #67 competing schema lineage; no independent merge; audit feature parity vs #80 |
| `work/workspace-context-safety` | [dac1ab2ab](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/dac1ab2ab5bdf9a279d73793132560d36dd21a5e) | #79 verified safeguards/picker; integrate with #80 and review branch before shell |

## All formal issues (non-PR)

Checklist counts are literal Markdown checkboxes, not percent complete. Historical status preserved.

| Issue | Title | State | Checked | Unchecked |
| --- | --- | --- | ---: | ---: |
| [#1](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1) | Roadmap Phase 0 — Architecture Baseline & Platform Feasibility | closed | 11 | 5 |
| [#2](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/2) | Roadmap Phase 1 — World Persistence, Temporal Model & Provenance | closed | 4 | 10 |
| [#3](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3) | Roadmap Phase 2 — Relationships, Tags & Event-Driven History Vertical Slice | closed | 30 | 0 |
| [#4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4) | Roadmap Phase 3 — Universal Search, Historical Queries, Backlinks & Discovery | open | 54 | 30 |
| [#5](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/5) | Roadmap Phase 4 — Node, Composable Canvas & Authorized Display | open | 0 | 14 |
| [#6](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/6) | Roadmap Phase 5 — Genealogy, Dynasties, Households & Organization Networks | open | 0 | 16 |
| [#7](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/7) | Roadmap Phase 6 — Event Studio, Actor Agency, Conflict Atlas & Causal Impact | open | 0 | 16 |
| [#8](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/8) | Roadmap Phase 7 — Historical Evidence, Perspectives, Branches & Privacy Hardening | open | 0 | 16 |
| [#9](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/9) | Roadmap Phase 8 — Synthetic Genetics, DNA Relatives & Population History | open | 0 | 16 |
| [#10](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/10) | Roadmap Phase 9 — Release Hardening, Portable Worlds, Performance & Optional NPC Integration | open | 0 | 16 |
| [#65](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/65) | Research: NestedLocations — Hierarchical, Spatial and Historical Place Modeling | closed | 4 | 0 |
| [#66](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/66) | Research: Procedural Vector Cartography from NestedLocations Data | closed | 5 | 0 |
| [#68](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/68) | New Feature: NestedLocations and Procedural Vector Cartography | open | 5 | 9 |
| [#69](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/69) | Research: Maltego-Inspired Transforms for Historical World Exploration | closed | 6 | 0 |
| [#70](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70) | New Feature: Composable Worldbuilding Transforms and Exploration Workflows | open | 4 | 12 |
| [#71](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/71) | Research: Universal Worldbuilding Ontology, Entity Classification and Domain Coverage | closed | 7 | 0 |
| [#72](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72) | Architectural Evolution: Universal Worldbuilding Ontology and Governed Semantic Extensibility | open | 3 | 11 |
| [#76](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/76) | Research: Comparative Worldbuilding Workspaces, Encyclopedia Systems and Authoring UX | open | 0 | 13 |
| [#77](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/77) | Architectural Evolution: Independent Editorial Draft and Publication Lifecycle | open | 0 | 5 |
| [#78](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/78) | New Feature: Configurable World Management Workspaces and Dashboards | open | 0 | 5 |

## All pull requests

| PR | Title | Head | Base | Disposition at audit |
| --- | --- | --- | --- | --- |
| [#11](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/11) | Build the shared Rust/SQLite foundation for Phase One | [208244c21](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/208244c211337dbf8e27cc3e2ad357caf7581b9d) | `main` | Merged 2026-11-21 |
| [#12](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/12) | Phase Zero: Tauri desktop baseline and accessible graph fallback | [dfe42f4b3](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/dfe42f4b3b7646e9f4aea3de57daa41aa969e9aa) | `main` | Merged 2026-11-20 |
| [#13](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/13) | Fix desktop localhost refusal in standalone Cargo builds | [442765397](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/4427653970f5fb132e6d99b3943519f9f68cd76b) | `main` | Closed unmerged; #13 closed unmerged; candidate already has custom-protocol default, loopback and build wiring; compare residual patch before archive |
| [#14](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/14) | test(desktop): verify live Vite server starts and serves frontend | [615a5b8a7](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/615a5b8a70e47d05dbcb9ea13c119dc3ef84a7f9) | `main` | Merged 2026-11-21 |
| [#15](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/15) | Phase 2: versioned relationships, event participation and actor actions | [254ee89dd](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/254ee89dd50ad721261586d5d74fdc5211cb1d6b) | `main` | Merged 2026-11-21 |
| [#16](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/16) | Complete Phase 2 relationship and event history | [ca21b6683](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/ca21b66832a39e972fc645edf9e279c8739280cf) | `main` | Merged 2026-11-21 |
| [#17](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/17) | Add executable foundations for roadmap phases 3–9 | [a9e53b306](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/a9e53b3068033f5125b3ac360566b0e42b8c7a03) | `main` | Merged 2026-11-21 |
| [#18](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/18) | Automate native Phase 0 renderer acceptance | [174b9431d](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/174b9431d69b526fbacc57e8de205f8814765e2b) | `main` | Merged 2026-11-21 |
| [#19](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/19) | Preserve Windows smoke exit codes and record RTX 2060 verification | [6a8f41b6c](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/6a8f41b6cf18f938277b0dea913d1cb39047319e) | `main` | Merged 2026-11-21 |
| [#20](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/20) | Phase 3: add historical search to the persistent Node workspace | [5c27b1989](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/5c27b198941f5cccceadc9781759d8ca7599a32c) | `main` | Merged 2026-11-21 |
| [#21](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/21) | Phase 3: bounded historical relationship traversal in Node workspace | [8fca4acee](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/8fca4acee95d2d27c5c485cd02e44a20217eeab3) | `main` | Merged 2026-11-21 |
| [#22](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/22) | Phase 3: version-aware FTS5 label search, migration and rebuild | [f93e61640](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/f93e6164052bfac62ad6a2318fcd69fd0a44760a) | `main` | Merged 2026-11-21 |
| [#23](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/23) | Phase 3: durable, revisioned saved label searches in the author workspace | [2cbc8a7ea](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/2cbc8a7eaee2b3fae96d345f994ffb5a8ca71ef9) | `main` | Merged 2026-11-21 |
| [#24](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/24) | Phase 3: read-only historical mention candidates with disambiguation | [d2d6c498d](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/d2d6c498dd3130049f70edca4302f012f115f2c5) | `main` | Merged 2026-11-21 |
| [#25](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/25) | Phase 3: versioned author review for mention links and backlinks | [e8509f469](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e8509f46999a9dcd1b4bba781c564a05748ee0a1) | `main` | Merged 2026-11-21 |
| [#26](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/26) | Phase 3: indexed historical sourced claim search and saved replay | [c02fe315d](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/c02fe315d998bec80392cd73b744c96e1112a303) | `main` | Merged 2026-11-21 |
| [#27](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/27) | Phase 3: query recorded historical events by person and role | [1de8b657a](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/1de8b657a7352280d3f97268597526c7be7995ee) | `main` | Merged 2026-11-21 |
| [#28](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/28) | Phase 3: bounded typed historical AND/OR/NOT predicate queries | [48437f9fd](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/48437f9fd2dfd7bb0009fe03005280464dbd35f4) | `main` | Merged 2026-11-21 |
| [#29](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/29) | Phase 3: durable versioned typed composite queries and replay | [c68e2fae1](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/c68e2fae17d2b1d9db0eafdcb42cd88f60dcd913) | `main` | Merged 2026-11-21 |
| [#30](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/30) | Phase 3: indexed historical participation and event-role queries | [dc59db3e0](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/dc59db3e000da05c4b1d7bd531f2bd596acd13b9) | `main` | Merged 2026-11-21 |
| [#31](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/31) | Phase 3: versioned person aliases and historical mention disambiguation | [b95f26e14](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/b95f26e14b946b0355be7afdf75fca2a2356cc4d) | `main` | Merged 2026-11-21 |
| [#32](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/32) | Phase 3: bounded indexed full-text compound predicates | [d4422d85e](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/d4422d85e61b01aa5f574e2fdbc85e96ffcb1073) | `main` | Merged 2026-11-21 |
| [#33](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/33) | Phase 3: search-only workspace on scene budget and pinned historical reads | [f117ed07b](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/f117ed07bc762025539731762459840825a68674) | `main` | Merged 2026-11-21 |
| [#34](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/34) | Phase 3: indexed historical relationship traversal | [07a4a067e](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/07a4a067e46455bbe6bee5fcea51a9c373a86e49) | `main` | Merged 2026-11-21 |
| [#35](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/35) | Phase 3: indexed historical exact alias search | [303fb26af](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/303fb26af195a7f53a4ca3c7aa0ce495937ad355) | `main` | Merged 2026-11-21 |
| [#36](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/36) | Phase 3: indexed typed backlinks and reviewed references | [e8583742b](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e8583742bd149255796be58e35b9b9d51ea5c583) | `main` | Merged 2026-11-21 |
| [#37](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/37) | Phase 3: stable historical query pagination core | [9ed185f30](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/9ed185f30f82de9cf3a7d4dce289a2c58f7487c4) | `main` | Merged 2026-11-21 |
| [#38](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/38) | Phase 3: cooperative query cancellation and deadlines | [3ca816230](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/3ca816230872c8339678f2d923e1436add22f29a) | `main` | Merged 2026-11-21 |
| [#39](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/39) | Phase 3: archive verified historical pagination screenshot | [241f74349](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/241f74349925b685468199a38bfe1254c39f6165) | `main` | Merged 2026-11-21 |
| [#40](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/40) | Phase 3: historical lineage candidate bounds | [ee08ba008](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/ee08ba008317c96ba2109ee4106d25e342cbbd53) | `main` | Merged 2026-11-21 |
| [#41](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/41) | Phase 3: typed saved query parameters | [3f8e82acb](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/3f8e82acb5d5647cfe02de191825598c4af99f5e) | `main` | Merged 2026-11-21 |
| [#42](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/42) | Phase 3: focused discovery and revision-delta invalidation | [08f90ef88](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/08f90ef8829b60bcb1f13eb2a0467c1b3a9653de) | `main` | Merged 2026-11-21 |
| [#43](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/43) | Phase 3: archive verified saved parameter capture | [b7302d8af](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/b7302d8af9ec66ea680188738b478ed82865cdc8) | `main` | Merged 2026-11-21 |
| [#44](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/44) | Phase 3: materialized historical mention name indexes | [9795ec7f0](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/9795ec7f036bd9152a66bac51c74d1e5abb4ddc5) | `main` | Merged 2026-11-21 |
| [#45](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/45) | Phase 3: dependency-driven discovery preview refresh | [2741a1561](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/2741a15616317e20484340c845271b6167ea043f) | `main` | Merged 2026-11-21 |
| [#46](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/46) | Repository-wide audit and verified Phase 3 screenshot archive | [30455a278](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/30455a2781f00c181eaf33655b304323fde719fd) | `main` | Merged 2026-11-21 |
| [#47](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/47) | Phase 3: validate indexed alias reviews and bound incremental discovery work | [8d0af9dcd](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/8d0af9dcd32b927babfc325372f5ea1ca89a7a1a) | `main` | Merged 2026-11-21 |
| [#48](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/48) | Phases 3–9: persistent character pages, focused family and event reads, and verified archive recovery | [e36ca50b8](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e36ca50b806d4830af622a8ed52b4b59d3e5b602) | `main` | Merged 2026-11-21 |
| [#49](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/49) | Phase 5: persist dated family links and compare fictional inheritance from recorded parents | [e48368f25](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e48368f25bf75162c8647ffefac5599e71038567) | `main` | Merged 2026-11-21 |
| [#50](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/50) | Phase 3: indexed authorized entry and world-wide published page search | [23788aca6](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/23788aca64cc6d79ff9788a9977349f590af1d27) | `main` | Merged 2026-11-21 |
| [#51](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/51) | Phase 3/4: version-pinned saved query layout bindings | [7234d6713](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/7234d67138b0ee1392a8b95af8035acaa8aaab22) | `main` | Merged 2026-11-21 |
| [#52](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/52) | Phase 3/4: authored entry links and historical authorized backlinks | [ab71fe330](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/ab71fe3305042833260afa1db9d9110153c23c5a) | `main` | Merged 2026-11-21 |
| [#53](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/53) | Phase 3/4: interconnected encyclopedia authoring and reader navigation | [1d74f6492](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/1d74f6492ab6ab8b15940fc5c4328a20a6de05be) | `main` | Merged 2026-11-22 |
| [#54](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/54) | Phase 3/4: destination-authorized hyperlink caption search | [256d36ea7](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/256d36ea70734c5aef276f48bbe7632460a721da) | `main` | Merged 2026-11-22 |
| [#55](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/55) | Phase 4: general entry authoring from event, relationship and search views | [eb55a8899](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/eb55a889962a21b77e18abcb705d6706194e26e1) | `main` | Merged 2026-11-22 |
| [#56](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/56) | Phase 4: persisted Grid and Panel containers with shared authorized projection | [bfd0ee002](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/bfd0ee002b92a1fc8f75adb9614445ab6a5145d0) | `main` | Merged 2026-11-22 |
| [#57](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/57) | Phase 4: structured Canvas tree editor and privacy-preserving component moves | [4ce94821e](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/4ce94821e90744b22815e023c8ceb337f85e71f5) | `main` | Merged 2026-11-22 |
| [#58](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/58) | Phase 4: versioned Canvas template studio, multiple saved bindings and live historical components | [cd239c153](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/cd239c153604f0ad96c0370c5144d8693a41ca95) | `main` | Merged 2026-11-22 |
| [#59](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/59) | Cross-phase: canonical institutions, sourced historical networks, draft discovery and author studio | [43629ca71](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/43629ca711b205fdb2981a993a769d157e64e9c9) | `main` | Merged 2026-11-22 |
| [#60](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/60) | Phase 4: persistent custom template library, version pins, drag/drop and rebinding | [1f8c8c9f0](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/1f8c8c9f047ae31751b90c7c9fc1dfb2c63659d6) | `main` | Merged 2026-11-22 |
| [#61](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/61) | Cross-phase: inherited historical timelines, ghost resolution and multigenerational founder ancestry | [82052bb1f](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/82052bb1f6e8d7f3e9f4779089276625381c54ca) | `main` | Merged 2026-11-22 |
| [#62](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/62) | Branch-aware genealogy, institutional history and succession foundations | [24e09b307](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/24e09b307307bde201268615cb786985a649ab1b) | `main` | Merged 2026-11-22 |
| [#63](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/63) | Canvas raster assets, authorized hotspots and media-inclusive portable archives | [776198393](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/776198393e13b2227881acb154bab1efd9f0eeb1) | `main` | Merged 2026-11-22 |
| [#64](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/64) | Preserve seven verified development PNG captures and SHA-256 manifest | [d2f007890](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/d2f0078909727f6b459230e047bf4809694b27e2) | `main` | Merged 2026-11-22 |
| [#67](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/67) | Spatial foundation: branch-aware Place identity and historical containment | [31038560d](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/31038560d0e49a93f73aee42195bc8d654e2a9ed) | `main` | Open draft; #67 competing schema lineage; no independent merge; audit feature parity vs #80 |
| [#73](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/73) | Advance #72 with native author-task baseline and shared search kind mapping | [680c52629](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/680c52629e72b9d166ff902af25e45ab8a11ec6f) | `main` | Open draft; #73 ancestor of #80, 46 behind; superseded integration candidate |
| [#74](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/74) | Held integration candidate: ontology, spatial registry and sourced site-area values (#72 #68) | [b6d4cfaa6](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/b6d4cfaa6993ef6b9696a1e89af4775adcc31409) | `work/ae01-integration` | Closed unmerged; #74 closed unmerged, ancestor of #80, 23 behind |
| [#75](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/75) | Consolidate verified ontology, spatial foundation and protected schema migration (#72) | [30b6710a5](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/30b6710a583550df68e8ba88cb26c6ad01b26e56) | `main` | Open draft; #75 frozen base of #80, 6 behind; integration base, not independently final |
| [#79](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/79) | Guard author creation revisions and complete object-kind selection (#4 #5 #8 #72) | [dac1ab2ab](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/dac1ab2ab5bdf9a279d73793132560d36dd21a5e) | `review/ui-ux-audit` | Open draft; #79 verified safeguards/picker; integrate with #80 and review branch before shell |
| [#80](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/80) | Complete ontology verification and preserve fact provenance (#72) | [1aa9b0b6f](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/1aa9b0b6fa6a4f189e5ee0bb5d88ca4a9129aa0d) | `work/ontology-consolidation` | Open draft; #80 primary code candidate; complete verification |

## Tracked file register, four principal candidate trees

Exact blob IDs permit future parity comparisons. M = main; O = ontology final; R = review; C = context safety. A dash means absent, not deleted intentionally. Full path inventory is not a claim of exhaustive line-by-line review. Original research, source and test files preserved. Other unique branches are recorded above and in the report.

| Path | M | O | R | C |
| --- | --- | --- | --- | --- |
| `.github/workflows/desktop.yml` | a87ccf4238fc | 3dafa805cfcf | bc414340ddee | 685c26901908 |
| `.github/workflows/rust.yml` | bc828166f2b5 | 0e42406ebc9f | 0e42406ebc9f | 0e42406ebc9f |
| `.gitignore` | 0490bc32fbc0 | 0490bc32fbc0 | 0490bc32fbc0 | 0490bc32fbc0 |
| `Cargo.lock` | 8d7a6f06dbc7 | 8d7a6f06dbc7 | 8d7a6f06dbc7 | 8d7a6f06dbc7 |
| `Cargo.toml` | a452968aaf79 | a452968aaf79 | a452968aaf79 | a452968aaf79 |
| `Design-Docs/00-Product-Charter.md` | 42fd44a83645 | 42fd44a83645 | 42fd44a83645 | 42fd44a83645 |
| `Design-Docs/01-Requirements-and-Invariants.md` | f9b797ecb145 | f9b797ecb145 | f9b797ecb145 | f9b797ecb145 |
| `Design-Docs/02-System-Architecture.md` | 1a0093e03229 | 1a0093e03229 | 1a0093e03229 | 1a0093e03229 |
| `Design-Docs/03-Domain-and-Temporal-Model.md` | 3796854d8507 | 3796854d8507 | 3796854d8507 | 3796854d8507 |
| `Design-Docs/04-Genealogy-Relations-and-Genetics.md` | f40ed607a33e | f40ed607a33e | f40ed607a33e | f40ed607a33e |
| `Design-Docs/05-Events-Actors-and-Conflicts.md` | 2df3d8da7f8b | 2df3d8da7f8b | 2df3d8da7f8b | 2df3d8da7f8b |
| `Design-Docs/06-Three-Layer-Entries-and-Canvas.md` | 59a87bf0d793 | 59a87bf0d793 | ca49b1ccc5c8 | ca49b1ccc5c8 |
| `Design-Docs/07-Search-Query-and-Discovery.md` | 0a2efe0f4eae | 0a2efe0f4eae | 0e87a925d9de | 0e87a925d9de |
| `Design-Docs/08-Storage-and-API-Contracts.md` | c3a1b5a74877 | c3a1b5a74877 | acdcabe6a36e | acdcabe6a36e |
| `Design-Docs/09-Acceptance-and-Verification.md` | a36850ba4a56 | a36850ba4a56 | a36850ba4a56 | a36850ba4a56 |
| `Design-Docs/10-Delivery-Roadmap.md` | 8ee408ebf82b | 8ee408ebf82b | 882f774bcf6b | 882f774bcf6b |
| `Design-Docs/11-AI-Engineering-Instructions.md` | 9bd1d10096e4 | 9bd1d10096e4 | 9bd1d10096e4 | 9bd1d10096e4 |
| `Design-Docs/12-Decisions-and-Reconciliation.md` | d80f3ea6bad0 | d80f3ea6bad0 | 815b11dffb9e | 815b11dffb9e |
| `Design-Docs/13-Spatial-and-Nested-Location-Model.md` | 7c3e0a02a412 | 7c3e0a02a412 | 7c3e0a02a412 | 7c3e0a02a412 |
| `Design-Docs/ADRs/adr-007-governed-semantic-extensibility.md` | 5763b8175720 | 5763b8175720 | 5763b8175720 | 5763b8175720 |
| `Design-Docs/ADRs/adr-014-independent-editorial-release.md` | — | — | 1369ac2e9576 | 1369ac2e9576 |
| `Design-Docs/ADRs/branch-local-metadata.md` | dab7222ac345 | dab7222ac345 | dab7222ac345 | dab7222ac345 |
| `Design-Docs/ADRs/cross-phase-lineage-discovery-genetics.md` | 52d7e79d7c6a | 52d7e79d7c6a | 52d7e79d7c6a | 52d7e79d7c6a |
| `Design-Docs/ADRs/cross-phase-network-discovery.md` | cbc01a2cc007 | cbc01a2cc007 | cbc01a2cc007 | cbc01a2cc007 |
| `Design-Docs/ADRs/cross-phase-spatial-cartography.md` | 820e2f22849b | 820e2f22849b | 820e2f22849b | 820e2f22849b |
| `Design-Docs/ADRs/phase-0-architecture-baseline.md` | f5fcc04f5de4 | f5fcc04f5de4 | f5fcc04f5de4 | f5fcc04f5de4 |
| `Design-Docs/ADRs/phase-1-persistence-slice.md` | fc6b87d6fcc5 | fc6b87d6fcc5 | fc6b87d6fcc5 | fc6b87d6fcc5 |
| `Design-Docs/ADRs/phase-2-relationships-agency.md` | 0c2c8012b1c5 | 0c2c8012b1c5 | 0c2c8012b1c5 | 0c2c8012b1c5 |
| `Design-Docs/ADRs/phase-3-authored-entry-links.md` | 6c6b38fcd808 | 6c6b38fcd808 | 6c6b38fcd808 | 6c6b38fcd808 |
| `Design-Docs/ADRs/phase-3-authorized-caption-search.md` | 8b7692a2f432 | 8b7692a2f432 | 8b7692a2f432 | 8b7692a2f432 |
| `Design-Docs/ADRs/phase-3-authorized-entry-search.md` | 8e7742620409 | 8e7742620409 | 8e7742620409 | 8e7742620409 |
| `Design-Docs/ADRs/phase-3-contextual-exploration-review.md` | 6e1b9a448a09 | 6e1b9a448a09 | 6e1b9a448a09 | 6e1b9a448a09 |
| `Design-Docs/ADRs/phase-3-discovery-refresh.md` | 6431047e8955 | 6431047e8955 | 6431047e8955 | 6431047e8955 |
| `Design-Docs/ADRs/phase-3-fts5-label-index.md` | a2e12b57ee3e | a2e12b57ee3e | a2e12b57ee3e | a2e12b57ee3e |
| `Design-Docs/ADRs/phase-3-historical-search-slice.md` | 2b6bdd9f361a | 2b6bdd9f361a | 2b6bdd9f361a | 2b6bdd9f361a |
| `Design-Docs/ADRs/phase-3-incremental-discovery.md` | dd027bd5a975 | dd027bd5a975 | dd027bd5a975 | dd027bd5a975 |
| `Design-Docs/ADRs/phase-3-indexed-aliases.md` | fda8e32bf911 | fda8e32bf911 | fda8e32bf911 | fda8e32bf911 |
| `Design-Docs/ADRs/phase-3-indexed-backlinks.md` | 58e853accf78 | 58e853accf78 | 58e853accf78 | 58e853accf78 |
| `Design-Docs/ADRs/phase-3-indexed-claim-statements.md` | 075e204c74e6 | 075e204c74e6 | 075e204c74e6 | 075e204c74e6 |
| `Design-Docs/ADRs/phase-3-indexed-composite.md` | 0ab10b8b0d7a | 0ab10b8b0d7a | 0ab10b8b0d7a | 0ab10b8b0d7a |
| `Design-Docs/ADRs/phase-3-indexed-event-participation.md` | b528ae6030b3 | b528ae6030b3 | b528ae6030b3 | b528ae6030b3 |
| `Design-Docs/ADRs/phase-3-indexed-paths.md` | 35ec900ac72e | 35ec900ac72e | 35ec900ac72e | 35ec900ac72e |
| `Design-Docs/ADRs/phase-3-lineage-candidates.md` | e3dc627e99a3 | e3dc627e99a3 | e3dc627e99a3 | e3dc627e99a3 |
| `Design-Docs/ADRs/phase-3-mention-discovery.md` | a5e2ba38e8b5 | a5e2ba38e8b5 | a5e2ba38e8b5 | a5e2ba38e8b5 |
| `Design-Docs/ADRs/phase-3-mention-name-index.md` | 2796b6d48ec9 | 2796b6d48ec9 | 2796b6d48ec9 | 2796b6d48ec9 |
| `Design-Docs/ADRs/phase-3-participant-events.md` | 6b2051eb68d2 | 6b2051eb68d2 | 6b2051eb68d2 | 6b2051eb68d2 |
| `Design-Docs/ADRs/phase-3-person-aliases.md` | 4f4fd6072978 | 4f4fd6072978 | 4f4fd6072978 | 4f4fd6072978 |
| `Design-Docs/ADRs/phase-3-query-control.md` | 2f47f101fd5b | 2f47f101fd5b | 2f47f101fd5b | 2f47f101fd5b |
| `Design-Docs/ADRs/phase-3-query-pages.md` | 4be92ed4247b | 4be92ed4247b | 4be92ed4247b | 4be92ed4247b |
| `Design-Docs/ADRs/phase-3-reviewed-mentions.md` | be411f089fbe | be411f089fbe | be411f089fbe | be411f089fbe |
| `Design-Docs/ADRs/phase-3-saved-composite-queries.md` | cf0d805ca8dc | cf0d805ca8dc | cf0d805ca8dc | cf0d805ca8dc |
| `Design-Docs/ADRs/phase-3-saved-label-queries.md` | 2b64dde57381 | 2b64dde57381 | 2b64dde57381 | 2b64dde57381 |
| `Design-Docs/ADRs/phase-3-saved-layout-bindings.md` | 4d56117a886a | 4d56117a886a | 4d56117a886a | 4d56117a886a |
| `Design-Docs/ADRs/phase-3-saved-parameters.md` | d76c8cb1b601 | d76c8cb1b601 | d76c8cb1b601 | d76c8cb1b601 |
| `Design-Docs/ADRs/phase-3-search-only-context.md` | bb360bccd4a5 | bb360bccd4a5 | bb360bccd4a5 | bb360bccd4a5 |
| `Design-Docs/ADRs/phase-3-typed-composite-predicates.md` | 96b3ed776b76 | 96b3ed776b76 | 96b3ed776b76 | 96b3ed776b76 |
| `Design-Docs/ADRs/phase-4-custom-template-library.md` | b7c2a78c7429 | b7c2a78c7429 | b7c2a78c7429 | b7c2a78c7429 |
| `Design-Docs/ADRs/phase-4-general-entry-authoring.md` | b18fe6cb7ccf | b18fe6cb7ccf | b18fe6cb7ccf | b18fe6cb7ccf |
| `Design-Docs/ADRs/phase-4-grid-panel-containers.md` | 54f683a7edad | 54f683a7edad | 54f683a7edad | 54f683a7edad |
| `Design-Docs/ADRs/phase-4-image-assets-portable.md` | 753cc974b00d | 753cc974b00d | 753cc974b00d | 753cc974b00d |
| `Design-Docs/ADRs/phase-4-linked-encyclopedia.md` | 03ba418bc941 | 03ba418bc941 | 03ba418bc941 | 03ba418bc941 |
| `Design-Docs/ADRs/phase-4-structured-canvas-tree.md` | 0bacb31faee4 | 0bacb31faee4 | 0bacb31faee4 | 0bacb31faee4 |
| `Design-Docs/ADRs/phase-4-template-query-studio.md` | f8b4697d60de | f8b4697d60de | f8b4697d60de | f8b4697d60de |
| `Design-Docs/ADRs/phase-5-dated-parentage.md` | 9d3db7a07a03 | 9d3db7a07a03 | 9d3db7a07a03 | 9d3db7a07a03 |
| `Design-Docs/Architectural-Evolutions/Evolutions.md` | 2871f6375468 | 2871f6375468 | 0cea869d381d | 0cea869d381d |
| `Design-Docs/Architectural-Evolutions/independent-editorial-draft-and-publication-lifecycle.md` | — | — | 1e28b3267546 | 1e28b3267546 |
| `Design-Docs/Architectural-Evolutions/ontology-classification-pagination-candidate.md` | — | 97c72e94ecfd | 97c72e94ecfd | 97c72e94ecfd |
| `Design-Docs/Architectural-Evolutions/ontology-consolidation-review-2026-11-22.md` | — | 4a1540e63f36 | 4a1540e63f36 | 4a1540e63f36 |
| `Design-Docs/Architectural-Evolutions/ontology-final-evidence-2026-11-22.md` | — | 43e3bbcecb71 | — | — |
| `Design-Docs/Architectural-Evolutions/ontology-implementation-foundations.md` | — | c96e163345f7 | c96e163345f7 | c96e163345f7 |
| `Design-Docs/Architectural-Evolutions/universal-worldbuilding-ontology-and-governed-semantic-extensibility.md` | 207eb2cd0ffc | be87d8138849 | be87d8138849 | be87d8138849 |
| `Design-Docs/Article-and-Entry-UX-Concepts-2026-11-22.md` | — | — | 17f4945ea219 | — |
| `Design-Docs/Consolidation-2026-11-21.md` | 2e0c1006e506 | 2e0c1006e506 | 2e0c1006e506 | 2e0c1006e506 |
| `Design-Docs/Cross-Phase-Entry-Verification.md` | efff2f5891ad | efff2f5891ad | efff2f5891ad | efff2f5891ad |
| `Design-Docs/Encyclopedia-Audit-2026-11-22.md` | — | — | 17769ac49569 | 17769ac49569 |
| `Design-Docs/Entry-Ontology-Review-2026-11-22.md` | e24bfa288173 | e24bfa288173 | e24bfa288173 | e24bfa288173 |
| `Design-Docs/Features/Features.md` | 706f88e5fa8e | 706f88e5fa8e | 72088c10287b | 72088c10287b |
| `Design-Docs/Features/composable-worldbuilding-transforms-and-exploration-workflows.md` | 603645f42414 | 603645f42414 | 603645f42414 | 603645f42414 |
| `Design-Docs/Features/configurable-world-management-workspaces-and-dashboards.md` | — | — | c4ff293ebbd8 | c4ff293ebbd8 |
| `Design-Docs/Features/nested-locations-and-procedural-vector-cartography.md` | 4ae7a0bcf2c4 | 4ae7a0bcf2c4 | 4ae7a0bcf2c4 | 4ae7a0bcf2c4 |
| `Design-Docs/Functionality-Audit-2026-11-22.md` | — | — | fc791a342298 | fc791a342298 |
| `Design-Docs/Phase-0-Verification.md` | 34c2fdfd11fb | 34c2fdfd11fb | 34c2fdfd11fb | 34c2fdfd11fb |
| `Design-Docs/Phase-Foundation.md` | 49ac6deb403e | 49ac6deb403e | 49ac6deb403e | 49ac6deb403e |
| `Design-Docs/README.md` | dd5e6ce327fc | af3cfd3f06b2 | b9781622030c | af3cfd3f06b2 |
| `Design-Docs/Repository-Audit-2026-11-21.md` | ea47f0529680 | ea47f0529680 | ea47f0529680 | ea47f0529680 |
| `Design-Docs/Research-Studies/Studies.md` | 6d9323653ad8 | 6d9323653ad8 | d81ad6d1992c | 2111eef2c075 |
| `Design-Docs/Research-Studies/article-search-prototype/README.md` | — | — | ddfed76153fc | ddfed76153fc |
| `Design-Docs/Research-Studies/article-search-prototype/check.py` | — | — | e5148083bde5 | e5148083bde5 |
| `Design-Docs/Research-Studies/article-search-prototype/results.json` | — | — | 45a45a836432 | 45a45a836432 |
| `Design-Docs/Research-Studies/cartography-prototype/check.py` | b6ae0bd80337 | b6ae0bd80337 | b6ae0bd80337 | b6ae0bd80337 |
| `Design-Docs/Research-Studies/cartography-prototype/followup.py` | c4e4121ade36 | c4e4121ade36 | c4e4121ade36 | c4e4121ade36 |
| `Design-Docs/Research-Studies/cartography-prototype/prototype.html` | 63f69e5cff56 | 63f69e5cff56 | 63f69e5cff56 | 63f69e5cff56 |
| `Design-Docs/Research-Studies/comparative-research-ticket-disposition-2026-11-22.md` | — | — | f4dbc3b43498 | f4dbc3b43498 |
| `Design-Docs/Research-Studies/comparative-worldbuilding-workspaces-and-encyclopedia-ux.md` | — | — | c45e85ee308a | c45e85ee308a |
| `Design-Docs/Research-Studies/maltego-inspired-transforms.md` | af1063e2842e | af1063e2842e | af1063e2842e | af1063e2842e |
| `Design-Docs/Research-Studies/nested-locations.md` | 2c2ccb966f19 | 2c2ccb966f19 | 2c2ccb966f19 | 2c2ccb966f19 |
| `Design-Docs/Research-Studies/ontology-prototype/check.py` | 4fc13ec463e7 | 4fc13ec463e7 | 4fc13ec463e7 | 4fc13ec463e7 |
| `Design-Docs/Research-Studies/ontology-prototype/results.txt` | 92e700619619 | 92e700619619 | 92e700619619 | 92e700619619 |
| `Design-Docs/Research-Studies/ontology-value-prototype/check.py` | dc6c481f800b | dc6c481f800b | dc6c481f800b | dc6c481f800b |
| `Design-Docs/Research-Studies/ontology-value-prototype/results.txt` | 9386438c1415 | 9386438c1415 | 9386438c1415 | 9386438c1415 |
| `Design-Docs/Research-Studies/procedural-vector-cartography.md` | 76c0b3cd7763 | 76c0b3cd7763 | 76c0b3cd7763 | 76c0b3cd7763 |
| `Design-Docs/Research-Studies/transform-prototype/check.py` | 8c2d45dade93 | 8c2d45dade93 | 8c2d45dade93 | 8c2d45dade93 |
| `Design-Docs/Research-Studies/universal-worldbuilding-ontology.md` | 7626deb32970 | 7626deb32970 | 7626deb32970 | 7626deb32970 |
| `Design-Docs/Research-Studies/workspace-lifecycle-prototype/README.md` | — | — | c6e970e5c8a7 | c6e970e5c8a7 |
| `Design-Docs/Research-Studies/workspace-lifecycle-prototype/check.js` | — | — | 69a19a8078e7 | 69a19a8078e7 |
| `Design-Docs/Research-Studies/workspace-lifecycle-prototype/results.json` | — | — | 710674358480 | 710674358480 |
| `Design-Docs/Research-Studies/workspace-projection-prototype/README.md` | — | — | 66fa3bd16862 | 66fa3bd16862 |
| `Design-Docs/Research-Studies/workspace-projection-prototype/check.js` | — | — | 47ad5ecfabfa | 47ad5ecfabfa |
| `Design-Docs/Research-Studies/workspace-projection-prototype/results.json` | — | — | 05bdbdb61baa | 05bdbdb61baa |
| `Design-Docs/Research-Studies/world-bible-prototype/index.html` | — | — | ccb079d19dff | — |
| `Design-Docs/Research-Studies/world-bible-research-reassessment-2026-11-22.md` | — | — | 3a4e68acd76e | — |
| `Design-Docs/UI-UX-Audit-2026-11-22.md` | — | — | fed90fc0e67a | fed90fc0e67a |
| `Design-Docs/UI-Visual-Design-Audit-2026-11-22.md` | — | — | f5d00fc13e2f | f5d00fc13e2f |
| `Design-Docs/Workspace-Context-Safety-2026-11-22.md` | — | — | — | 4a2b915265e3 |
| `Design-Docs/World-Bible-UX-Direction-2026-11-22.md` | — | — | bb4eed622644 | — |
| `Development-Screenshots/2026-11-20_22-52-16Z_phase-00_desktop_keyboard-canvas-fallback_01.png` | 124148dbcfba | 124148dbcfba | 124148dbcfba | 124148dbcfba |
| `Development-Screenshots/2026-11-20_23-13-39Z_phase-00_renderer_benchmark-canvas-fallback_01.png` | 27f9734a3035 | 27f9734a3035 | 27f9734a3035 | 27f9734a3035 |
| `Development-Screenshots/2026-11-21_03-34-12Z_phase-02_relationships_recorded-before-and-after_01.png` | bdc0b4d2e6ef | bdc0b4d2e6ef | bdc0b4d2e6ef | bdc0b4d2e6ef |
| `Development-Screenshots/2026-11-21_03-34-13Z_phase-02_graph_multi-actor-event_01.png` | b4c2bc789490 | b4c2bc789490 | b4c2bc789490 | b4c2bc789490 |
| `Development-Screenshots/2026-11-21_03-34-14Z_phase-02_events_saved-action-correction_01.png` | 687af2862746 | 687af2862746 | 687af2862746 | 687af2862746 |
| `Development-Screenshots/2026-11-21_04-41-30Z_phase-02_evidence_author-account-inspector_01.png` | e26d25018000 | e26d25018000 | e26d25018000 | e26d25018000 |
| `Development-Screenshots/2026-11-21_04-41-31Z_phase-02_evidence_public-rumor-preview_01.png` | 5809fd1bf305 | 5809fd1bf305 | 5809fd1bf305 | 5809fd1bf305 |
| `Development-Screenshots/2026-11-21_05-12-22Z_phase-00_renderer_benchmark-fallback-state_01.png` | e275c982496f | e275c982496f | e275c982496f | e275c982496f |
| `Development-Screenshots/2026-11-21_18-34-46Z_phase-03_query_mock-native-page-two_01.png` | 474167521aea | 474167521aea | 474167521aea | 474167521aea |
| `Development-Screenshots/2026-11-21_19-00-22Z_phase-03_query_saved-parameter-override_01.png` | f7237d6c0ce0 | f7237d6c0ce0 | f7237d6c0ce0 | f7237d6c0ce0 |
| `Development-Screenshots/2026-11-21_19-18-45Z_phase-03_discovery_focused-claim_01.png` | 9efa19732056 | 9efa19732056 | 9efa19732056 | 9efa19732056 |
| `Development-Screenshots/2026-11-21_20-07-42Z_phase-04_character_public-page-preview_01.png` | 665361712364 | 665361712364 | 665361712364 | 665361712364 |
| `Development-Screenshots/2026-11-21_20-38-21Z_phase-05_family_saved-parentage-comparison_01.png` | 4e3c1cbd95fc | 4e3c1cbd95fc | 4e3c1cbd95fc | 4e3c1cbd95fc |
| `Development-Screenshots/2026-11-21_23-49-10Z_phase-04_links_mock-native-reader-backlinks_01.png` | 3ab1e28287eb | 3ab1e28287eb | 3ab1e28287eb | 3ab1e28287eb |
| `Development-Screenshots/2026-11-22_00-50-15Z_phase-04_canvas_mock-native-source-entry_01.png` | e76076f9b551 | e76076f9b551 | e76076f9b551 | e76076f9b551 |
| `Development-Screenshots/2026-11-22_00-52-28Z_phase-04_canvas_mock-native-panel_01.png` | 192a1db57b83 | 192a1db57b83 | 192a1db57b83 | 192a1db57b83 |
| `Development-Screenshots/2026-11-22_00-59-18Z_phase-04_canvas_mock-native-tree_01.png` | 260519353ccb | 260519353ccb | 260519353ccb | 260519353ccb |
| `Development-Screenshots/2026-11-22_00-59-19Z_phase-04_canvas_mock-native-template-query_01.png` | a967607db1c7 | a967607db1c7 | a967607db1c7 | a967607db1c7 |
| `Development-Screenshots/2026-11-22_01-06-45Z_phase-05_network_mock-native-institution-ghost_01.png` | 1a0d0fb28151 | 1a0d0fb28151 | 1a0d0fb28151 | 1a0d0fb28151 |
| `Development-Screenshots/2026-11-22_01-40-17Z_phase-04_canvas_mock-native-custom-template_01.png` | f8ea69ed2720 | f8ea69ed2720 | f8ea69ed2720 | f8ea69ed2720 |
| `Development-Screenshots/2026-11-22_02-58-48Z_phase-04_cartography_research-66-browser-prototype_01.png` | 9367fff5fb10 | 9367fff5fb10 | 9367fff5fb10 | 9367fff5fb10 |
| `Development-Screenshots/2026-11-22_03-19-09Z_phase-04_cartography_research-66-locked-unknown-fixture_01.png` | 9076a153ee7f | 9076a153ee7f | 9076a153ee7f | 9076a153ee7f |
| `Development-Screenshots/2026-11-22_05-34-14Z_phase-03_query_native-ontology-author-tasks_01.png` | — | 35237c0104d4 | 35237c0104d4 | 35237c0104d4 |
| `Development-Screenshots/Dev-Screenshots.md` | 015ed0e37cce | d2f757dcbe6c | d2f757dcbe6c | d2f757dcbe6c |
| `Development-Screenshots/Verified-Capture-Hashes.md` | b629953e0530 | b629953e0530 | b629953e0530 | b629953e0530 |
| `README.md` | 67d79641717d | 67d79641717d | 67d79641717d | 67d79641717d |
| `apps/desktop/index.html` | d9eda8b6b669 | 0cc6eecb0aca | 0cc6eecb0aca | ec0c5036c800 |
| `apps/desktop/package-lock.json` | 0f4b34e4cb22 | 0f4b34e4cb22 | 0f4b34e4cb22 | 0f4b34e4cb22 |
| `apps/desktop/package.json` | b2b6061658d1 | b2b6061658d1 | b2b6061658d1 | b2b6061658d1 |
| `apps/desktop/playwright.config.ts` | 261d9d9291b8 | 261d9d9291b8 | 261d9d9291b8 | 261d9d9291b8 |
| `apps/desktop/scripts/build-renderer.mjs` | 7de084dd523e | 7de084dd523e | 7de084dd523e | 7de084dd523e |
| `apps/desktop/scripts/dev-server-smoke.mjs` | 86e13b69151c | 86e13b69151c | 86e13b69151c | 86e13b69151c |
| `apps/desktop/scripts/launch-config.test.mjs` | bc901fdfa73b | bc901fdfa73b | bc901fdfa73b | bc901fdfa73b |
| `apps/desktop/scripts/windows-platform-smoke.ps1` | df4796e6e029 | fea25844529f | fea25844529f | fea25844529f |
| `apps/desktop/src-tauri/Cargo.toml` | ea09c1d64fd1 | ea09c1d64fd1 | ea09c1d64fd1 | ea09c1d64fd1 |
| `apps/desktop/src-tauri/build.rs` | 09a3b71c1a0b | 2beb00bec203 | 2beb00bec203 | 2beb00bec203 |
| `apps/desktop/src-tauri/capabilities/platform-shell.json` | 62b189bb8aa8 | 8be690148acb | 8be690148acb | 8be690148acb |
| `apps/desktop/src-tauri/icons/icon.ico` | a4892519ee87 | a4892519ee87 | a4892519ee87 | a4892519ee87 |
| `apps/desktop/src-tauri/icons/icon.png` | fcd302f4a2d0 | fcd302f4a2d0 | fcd302f4a2d0 | fcd302f4a2d0 |
| `apps/desktop/src-tauri/src/history.rs` | c1223da4698e | c1223da4698e | c1223da4698e | c1223da4698e |
| `apps/desktop/src-tauri/src/main.rs` | 01b6b5c01f9d | 29060ab7227f | 29060ab7227f | 29060ab7227f |
| `apps/desktop/src-tauri/src/workspace.rs` | 551be09bf19b | 9277c5f264eb | 6e2ee1fdf953 | d53cb701ee9a |
| `apps/desktop/src-tauri/tauri.conf.json` | e0e14c6d371e | e0e14c6d371e | e0e14c6d371e | e0e14c6d371e |
| `apps/desktop/src/author-workspace.ts` | 786d0499e79b | 603fcc2a09e9 | 603fcc2a09e9 | b7c21cdd41f7 |
| `apps/desktop/src/canvas-templates.ts` | 3433f1b833be | 3433f1b833be | 3433f1b833be | 3433f1b833be |
| `apps/desktop/src/canvas-tree.ts` | 87dec45273e1 | 6dd416657997 | 6dd416657997 | 25848a69b544 |
| `apps/desktop/src/entry-drafts.ts` | — | — | — | f6d0cb355268 |
| `apps/desktop/src/entry-editor.ts` | 3dc2ba3265c7 | 7ef90c32c6c5 | 7ef90c32c6c5 | 179f6f1e47bb |
| `apps/desktop/src/family-history.ts` | 0faf49a137b1 | 0faf49a137b1 | 0faf49a137b1 | ac8640f3b78d |
| `apps/desktop/src/graph-benchmark.ts` | 5eb6628a8fc8 | 5eb6628a8fc8 | 5eb6628a8fc8 | 5eb6628a8fc8 |
| `apps/desktop/src/main.ts` | 6b843cdee4ea | 177243d13dc7 | 177243d13dc7 | 44aed913002a |
| `apps/desktop/src/media-studio.ts` | b37cfb886fa7 | b37cfb886fa7 | b37cfb886fa7 | b662dcf1b7af |
| `apps/desktop/src/network-studio.ts` | 74395ff6c5cf | ebc3722abf5a | 9a224929e997 | ddeec1c5952e |
| `apps/desktop/src/northbridge.json` | 2c12c55d06ed | 2c12c55d06ed | 2c12c55d06ed | 2c12c55d06ed |
| `apps/desktop/src/object-kinds.ts` | — | — | — | ae0cc784d042 |
| `apps/desktop/src/ontology-smoke.ts` | — | d9303a1fd974 | d9303a1fd974 | d9303a1fd974 |
| `apps/desktop/src/public-reader.ts` | d1b53b4da787 | d1b53b4da787 | d1b53b4da787 | d1b53b4da787 |
| `apps/desktop/src/reference-picker.ts` | — | — | — | 82c2dabfcabd |
| `apps/desktop/src/style.css` | 9cf4a4ba6ce2 | 9cf4a4ba6ce2 | 9cf4a4ba6ce2 | 9cf4a4ba6ce2 |
| `apps/desktop/tests/platform.spec.ts` | c8254dd98246 | c0d97219618f | 07dfef78f40c | e9dc28976e6b |
| `apps/desktop/tests/world-bible-prototype.spec.ts` | — | — | 186baf5f5914 | — |
| `apps/desktop/tsconfig.json` | ecd104cfa8f0 | ecd104cfa8f0 | ecd104cfa8f0 | ecd104cfa8f0 |
| `crates/atlas-renderer/Cargo.toml` | 60dc2b592f43 | 60dc2b592f43 | 60dc2b592f43 | 60dc2b592f43 |
| `crates/atlas-renderer/src/graph.wgsl` | f883cb9eb306 | f883cb9eb306 | f883cb9eb306 | f883cb9eb306 |
| `crates/atlas-renderer/src/lib.rs` | 9386c834ea97 | 9386c834ea97 | 9386c834ea97 | 9386c834ea97 |
| `crates/atlas-store/Cargo.toml` | 0056b6a73252 | 0056b6a73252 | 0056b6a73252 | 0056b6a73252 |
| `crates/atlas-store/examples/focused_world.rs` | 32175b777180 | 32175b777180 | 32175b777180 | 32175b777180 |
| `crates/atlas-store/examples/migration_baseline.rs` | — | a81e52ccb03d | a81e52ccb03d | a81e52ccb03d |
| `crates/atlas-store/examples/ontology_author_tasks.rs` | — | c3810935e100 | c3810935e100 | c3810935e100 |
| `crates/atlas-store/examples/semantic_workload.rs` | — | 6b570afbd105 | f38cf7aa769e | f38cf7aa769e |
| `crates/atlas-store/migrations/0001.sql` | 1e99aea95591 | 1e99aea95591 | 1e99aea95591 | 1e99aea95591 |
| `crates/atlas-store/migrations/0002.sql` | 044c7bf9e5a1 | 044c7bf9e5a1 | 044c7bf9e5a1 | 044c7bf9e5a1 |
| `crates/atlas-store/migrations/0003.sql` | 429337fe54c8 | 429337fe54c8 | 429337fe54c8 | 429337fe54c8 |
| `crates/atlas-store/migrations/0004.sql` | b710ae992be5 | b710ae992be5 | b710ae992be5 | b710ae992be5 |
| `crates/atlas-store/migrations/0005.sql` | 167b65bdea69 | 167b65bdea69 | 167b65bdea69 | 167b65bdea69 |
| `crates/atlas-store/migrations/0006.sql` | 0dc7fc89e156 | 0dc7fc89e156 | 0dc7fc89e156 | 0dc7fc89e156 |
| `crates/atlas-store/migrations/0007.sql` | 6da21f792e1b | 6da21f792e1b | 6da21f792e1b | 6da21f792e1b |
| `crates/atlas-store/migrations/0008.sql` | 276c7db3605e | 276c7db3605e | 276c7db3605e | 276c7db3605e |
| `crates/atlas-store/migrations/0009.sql` | 54b93c141ca1 | 54b93c141ca1 | 54b93c141ca1 | 54b93c141ca1 |
| `crates/atlas-store/migrations/0010.sql` | 4806b65cb8e4 | 4806b65cb8e4 | 4806b65cb8e4 | 4806b65cb8e4 |
| `crates/atlas-store/migrations/0011.sql` | ec02ae44c749 | ec02ae44c749 | ec02ae44c749 | ec02ae44c749 |
| `crates/atlas-store/migrations/0012.sql` | 00d49e950efc | 00d49e950efc | 00d49e950efc | 00d49e950efc |
| `crates/atlas-store/migrations/0013.sql` | ae1233149c5c | ae1233149c5c | ae1233149c5c | ae1233149c5c |
| `crates/atlas-store/migrations/0014.sql` | eee9acc49bdd | eee9acc49bdd | eee9acc49bdd | eee9acc49bdd |
| `crates/atlas-store/migrations/0016.sql` | 4260483470bd | 4260483470bd | 4260483470bd | 4260483470bd |
| `crates/atlas-store/migrations/0017.sql` | f5b99c9c591f | f5b99c9c591f | f5b99c9c591f | f5b99c9c591f |
| `crates/atlas-store/migrations/0018.sql` | 3534c70ca268 | 3534c70ca268 | 3534c70ca268 | 3534c70ca268 |
| `crates/atlas-store/migrations/0019.sql` | 542ca199e695 | 542ca199e695 | 542ca199e695 | 542ca199e695 |
| `crates/atlas-store/migrations/0020.sql` | c00479ba90f2 | c00479ba90f2 | c00479ba90f2 | c00479ba90f2 |
| `crates/atlas-store/migrations/0021.sql` | 3190fd2981f2 | 3190fd2981f2 | 3190fd2981f2 | 3190fd2981f2 |
| `crates/atlas-store/migrations/0022.sql` | c25fe95f28d5 | c25fe95f28d5 | c25fe95f28d5 | c25fe95f28d5 |
| `crates/atlas-store/migrations/0023.sql` | 7eccc135a8e8 | 7eccc135a8e8 | 7eccc135a8e8 | 7eccc135a8e8 |
| `crates/atlas-store/migrations/0024.sql` | 6525ba5fc7c0 | 6525ba5fc7c0 | 6525ba5fc7c0 | 6525ba5fc7c0 |
| `crates/atlas-store/migrations/0026.sql` | 302183f291b5 | 302183f291b5 | 302183f291b5 | 302183f291b5 |
| `crates/atlas-store/migrations/0027.sql` | 6ab998b3e29f | 6ab998b3e29f | 6ab998b3e29f | 6ab998b3e29f |
| `crates/atlas-store/migrations/0030.sql` | d85eee50aa39 | d85eee50aa39 | d85eee50aa39 | d85eee50aa39 |
| `crates/atlas-store/migrations/0031.sql` | ba8861540683 | ba8861540683 | ba8861540683 | ba8861540683 |
| `crates/atlas-store/migrations/0032.sql` | fd4d2cb9ddf6 | fd4d2cb9ddf6 | fd4d2cb9ddf6 | fd4d2cb9ddf6 |
| `crates/atlas-store/migrations/0033.sql` | 4564781d6511 | 4564781d6511 | 4564781d6511 | 4564781d6511 |
| `crates/atlas-store/migrations/0034.sql` | 19d6b16054fe | 19d6b16054fe | 19d6b16054fe | 19d6b16054fe |
| `crates/atlas-store/migrations/0035.sql` | 45d86039b9bc | 45d86039b9bc | 45d86039b9bc | 45d86039b9bc |
| `crates/atlas-store/migrations/0036.sql` | 747257c4fa0b | 747257c4fa0b | 747257c4fa0b | 747257c4fa0b |
| `crates/atlas-store/migrations/0037.sql` | — | 5416a53a4d71 | 5416a53a4d71 | 5416a53a4d71 |
| `crates/atlas-store/migrations/0038.sql` | — | e8959faee87f | e8959faee87f | e8959faee87f |
| `crates/atlas-store/migrations/0039.sql` | — | 68c72955cf04 | 68c72955cf04 | 68c72955cf04 |
| `crates/atlas-store/migrations/0040.sql` | — | 8237a011011b | 8237a011011b | 8237a011011b |
| `crates/atlas-store/migrations/0041.sql` | — | 3e8ff779bce9 | 3e8ff779bce9 | 3e8ff779bce9 |
| `crates/atlas-store/migrations/0042.sql` | — | 64d0abac46e3 | 64d0abac46e3 | 64d0abac46e3 |
| `crates/atlas-store/migrations/0043.sql` | — | 27a03dbe156c | 27a03dbe156c | 27a03dbe156c |
| `crates/atlas-store/migrations/0044.sql` | — | 97e1492768e5 | 97e1492768e5 | 97e1492768e5 |
| `crates/atlas-store/src/access.rs` | 8ea87310befc | 8ea87310befc | 8ea87310befc | 8ea87310befc |
| `crates/atlas-store/src/archive.rs` | c1cc3134b337 | 64fdb7dad5d9 | 64fdb7dad5d9 | 64fdb7dad5d9 |
| `crates/atlas-store/src/calendar.rs` | 952a70b1cc0a | 952a70b1cc0a | 952a70b1cc0a | 952a70b1cc0a |
| `crates/atlas-store/src/conflict.rs` | da261c986805 | da261c986805 | da261c986805 | da261c986805 |
| `crates/atlas-store/src/discovery.rs` | 6b7ce3627217 | 6b7ce3627217 | 6b7ce3627217 | 6b7ce3627217 |
| `crates/atlas-store/src/genealogy.rs` | e5d41b1755ed | e5d41b1755ed | e5d41b1755ed | e5d41b1755ed |
| `crates/atlas-store/src/graph.rs` | 3bb5e8e80169 | 3bb5e8e80169 | 3bb5e8e80169 | 3bb5e8e80169 |
| `crates/atlas-store/src/inheritance.rs` | 9508c325dad0 | 9508c325dad0 | 9508c325dad0 | 9508c325dad0 |
| `crates/atlas-store/src/layout.rs` | 97db3d756d3c | 1552ce2c7797 | 1552ce2c7797 | 1552ce2c7797 |
| `crates/atlas-store/src/lib.rs` | 51a44f587658 | 6c6777343534 | 6c6777343534 | 6c6777343534 |
| `crates/atlas-store/src/network.rs` | 3d207b345351 | c227fb423e48 | c227fb423e48 | c227fb423e48 |
| `crates/atlas-store/src/query.rs` | d1e4fef0b342 | e56a090e2d73 | e56a090e2d73 | e56a090e2d73 |
| `crates/atlas-store/src/semantic.rs` | — | c865c9c68163 | 177da86450b8 | 177da86450b8 |
| `crates/atlas-store/src/store.rs` | c5c78282a8bc | 112d4267934c | 112d4267934c | 112d4267934c |
| `crates/atlas-store/src/store/annotation.rs` | — | 98e8bb82137a | 98e8bb82137a | 98e8bb82137a |
| `crates/atlas-store/src/store/classification.rs` | — | be4c7c95cef0 | be4c7c95cef0 | be4c7c95cef0 |
| `crates/atlas-store/src/store/entries.rs` | f2ad92609cc1 | da0d2e580ed7 | da0d2e580ed7 | da0d2e580ed7 |
| `crates/atlas-store/src/store/family.rs` | d0f5047d02c8 | d0f5047d02c8 | d0f5047d02c8 | d0f5047d02c8 |
| `crates/atlas-store/src/store/media.rs` | 669a3cfba877 | 669a3cfba877 | 669a3cfba877 | 669a3cfba877 |
| `crates/atlas-store/src/store/network.rs` | 971432c670b8 | 5474c8dc01dd | 5474c8dc01dd | 5474c8dc01dd |
| `crates/atlas-store/src/store/semantic.rs` | — | e4aa84d3df65 | 6cfbf923e045 | 6cfbf923e045 |
| `crates/atlas-store/src/store/site_area.rs` | — | 0fefa4fcdc5a | 0fefa4fcdc5a | 0fefa4fcdc5a |
| `crates/atlas-store/src/store/templates.rs` | d7ce8f1830e3 | fb1bf090c2f4 | fb1bf090c2f4 | fb1bf090c2f4 |
| `crates/atlas-store/tests/calendars.rs` | 6939998a949b | 6939998a949b | 6939998a949b | 6939998a949b |
| `crates/atlas-store/tests/claim_search.rs` | 7dcad5a34550 | 7dcad5a34550 | 7dcad5a34550 | 7dcad5a34550 |
| `crates/atlas-store/tests/composite_query.rs` | cc47397a3ac9 | cc47397a3ac9 | cc47397a3ac9 | cc47397a3ac9 |
| `crates/atlas-store/tests/evidence.rs` | ea9448ebd45e | ea9448ebd45e | ea9448ebd45e | ea9448ebd45e |
| `crates/atlas-store/tests/indexed_paths.rs` | d43e5136dc73 | d43e5136dc73 | d43e5136dc73 | d43e5136dc73 |
| `crates/atlas-store/tests/mention_discovery.rs` | 2cd62cab7bf5 | 2cd62cab7bf5 | 2cd62cab7bf5 | 2cd62cab7bf5 |
| `crates/atlas-store/tests/mention_reviews.rs` | 445329cfabfc | 445329cfabfc | 445329cfabfc | 445329cfabfc |
| `crates/atlas-store/tests/migration_release.rs` | — | 191d3e8fe3f5 | 191d3e8fe3f5 | 191d3e8fe3f5 |
| `crates/atlas-store/tests/participant_events.rs` | bbfd45a1f9e6 | bbfd45a1f9e6 | bbfd45a1f9e6 | bbfd45a1f9e6 |
| `crates/atlas-store/tests/persistence.rs` | c35f5b623f95 | a055645ba9f5 | a055645ba9f5 | a055645ba9f5 |
| `crates/atlas-store/tests/person_aliases.rs` | 050b2f1768ff | 050b2f1768ff | 050b2f1768ff | 050b2f1768ff |
| `crates/atlas-store/tests/relationships.rs` | 64b6d3e2653f | 64b6d3e2653f | 64b6d3e2653f | 64b6d3e2653f |
| `crates/atlas-store/tests/roadmap_foundations.rs` | efb723c6510c | 250b6d96078d | fdb66ad6406e | fdb66ad6406e |
| `crates/atlas-store/tests/saved_composites.rs` | c9e1bcd35daf | c9e1bcd35daf | c9e1bcd35daf | c9e1bcd35daf |
| `crates/atlas-store/tests/saved_queries.rs` | 3ca846518b06 | 3ca846518b06 | 3ca846518b06 | 3ca846518b06 |
| `crates/atlas-store/tests/search_index.rs` | 4b6cf2d17b32 | 4b6cf2d17b32 | 4b6cf2d17b32 | 4b6cf2d17b32 |
| `start.bat` | c45985dfea44 | c45985dfea44 | c45985dfea44 | c45985dfea44 |
