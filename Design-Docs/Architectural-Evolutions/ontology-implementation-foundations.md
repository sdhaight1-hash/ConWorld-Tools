# Ontology implementation foundations — #72

Work on PR #73 starts from schema 36 and ADR-007. This is an implementation checklist, not a new normative design or passed acceptance gates.

| Gate | Foundation and dependency | Required regression evidence |
|---|---|---|
| AE-01 | Existing native author-task baseline; compare completed Concept baseline against curated profiles before expanding scope | Exact-head CI, real author tasks, explicit stop rule |
| AE-02 | Existing registry; discriminated SemanticRef / FactRef; separate network and parentage authority | Family collision, wrong world/kind, fork, rename, withdrawn facts, public denial |
| AE-03 | Canonical Concept first; governed definitions and descriptor pins | Old unclassified objects unchanged; invalid kind/version rejected |
| AE-04 | Existing typed payload validators; approved fields only | Missing vs unknown vs zero, unit pins, historical correction, atomic writes |
| AE-05 | Existing query AST, query control, saved replay and discovery review | Historical parity, bounded results, cancellation, no hidden facets |
| AE-06 | Existing Node author commands and authorized Canvas/Display projections | End-to-end edits, binding parity, reauthorization, accessibility |
| AE-07 | Additive migrations coordinated with #67; existing archive authority | Schema-36 and current restore, stable IDs, rollback, old binary rejection |
| AE-08 | Existing CI and native smoke harness | Dense/sparse scale, Windows/offline fallback, genuine captures |
| AE-09 | Accepted ADR-007 and existing phase/feature owners | Traceability, independent gates, owner review |

## First executable foundation

The new semantic reference module preserves UUIDs and distinguishes network and parentage even if IDs coincide. Its object-only adapter delegates to Store::get_object; it denies raw non-author reads before reference lookup. Fact resolution is not implemented by this slice: use family-specific indexed point reads within one historical snapshot, not bounded adjacency scans. Do not expose raw fact data through Tauri or published views. No schema changes, new descriptor persistence, or feature-gate completion in this commit.

Tests added for reference family discrimination, serialization, scope and wrong-world handling; execution pending CI. No visual change requiring a capture.

## AE-02 historical point reads

Prior reference-vocabulary commit `9086041` passed Windows and Ubuntu build/test and desktop verification. This slice adds author-only point resolution of network and parentage facts through the existing historical metadata CTE, with original source branch, author revision, rationale and typed payload. No migration or duplicate fact table. The existing branch-local regression now checks original, withdrawn, sibling and parent fact lookups, family mismatch and public denial. New checks await CI; scoped publication and annotation/backlink integration are still open. No visual changes.

## Definition-pin validation foundation

AE-03 adds validation of exact existing Definition versions within a historical author view. Wrong kind, stale version and unauthorized reads fail closed. This is deliberately not descriptor persistence or historical classification assignment; pinned historical replay must supply the original view. The Concept registry migration overlaps spatial PR #67's schema 37 and remains to be reconciled. Regression tests cover correction and prior-revision replay. CI pending; no visual changes.

## Canonical Concept vertical slice

Adds Concept to the canonical object registry, identity creation, ghost proposal vocabulary, historical read/write, typed search and desktop authoring choices. Migration 37 is provisional on this branch; spatial PR #67 must be reconciled before either migration is released together (renumber the spatial migration and preserve Concept in both registry constraints). No existing IDs are rewritten. Tests cover historical visibility, kind search, scope and FK integrity. CI and actual desktop screenshots remain pending. This does not complete governed classification persistence.

## Curated classification persistence

Schema 38 adds author-only temporal classification assignments and branch-local withdrawals. The first four curated profiles are Culture, Language, Material and Technology, all compatible with Concept only. Assignments retain an exact Tag Definition version and source, reject overlapping same-profile assignments, and preserve the original author revision and branch. Reads are bounded, historical and scoped; corrected labels do not replace the recorded pin. No inferred membership, general field runtime or published projection. The held spatial migration must be renumbered after these migrations and retain Concept. CI pending; no UI capture applicable to this store slice.

