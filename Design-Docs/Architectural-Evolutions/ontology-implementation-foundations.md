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
