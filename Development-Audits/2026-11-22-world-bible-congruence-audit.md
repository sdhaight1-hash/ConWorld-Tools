# ConWorld Tools — full repository congruence audit, World Bible-first

**Status:** In Review, dated 2026-11-22 UTC. Author/reviewer: Lola. Requested comprehensive repository assessment, not a new architectural evolution or authorization to rewrite production. [Inventory annex](2026-11-22-world-bible-congruence-inventory.md). [Revised UX direction](../Design-Docs/World-Bible-UX-Direction-2026-11-22.md), [research reassessment](../Design-Docs/Research-Studies/world-bible-research-reassessment-2026-11-22.md). This audit is isolated on `audit/world-bible-congruence-2026-11-22` to avoid altering candidates or cancelling in-progress checks.

## Executive conclusion

Retain the historical typed semantic model, SQLite/Rust authority, separate Canvas composition, authorization and publication projection. The World Bible is the product; the three representations are architectural concerns, not three obligatory screens. The main risk is **integration and coherence**, followed by actual end-to-end usability, not missing ontology features. There are currently three separate versions of substantial work: consolidated ontology evidence, context/draft/reference safety, and World Bible research/design/prototype. Additional verified recovery and network fixes, test world and cartographic research reside on sibling branches. Main is older than all of these. Do not abandon the backend, start a greenfield UI, or mechanically merge different schema lineages.

### Method and boundaries

Enumerated every branch, PR and non-PR issue metadata, all 235 main-history commits through an empty subsequent page; recursive inventories of main, #80, review and #79; read all 14 numbered canonical document sections/relevant excerpts, README governance and screenshots guide, formal issue bodies/checklists, prior research and audit source extracts, active branch-specific audit/fixture/label report, relevant candidate code for layout/reader/search/network/backup/reference/drafts and prototype, actual current CI jobs and artifact metadata. Compared non-merged divergent branches to primary candidate; literal ancestry and GitHub merged PR metadata distinguished. The complete file/branch/ticket annex includes exact refs. This is a repository-wide risk-based static congruence review, not exhaustive line-by-line review of every historic branch, security penetration testing, new native build or runtime usability study. Closed PR titles/heads/merge states reviewed; not every historic review comment or diff. The attached README is not substituted for live README. Previous research evidence retained without rerunning. No screenshot fabricated; new visual evidence is in CI artifacts, not independently downloaded or visually inspected during this audit.

## 1. Repository facts and exact-head verification