## Classification query integration

AE-05: the existing typed query planner now accepts curated profile predicates, including boolean composition, persisted predicate encoding and typed parameter binding. The profile vocabulary is fixed, while each assignment retains its descriptor version. Queries remain author-only, reuse the bounded classification service and honor query control. Profile filtering now uses separate SQL shapes for subject/profile indexes. Schema 38 migration is repeatable for existing downgrade-simulation regression fixtures. Desktop parameter controls updated, including composite profile selection. CI and native UI captures pending.

## Native command and studio integration

The existing branch-aware author studio exposes record, inspect and withdraw classification commands, with explicit Concept, Tag Definition, source, profile and dates. Rust resolves the descriptor in the requested historical context, checks the pin inside the write transaction and returns original recorded provenance when inspecting. No raw classification command is registered in the public reader. Branch changes clear stale results, and late responses are discarded. Native GUI verification and screenshots remain outstanding.

## Role-qualified Concept relations

Schema 39 extends canonical network facts with CultureLanguage (Culture -> Language, documented_language) and WorkingMaterial (Technology -> Material, working_medium). Both require source-backed active compatible classifications at the effective start date, checked in the same write transaction. They are independent authored facts, not inferred membership, speech proficiency, manufacturing or simulation. Later classification corrections do not rewrite the existing relation; the recorded author revision permits historical explanation. Existing network UUIDs, withdrawals and validators are retained. Desktop network authoring exposes the two typed relation choices; regression covers wrong profiles, roles and historical semantic lookup. Spatial #67 remains held and must preserve the expanded network vocabulary when reconciled.

## Stable classification references and fact backlinks

AE-02 now includes Classification alongside Network and Parentage as a discriminated fact family. Exact point and list reads share the same classification payload decoder and endpoint visibility checks. Author-only fact backlinks traverse existing network endpoints/source, parentage endpoints, and classification subject/descriptor/source in one historical transaction with an explicit work bound. Withdrawn and invisible candidates never appear in results; no new fact copies. Tests exercise source/descriptor backlinks, exact UUID and branch withdrawal. Annotation authoring remains separate and no public projection is implied.

## Archive, indexing and Canvas replay

Schema 40 adds source/descriptor backlink indexes. Archive inspection and migrations check classification endpoint kinds, pinned Tag payload and chronological author revisions, beyond FK integrity. The classification lifecycle regression now persists a typed saved query, binds it through existing Canvas author controls, checks an empty public projection, and backs up/restores the same IDs, facts and page output on a separate path. Fixed the CI-reported rustfmt differences in the preceding two slices. Verification pending at the new head; no new scope or duplicate storage.

## Desktop fact navigation and capability reconciliation

Registered the classification and semantic reference commands in the Tauri handler, generated app manifest and main-window capability together (fixing the earlier launch-config CI failure). The existing author network studio now supports evidence/identity backlinks and exact family/UUID resolution, preserving recorded author rationale and source. Branch changes clear results; no public reader projection. Added Playwright frontend command-contract coverage; it is mock-native, not genuine desktop runtime evidence. CI remains required.

## Descriptor identity discovery

Verified baseline cd7b917: all six Windows/Ubuntu CI checks passed. Added descriptor-ID classification predicates to the shared query planner, saved query wire shape and runtime parameter contract; existing assignments retain their historical version pins even when the definition label changes. Author-only descriptor lookup uses the normalized index and the same visibility/budget rules as profile lookup. Desktop combined-query controls expose the stable descriptor ID. This is metadata categorization, not a new arbitrary field editor.

Canvas parameter editing now also supports stable descriptor references and curated profile selectors using the same typed saved-query contract, without exposing author metadata in published components.

## Combined spatial and ontology candidate (held)

The separate work/ontology-spatial-reconciliation branch preserves the original #67 containment validators, Place identities and spatial tests while retaining Concept, profile predicates, and source-backed semantic relations. Renumbered spatial migration 0041 preserves both object/ghost registries and network kinds plus the source backlink index. This is not a release or approval of SP/WT gates. Unreleased incompatible schema-37 spatial databases must not be opened with the ontology schema-37 binary.

