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