| Branch / exact head | Schema / files | Role |
| --- | --- | --- |
| main [5777f0fae](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/5777f0faea7dd5c4aa93908d35d3cfd7cfea773f) | 36 / 215 (245 tree entries) | Public development baseline; not final release |
| work/ontology-consolidation [30b6710a5](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/30b6710a583550df68e8ba88cb26c6ad01b26e56) | 44 | Frozen #75 green base |
| work/ontology-final-evidence [1aa9b0b6f](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/1aa9b0b6fa6a4f189e5ee0bb5d88ca4a9129aa0d) | 44 / 238 (268 entries) | #80 six commits after #75; 68 ahead of main; 51 files, +8422/-122 |
| work/workspace-context-safety [dac1ab2ab](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/dac1ab2ab5bdf9a279d73793132560d36dd21a5e) | 44 / 259 (292 entries) | #79 independently tested; not integrated into #80 |
| review/ui-ux-audit [e2947c3e4](https://github.com/sdhaight1-hash/ConWorld-Tools/commit/e2947c3e4f2c4e3c12b7a9c121423daff06c202c) | 44 / 260 (294 entries) | World Bible and #77/#78 documents/prototype; not #79 or #80 code |

GitHub returned 73 existing branches, 80 numbered records (20 issues + 60 PRs), 13 open formal issues, 7 closed formal issues, 53 merged PRs, 5 open draft PRs and 2 closed unmerged (#13, #74). 35 branch heads including main occur literally in main history; 19 further branch heads correspond to merged PRs but are not literal ancestors; 19 are neither (see branch annex). No deletion recommendation based on merge status alone. No count of total project completion from tickets, commits or tests.

| CI evidence | Result at observation | Caveat |
| --- | --- | --- |
| #80 exact 1aa9b0b [desktop 45074268551](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45074268551) | both OS jobs success; Windows native, Linux browser fallback and Rust workspace tests success | Browser is mock-native; screenshot existence not inspection; no combined integration |
| #80 [store 45074268553](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45074268553) | Ubuntu + Windows verify passed, predecessor schema36 migration and A/B/C comparison step passed; both scale jobs in progress | A/B/C automated tasks are not comparative human native UX; no new complete matrix claim |
| #79 exact dac1ab2 [desktop 45065303176](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45065303176) | Windows and Linux success | Different base; must re-run after integration |
| #75 exact 30b6710 [store 45019544281](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45019544281) / [desktop 45019544285](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45019544285) | six jobs green | Earlier base |
| Recovery 3ecd5bf [store 45019528360](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45019528360) / [desktop 45019528374](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45019528374) | independently green | Not included in #80 |
| Alder Basin 8a451ac [45019173223](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45019173223) | store incl. scale all green; previous desktop 45019173220 green | Expanded 91b1e18 unverified |
| review 20439fb [45074360856](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45074360856) | cancelled by next review push | Not failure or success |
| review e2947c3 [45074574231](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45074574231) | both desktop jobs in progress | No verified prototype result yet |

The native ontology screenshot artifact [14775175413](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45074268551/artifacts/14775175413) and semantic correction annotation [14775115987](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45074268551/artifacts/14775115987) were reported present, unexpired. Other 14 artifact groups present. Preserve useful actual captures in Development-Screenshots, validate file/date/hash and distinguish mock-native/browser/native; do not claim inspection from metadata. Audit produced no new visual UI.

## 2. Architecture congruence matrix

| Subsystem / contract | Reuse | Required change / gap |
| --- | --- | --- |
| Domain 03, registry, migrations, calendar, access | Retain | Shared stable identity; optional article. Preserve fictional effective time, recorded revision, branch, audience separately. No page-driven data model |
| Ontology #72, semantic.rs, classification, site_area, annotation, network | Retain with scope stop | Expose existing typed classifications and specialized controls; no generalized schema/ECS; category and display template separate. Core relation types/roles validated by Rust. AE gates independent |
| Query/discovery 07, query.rs, discovery.rs, FTS and saved definitions | Retain | Entry/go-to, in-world research and in-article search as separate tasks in same shell. Author paths/claims vs public projection distinct; avoid hidden facets and candidate leakage |
| Graph/renderer 02, graph.rs, atlas-renderer | Retain contextual | Focused graph/list and cable editing optional; no mandatory scene before editing, native accessibility and bounds |
| Genealogy/institution 04, family.rs, genealogy.rs, network.rs | Retain | Contextual family tree, institutional hierarchy, membership/office/title components, explicit independent relations. Cadet branch and corporate hierarchy require actual typed support rather than invented generic nesting |
| Event/conflict 05, conflict.rs, graph.rs | Retain | Event and conflict article templates, timeline/causal study from selected entry. Preserve causation vs sequence vs allegation, actor/effect provenance |
| Place/cartography 13 and #68 | Retain | Place article + contextual physical/admin hierarchy and map; exact vs schematic vs draft. Never infer geography from folders, control from presence, or invent coordinates |
| Canvas 06, layout.rs, store/templates.rs, store/entries.rs | Reuse substantial base | Unified semantic editing around the layout, reusable components and article presets; layout stores bindings not duplicate facts. Remove layout element not canonical fact. Need structured rich text/heading/link/citation, explicit self-bound vs literal template slots |
| Display/reader, publication | Retain compiler; elevate shell | Authorized World Bible home, directory/envelope, title/anchor, search group, back/focus. Reader not author data hidden by CSS. Current save couples draft/layout with published flag; #77 A1 before release redesign |
| Archive/recovery, media | Retain | Verified atomically published backup, safe restore, archive vs scoped reader export. Preserve images, versions, pins. Recovery sibling not integrated |
| Synthetic genetics #9 | Retain optional | Accessible from family/population article when valid; no real DNA, no culture inference, never required to create person |
| Recipes #70, dashboard #78 | Defer until readiness | Context actions and fixed shell/saved views first; genuine repeated-task benefit beyond simpler alternatives; no new engine |
| Technology stack | Retain | Rust/SQLite, Tauri 2, TypeScript, SVG/Canvas2D/optional wgpu; no framework/storage migration justified |

## 3. Documents and authority audit

The primary contradiction is *navigation framing*, not persistence. Live [README](../README.md) still describes Node Studio, Canvas Studio, Display Studio as layers; [00](../Design-Docs/00-Product-Charter.md) pillar 4; [01](../Design-Docs/01-Requirements-and-Invariants.md) interaction bullets; [02](../Design-Docs/02-System-Architecture.md) architecture diagram; [06](../Design-Docs/06-Three-Layer-Entries-and-Canvas.md) section 1; [07](../Design-Docs/07-Search-Query-and-Discovery.md) studio search titles; [10](../Design-Docs/10-Delivery-Roadmap.md) Phase 4 heading and #5 title. New design and research reviews expressly non-normative, while #77 independent editorial release and #78 dashboard scope already accepted on review branch. Need explicit owner-approved wording reconciliation rather than treating proposal as authoritative or reversing existing accepted contracts. Do not silently erase historical records.

| Canonical doc | Disposition |
| --- | --- |
| README / design index | Lead with World Bible, three representations not obligatory screens, scoped reader, contextual tools; clearly distinguish main/candidate |
| 00 Charter | Revise product pillar and exemplary workflow from node-first to entry-first. Preserve mission and historical emphasis |
| 01 Requirements | Retain INV/functional IDs; reframe interaction surfaces and map existing tests. No requirement deletion |
| 02 Architecture | Keep code/storage/GPU boundary, revise diagram above native authority to unified shell and contextual graph/tools |
| 03 Domain | No semantic model rewrite; clarify node is canonical entity, presentation isn't. Current implementation notes dated |
| 04 Genealogy / institutions | Retain typed kinship, household and membership, separate organizational composition/hierarchy lens; specialist tools accessible from entry |
| 05 Events / conflicts | Retain causal/agency contracts; entry-based authoring plus optional event/conflict views |
| 06 Entries / Canvas | Main reconciliation: three representations, one author shell; add explicit semantic component mutation contract distinct from layout/binding writes. Preserve compiler, stable IDs, optional pages, template pins |
| 07 Query | Reframe workflows by task, no duplicated query engine, author/reader scopes preserved |
| 08 API/storage | Preserve transaction/version/permission; #77 already accepted future release selection, exact migration/wire still gated; no new universal component schema |
| 09 Acceptance | Keep all AT/SP/AE/WT and phase exit IDs; replace UI choreography in reviewed tests only, add entry-centric parity scenarios without waiving gates |
| 10 Roadmap | Sequence vertical authoring slices across existing phases; no new phase or premature closure |
| 11 AI instructions | Maintain mandatory preflight/governance; use consolidated World Bible value test, actual evidence, no premature feature expansion |
| 12 Decisions | Record owner-approved UX reconciliation and decision boundary; preserve D-001/D-007/D-009/D-012/D-013 and spatial/ontology/editorial ADRs; no silent supersession |
| 13 Spatial | Keep specialized model and gates; map/containment available from entry, reusable view separate from semantics |

ADR families in full annex: phase-0 platform; phase-1 persistence; phase-2 relationships/agency; phase-3 indexing, aliases, mentions/reviews, query AST, paging/control/refresh, saved queries/parameters/bindings, public search/links, contextual exploration; phase-4 entry/tree/grid-panel/template/media/encyclopedia; phase-5 parentage; cross-phase network/lineage/spatial; branch metadata; adr-007/014. All retain historical decisions and tests. Reconcile any conflicting presentation terms in a new reviewed UI decision, not retroactively rewrite evidence. [Features](../Design-Docs/Features/Features.md), [Evolutions](../Design-Docs/Architectural-Evolutions/Evolutions.md), [Studies](../Design-Docs/Research-Studies/Studies.md) keep their authority rules. Closed research does not authorize production. See annex for every file and exact blob.

Research #65/#66/#69/#71/#76 reviewed in [reassessment](../Design-Docs/Research-Studies/world-bible-research-reassessment-2026-11-22.md); one branch-only supplement was not in that review: [cartography label index](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/8e6bc390d3d7c473785fea4c683adcc6600bc79b/Design-Docs/Research-Studies/cartography-label-index/README.md). Eight paired fixtures, synthetic rectangle broadphase, 50k 14,688,929 vs 503,801 exact checks, 1548.86 vs 538.20 ms single runs; not target app SLA. Preserve source/script/results, contextual map label budget, visible accessible list, viewport culling; no standalone map engine. Earlier audits [21](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/4fa8c1b9717c192f8bf4a1a982f2ee02e5ef4503/Development-Audits/2026-11-21-state-of-development.md), [22](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/4b274360f38186ce02b41a26bd8fe12b38b0ec6f/Design-Docs/Repository-Audit-2026-11-22.md), [bug hunt](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/65771fd6c5a8bb439a99c2d2f3556bb2837af99d/Design-Docs/Repository-Audit-2026-11-22-bugs-and-performance.md) remain dated evidence. Preserve unique files (including original paths or clear provenance) in integration. Documentation snapshot status labels must not be confused with latest CI.

## 4. Findings: actual gaps and risks

| ID / priority | Evidence and issue | Resolution / verification |
| --- | --- | --- |
| WB-01 P0 | Split branches: review vs #79 diverged three commits each from 15448c4; review vs #80 diverged 17 ahead, six behind from 30b6710. Recovery separate. | Create integration candidate from exact green #80 + reviewed #79/review/recovery/fixture changes. Resolve overlapping workspace.rs/platform.spec.ts/index/workflows/acceptance docs with explicit parity checks. Run all suites on integrated head. Do not merge older schema alternative #67 |
| WB-02 P0 | #79 draft guards and expected revision/reference picker not in #80; branch capability still partial. | Preserve canLeaveEntryDrafts, save/discard, stale writes and shared picker; use existing #79 tests. Native one-context task across tools; unsupported context explicitly unavailable |
| WB-03 P0 | Recovery branch unique and green; latest #80 store.rs 2542–55 ordinary backup still create_new final path; network.rs 642–674 persisted_impact global confirmed-edge load, 676 canonical_identity broad network facts, 484 incident rows. Prior BH-01–04 reproductions documented; #75 protects migration backup but not all ordinary backup/read paths. | Reconcile recovery 3ecd5bf on final candidate, preserve migration staging and archive contracts, exact parity, 4,001 unrelated edges, 501 incident edges + redirect, source expiry/fork, no-clobber/interruption checks. No assertion that all prior fixes included |
| WB-04 P0 | #80 entries.rs:9–13 save_entry_layout(published:bool), read/selection and public index coupled; prototype directly changes text of released mock records. #77 A1 outstanding. | Explain actual current save action; compare native staged editing requirement; separate current release selection only after A1/wire/migration. Revoke current grants and verify search/backlink/cache/export. Never present prototype as real lifecycle |
| WB-05 P1 | #80 author-workspace.ts:101 projects scene from existing snapshot; entry-editor.ts:23–55 hosts editor/tree/previews inside inspector; public-reader.ts:47–63 uses workspace_public_entry(worldId,day,entryId) and replaces host. No full unified World Bible shell. | Deliver author/reader directory, page envelope, open/read/edit/back and focus; stable IDs, no full graph required, scoped title/availability, context/draft guard. Reuse current modules |
| WB-06 P1 | #80 layout.rs:35–58 supports existing text/links/tree/published refs and private tables; no structured inline authoring; restricted publication at 343ff. | Small rich-text/heading/link/citation subset only after layout/schema review. Semantic controls invoke typed commands, not layout JSON. First approved live component uses scoped original facts, no author-only NetworkTable bypass |
| WB-07 P1 | #80 entries.rs:319 search_public_pages and :486 find_text_blocks use matching blocks, prior ENC-02/09 noted per-block first 50 and anchor navigation issue. | Group authorized distinct pages, stable section anchor/revalidate, bounded continuation; do not prelimit hidden candidates or expose titles/counts. Test old high-match corpus, Unicode, revocation |
| WB-08 P1 | Universal picker in #79 only; #80 network types distinct but relationships and profiles not universal UI, layout template doesn't establish fields/facts. | Reuse compatible picker/service, filter actual verified predicate/role/endpoint capabilities; keyword disambiguation and explicit direction, date, source, scope. No fabricated unsupported relation, subtype or automatic inference |
| WB-09 P1 | Prior UX-04 and #68 partial, no cross-domain hierarchy service established. Prototype hierarchy groups direct related records by kind, not recursive/cross-nested traversal. | First actual family/household, institution and physical/admin trees as typed routes; use shared tree view only. Preserve same ID in multiple lenses; no inferred membership/authority; co-effective cycle checks on strict predicates only; no unknown geometry=0 |
| WB-10 P1 | Prototype index.html line 30 open resets mode, line 36 render replaces workspace, edit save mutates same e including released text, no dirty state. Hardcoded fixture and predicate map; no temporal author context or true layout bindings. Test covers one mock title link, direct groups, save/filter and identity-only. | Label isolated mock, improve before using as comparative evidence; actual production comparison against same commands. No production trust/security claims. User test tasks include unsaved transition, source/roles, unknown IDs, article type/templates, publication, narrow/keyboard |
| WB-11 P1 | Alder Basin expanded workflow 91b1e18 staged skip-ci; original 8a451ac green separately. Current narrow native smoke insufficient cross-domain usability. | Integrate/test expanded fixture using native Store commands and retained DB only after tests; complete actual World Bible task flow across current API and branch scope. No copied production world |
| WB-12 P1 | Normative docs still studio oriented; accepted #77/#78 documents not main; open #5 title reflects old UI. Existing #4 checkbox includes passed narrow old workflows; later phase bodies largely unchecked. | Owner review terminology/decision, update existing docs and tickets with precise gate mapping and evidence; no percentage completion; no duplicate tickets or retroactive acceptance |
| WB-13 P2 | 73 branches, 53 merged PRs, redundant historical refs, no integrated master candidate; #13 closed unmerged overlaps current startup config (default custom-protocol; loopback) but full residual patch parity not certified. | Archive naming/history; no branch deletions until unique diff/media/PR verification and owner choice. Keep historical trace. Protect main against unreviewed direct code changes if owner chooses |
| WB-14 P2 | #70/#78 approved conditional, additional research and features tempting before basic authoring. | Keep WT-01/B1 stop gates. Fixed directory + ordinary queries first. Defer speculative work; no arbitrary GIS, EAV, plugin runtime, social simulation expansion |

## 5. Branch/PR integration plan

All open PRs draft. Do **not** merge independently based only on which is green. #80 on #75, #79 on review, #67 and #73 against main. Exact #80 already contains #73 and #74 ancestry (46/23 commits behind respectively), #75 six behind, classification pagination 17 behind, Boolean 16 behind, verification fixes eight behind. #67 diverges from #80 by three spatial commits and competing migration 0037; compare accepted spatial semantics and tests to consolidated lineage, never replay migration number mechanically. #74 already closed. #13 closed unmerged; present candidate config has several intended behaviors, not proof exact patch parity.

| PR | Decision |
| --- | --- |
| #75 | Frozen, known green consolidation base. Main merge only after complete integration review |
| #80 | Preferred starting production candidate after current exact-head scale/desktop and review, not closure of all #72 gates |
| #79 | Preserve unique context/draft/reference changes, proven separate desktop; reconcile candidate overlap |
| #73 | Superseded as independent integration path; preserve history, disposition after parity and owner review |
| #67 | Superseded schema path, compare #68 behaviors and tests; do not merge separately |
| #74 (closed) | Historical ancestor, do not reopen |
| #13 (closed) | Verify current startup behavior and residual diff before archival; no independent merge |

Suggested integration branch based on #80 (only after exact matrix confirms). Integrate recovery branch changes (4 ahead/8 behind, shared 27daf6c) and regression, expanded Alder Basin (3 ahead/8 behind), #79 code/context and **review branch** approved #77/#78/research/World Bible docs, prototype separate test, branch-only supplemental audit/research and exact archived evidence. Use explicit reviewed merges or minimal patch/cherry-pick with original refs; review source and resulting diff, no force pushes. Do not blindly resolve overlapping semantic or policy code. Maintain one migration lineage and prior released schema36 migration fixture. Add no new schema merely for UI. Verify migration, recovery/fixture, exact CI both OS, actual native author workflows, scope/cancellation/scale/offline; preserve rollback to old backup and older binary refusal. Once reviewed, consolidate to main through one reviewed path and close superseded PRs with links/evidence; no action taken now. Preserve old branches until integration record establishes patch parity, including branch-only focused discovery capture (compare main blob/index). No rebase/copy that mutates original evidence.

## 6. Issue and roadmap congruence

All 20 actual issues included in annex with literal checklist counts. #1–3 closed; maintain regressions, do not reopen because UI changes. #1 11/16 and #2 4/14 boxes checked despite scoped closure: historical checklist mismatch, not automatic failure. #3 30/30. #4 54 checked, 30 unchecked, not 64% complete (nested repeated evidence and heterogeneous items). #5 0/14, #6–10 0/16 each in bodies despite substantial partial code: reconcile evidence individually, never mark whole phase complete merely from PR count. #65/#66/#69/#71 closed research; #76 open. #68 five checked, nine unchecked; #70 four checked, twelve unchecked; #72 three checked, eleven unchecked; #77/#78 each zero checked/five unchecked. Some checkboxes concern procedure not product. No status edits in audit.

| Roadmap / extended ticket | World Bible-first disposition |
| --- | --- |
| #1 phase0 platform | Retain renderer/desktop fallback; current UX still needs real saved-world tests |
| #2 phase1 persistence | Retain stable temporal store, archive. Regression migration/recovery, no UI-driven store rewrite |
| #3 phase2 relation/event vertical | Retain explicit actor/action/relationship/effect; accessible inline semantic authoring |
| #4 phase3 discovery | Critical shared directory, stable picker, search/group/anchor, typed backlinks/exploration and focused editing; complete existing gates |
| #5 phase4 entries | Central product slice. Propose wording “World Bible, Semantic Entry Authoring, Composable Canvas & Authorized Display”; keep Node and graph capability, no mandatory separate construction. Owner-approved title/body/06 reconciliation |
| #6 phase5 family/org | Contextual genealogy and typed hierarchy, individual identity, membership/title/ownership; no generic nesting semantics |
| #7 phase6 history/conflict | Contextual timeline, event article, causal/actor specialist tools; not default separate app |
| #8 phase7 policy | Critical cross-phase context and security; author/reader/perspective and historical publication distinctions; no hidden data |
| #9 phase8 genetics | Optional family/population components; later full gates, no identity inference |
| #10 phase9 release | Native install/recovery/archive/scope export/benchmarks; continuous, not after all UI |
| #65/#66 closed | Preserve original experiments, integrate supplement label study, #68 governs |
| #69 closed | Contextual exploration first, no public recipe execution |
| #71 closed | Ontology stable reusable typed core, no new arbitrary schema |
| #68 accepted | Preserve SP gates; location directory, semantic hierarchy, maps in context; optional geometry first |
| #70 accepted | Preserve WT-01; demonstrate advantage over contextual actions, finite recipes only |
| #72 accepted | Finish exact matrix and AE-01–09 including real author comparative value; no expanding taxonomy |
| #76 open | Primary UI study; RQ-01–12 reassessed; comparative author tasks and owner review pending |
| #77 accepted | High priority A1 staged release need, exact schema/wire and revocation before production |
| #78 accepted | Lower priority B1 fixed shell/saved views comparator; defer if sufficient |

## 7. One coherent delivery path (not new phases)

**Gate I — converge**: finish #80, integrate tested recovery, context #79, expanded test world and reviewed documentation, archive actual evidence. Verify exact combined candidate. No production UI rewrite before baseline.

**Gate II — reconcile product contracts**: owner review this audit and World Bible UX direction, reconcile 00/01/02/06/07/10/12 and README, #4/#5/#6/#7/#8/#68/#70/#72/#76–78 ownership. Adopt canonical label “World Bible” for primary interface, “entry editor/Canvas”, “connections”, “hierarchy”, “read” for contextual work. Do not change architectural representation terms or original acceptance criteria without reviewed decision.

**Gate III — first useful World Bible**: existing Rust scope, world/date/revision, search directory, identity-only creation, open stable author entry, author/reader navigation envelope, read/edit/back/focus, no mandatory graph. Existing editor invoked within entry context, do not rewrite components or framework. Native tasks: locate same-name person/place/org; create without article; reopen; save/discard with conflict; denied reader title/links; large-world focused selection. Test against real Northbridge and Alder Basin.

**Gate IV — semantic Canvas**: context-sensitive relation picker (Person/Family/Organization/Place/Title/Office/Asset/Concept/Event where actually supported), typed commands and provenance, explicit temporal correction, direct/derived/claim distinction; reusable semantic summaries and property controls. Distinguish editor control vs Canvas presentation, explicit canonical write vs layout. Type/profile chooses compatible presets; template changes only layout; theme only presentation. Deliver small rich-text heading/link/citation with migration compatibility, validate first safe live relation component. #77 A1 and release gate before staged release claim. No mock author data for public. Native paired comparison and actual task steps/time/errors; not old synthetic metrics.

**Gate V — nested and specialized tools**: domain-specific ancestry/household/institution/location/event hierarchy and scoped contextual graph/list, date/fork parity and bounded traversal. Geography exact/schematic map, label budgets and explicit candidate generation per #68. Genealogy/chronology and source-aware history. Reuse views across article authorship and reader only with actual scoped APIs. Do not create arbitrary taxonomy or automatic nesting truth.

**Gate VI — advanced work only on value**: #70 finite recipes after WT-01; #78 dashboards after B1; optional genetics, richer style, portability and release continuity. No broad feature escalation as a workaround for incomplete core authoring. Existing phase exits, AE/SP/WT/A/B and test IDs continue.

### Acceptance scenario that proves congruence

From the World Bible home find an individual, connect to a family/household, guild, title, location and sourced event using actual compatible predicates; compare explicit typed facts and derived paths, preserve stable ref on rename; compose biography with bound membership/lineage and authored prose, alter layout/theme without changing facts; follow nested geography and institutional views; change date/branch and re-evaluate authorized projection without changing authored prose; handle conflicting/stale write; cancel draft navigation; publish only allowed content, search/return/revoke without hidden data; reopen archive and exact IDs, facts, sources/layout pins, restorable old schema. Run on actual persisted mixed fixture, narrow window/keyboard, 1080p target and no-GPU, 1k/10k/50k bounds. Record exact head, actual UI/task outcomes and captures. No claim that this scenario already passes.

**Decision:** preserve our engine. Establish a single integrated compatible product and finish the primary entry-centered experience. Stop adding standalone studios and new scopes until demonstrable author tasks expose a specific gap. Audit records recommend changes; no production changes, branch deletion, merges, ticket closure, normative edits or final completeness certification made.