Schema 42 adds a dedicated source-backed site-area measurement: integer hectare or square-metre reading, or explicit unknown distinct from missing and zero. Units have fixed meanings and are never automatically converted. The Tag definition's original version is pinned; corrections atomically withdraw the old fact only in the current branch and retain its identity and prior revision; real dated changes use nonoverlapping intervals. The canonical semantic resolver, source/descriptor backlinks and archive checks consume the same facts. Tests cover validation, correction, unknown, zero, history, public denial and portable archive replay. No measurement infers geometry or containment. Combined candidate CI pending.

## Native semantic scale harness

The new semantic_workload example creates real canonical Concept identities and source-backed classification facts through the Rust Store. At 1k/10k/50k it exercises sparse (1%) and dense (99%) profile search, explicit truncation/budget rejection, stable fact point lookup, subject backlinks and cancellation; reports observed setup and repeated read times, plus Linux process high-water RSS when available. The Rust CI workflow runs the workloads on Windows and Ubuntu. This is a native in-memory benchmark, not a human desktop workflow or measured disk-write benchmark. Actual results must be read from CI before AE-08 can pass.

## Site area author and query integration

Added native measurement commands with explicit source, descriptor and interval, a same-interval correction path, historical inspection, stable reference resolution and typed saved area predicates (unknown or minimum in the exact authored unit). Area predicates remain fixed in a saved definition; no unapproved runtime conversion. Desktop inspector and combined search use these existing author-only surfaces. CI and native screenshots remain outstanding. Draft PR #74 is held against #73; #67 remains separate.

## Release lineage guard

Schema 37 was assigned independently by #67 and #73 before release. Store open and standalone archive inspection now reject the incompatible spatial registry at versions 37–40 before migration, including empty development databases. The regression reconstructs that registry and confirms no schema version mutation. Released schema 36 remains the supported migration baseline; incompatible experimental databases require their original branch or a verified schema-36 backup. The held combined candidate must inherit this guard while advancing to schemas 41/42.

## Schema-36 migration and failure rollback regression

Added a representative reconstructed schema-36 fixture using the original 0032 registry DDL and 0035 network DDL while preserving seeded world, person, source, family-house, membership and accepted-parentage records. Tests inspect the old archive, restore to a separate destination, compare exact historical fact UUIDs/payloads and person version, verify no classification backfill, and demonstrate that a deliberately orphaned object fails migration without changing the old schema version or creating the new tables. This is a reconstructed fixture, not a separately executed old binary or a production archive; exact-head CI remains required.

## Isolated native desktop ontology smoke

The combined candidate adds opt-in --ontology-smoke to the existing native Windows WebView smoke workflow. Its independent in-memory world uses fixed source, descriptor, Concept and Place fixtures, never the author's normal database. It exercises real author studio form submissions and Rust IPC for culture/language, two technology Concepts sharing a world-specific descriptor and material identity, known zero, same-interval correction, separate dated unknown, historical replay, typed query and source backlink rendering. Success is reported by the existing native smoke report; failure is a nonzero exit. This is automated interaction (not a human timing study). Actual CI result and chronological native screenshots remain unverified.

## Fact annotation and descriptor lifecycle closure candidate

Combined schema 43 adds an author-only annotation link targeting a stable family/UUID and an existing sourced Claim (not another copy of its statement), with revision/date/branch bounds and visibility checks. It also adds dated branch-local retirement of Tag descriptors; new assignments or measurements using retired descriptors fail, existing pins and facts remain readable. Desktop controls link and inspect annotations and explicitly confirm retirement. Negative regression covers wrong world/family/kind, duplicate links, private scope, historical reads, branch-local retirement, co-classification and archive replay. Original claims, fact IDs, specialized validators and reader projections are unchanged. Exact-head CI and actual native capture remain required. This migration belongs to the combined schema-43 candidate, not the separate schema-40 branch.

Fact annotations now also resolve in reverse through the original claim identity, deduplicating against direct fact backlinks and reauthorizing both the claim source and target fact. The desktop backlink row provides a keyboard-operable selection action for annotation; mock-native command regression includes annotation and retirement controls. No native capture is claimed.

The desktop regression writes chronological UTC annotation form/list captures to Development-Screenshots and uploads them as the phase-03-semantic-annotation artifact. These are Playwright browser captures with synthetic native IPC, not evidence that the actual Tauri/SQLite runtime was exercised. Do not index or cite any image until it exists and is checked.

## AE-05/06 inspection invalidation

The network/semantic inspector now invalidates stale results and action controls on focus edits, branch switches, new reads and writes. Each read has a generation token and historical context; late replies cannot overwrite a newer result. Pinned historical annotation inspection remains enabled while mutation is disabled. Added deferred mock-native response regression. Exact-head CI pending.

AE-05 regression now checks area point/query parity through a branch-local correction, inherited and main history, cancellation followed by successful read, unsupported query version and both public and character scope denial. Archive replay verifies the branch correction's query output. Annotation selection and branch-list responses also use invalidation. New tests await CI.

## AE-06 compatible reference selection

A single author-only semantic picker uses the existing controlled indexed query service with the exact selected historical branch, type and label phrase. It filters Tag definitions and sourced claims, exposes bounded/truncated results, supports cancellation and keyboard-operable selection, and fills the original authoring or inspection field without modifying canonical data. Writes retain independent atomic validation. Regression exercises picking a compatible Concept instead of manually copying UUIDs while a previous inspection is pending. CI pending.

## AE-08 measured native author task report

The isolated Windows desktop smoke now uses the actual compatible selector through native controlled IPC, checks selected stable ID and focus transfer, then records elapsed milliseconds and committed form submissions separately for culture/language, reusable setting-specific category/shared material, and historical measurement/evidence tasks. The Rust smoke gate requires all three actual observations and expected write counts, not merely a frontend boolean. These are automated native workflow timings, not comparative human usability timings or completed measurements until CI reports them.

## Native Windows capture gate

The existing explicit ontology smoke mode now supports an opt-in bounded capture handshake with its Windows launcher. After the actual author task checks succeed, the WebView scrolls to the verified evidence view and waits for paint; the launcher identifies and foregrounds that exact process window, verifies its desktop bounds, saves a UTC-named screenshot under Development-Screenshots and acknowledges it. The isolated smoke process fails rather than reporting capture success if the window is unavailable or acknowledgment times out. Normal app launches have no capture behavior. Windows CI uploads the resulting artifact; visual inspection and indexing are required before claiming actual evidence. No capture generated or verified in this implementation commit.

## AE-01 comparative execution and AE-08 workload observability

The combined candidate now runs the original schema-36 testimony baseline and an automated A/B/C comparison in actual Rust Store worlds with identical seed identities and source. A uses named canonical objects and sourced claims; B adds curated profiles, reusable descriptors and protected typed relations; C alone adds structured area values and historical correction. Each of the three tasks checks history and evidence and reports elapsed microseconds, actual author revision increments, structured capability and invalid-command rejection. This measures command/API effort, not human UI time. C does not expand the culture/language or novel category tasks beyond B. The comparison must execute successfully before recording observed benefit or accepting AE-01.

Scale workloads report incremental fact setup progress, dense pagination parity at 1k and explicit budget rejection at 10k/50k, cancellation and deadline checks. Each scale has its own CI watchdog (45 minutes is infrastructure protection, not an app SLA), while original focused workloads remain. CI concurrency cancels superseded runs on the same workflow and ref only. No prior or new benchmark completion is inferred from the harness.

## Comparative discriminating category and rename checks

The earlier A/B/C harness assigned the same local technology descriptor to both B and C, so it could not distinguish their benefits. B now uses the curated technology profile with a generic descriptor; C uses the world-specific drive descriptor. Both receive the same unrelated technology control and share a single material ID. The comparison reports broad candidate counts and exact category matches separately. It also performs a language-name correction and a material-name correction, checking original historical names, preserved relationship IDs/payloads and current query results. AE-02 includes a separate stable-reference rename regression. These are newly committed executable checks, not observed results. B remains sufficient for the culture/language task; the category specificity and numeric area comparison must actually run before further extension is justified.
