# Architectural Evolutions

**Status:** Approved governance category — 2026-11-21 local / 2026-11-22 UTC.  
**Authority:** [Owner approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/71#issuecomment-7074128490), [root ticket policy](../../README.md#6a-architectural-evolution-tickets--governed-change-to-architectural-foundations), [design authority](../README.md#document-authority).

This directory holds versioned **accepted architectural evolution specifications** for material changes to existing or approved architectural foundations, shared semantic systems and contracts. It is not the research directory, ordinary phase task list, New Feature index or an alternate normative architecture. The canonical accepted decisions, requirements and specialist contracts remain authoritative; approved evolutions explicitly reconcile them through ADRs and precise change records.

## Classification and scope boundary

- Research investigates uncertainty and evidence. Publish its study in [Research-Studies](../Research-Studies/Studies.md).
- Roadmap/accepted feature/evolution tasks implement already agreed behavior; routine fixes and refactors stay with their owning issue.
- `Architectural Evolution: Descriptive Architecture Change` proposes a material architectural delta in service of accepted product intent, requiring individual recorded owner acceptance and reconciliation.
- `New Feature: Descriptive Feature Name` proposes genuinely additional product behavior outside accepted scope; use [Features](../Features/Features.md). Split mixed work into linked proposals where necessary. No evolution may quietly expand approved scope, weaken invariants or change phase exits.

## Lifecycle

1. Audit actual source and contracts; establish evidence and alternatives, including not changing anything.
2. Open a concrete Proposed Architectural Evolution issue following the root policy. List dependencies, exact scope and objective gates.
3. Obtain explicit owner acceptance of that specific delta, with constraints or rejection/defer record. Ticket-category approval, study closure and issue creation do not equal acceptance.
4. Author versioned specification here after acceptance; reconcile changed ADRs, requirements, domain/API contracts and tests together. Record superseded decisions, preserved invariants and compatibility boundaries. No production change before reconciliation and readiness.
5. Implement dependency-ready slices through linked owning issues/PRs; use test-first migrations, backups, rollback/restore, scoped integration and regression tests. Keep real evidence and documented limitations.
6. Close only on verified agreed implementation, exact-head evidence, affected documentation and review (or explicit deferred/rejected/superseded decision, not implementation completion).

## Required evolution specification

Use descriptive lowercase hyphenated filenames. Each specification includes:

- Title, status, version/date, owner/author, approved ticket and actual acceptance record, supporting research/audit and pinned baseline.
- Problem, architectural before/after contract, why narrower existing tasks are insufficient, scope boundaries and alternatives.
- Specific affected types, reference/identity and historical semantics, interfaces, invariants, ADR/requirement changes, preserved behavior and unresolved decisions.
- Cross-system dependency and ownership map: Rust authority, storage/schema, query/search/discovery, permissions, Node/Canvas/Display, approved features, archive and performance as relevant. No independent duplicate authority.
- Migration/version-pin and compatibility plan, stable IDs/history, atomicity, backup/dry-run, rollback and recovery, old-version errors; mark nonapplicable requirements explicitly.
- Design reconciliation table listing old/new contracts and exact affected documents/decisions, documented approval for any changed normative commitment, and no implicit new scope.
- Dependency-ordered implementation slices, readiness and stop criteria, risks/alternatives and testable acceptance gates.
- Actual test/provenance/evidence links, exact commits/PRs, native observations/captures where applicable, remaining limitations and closure review.

Document status: Draft, Accepted Design, Implementing, Implemented and Verified, Deferred, Rejected, Superseded. Issue status and design/production readiness are independent. Accepted Design alone does not pass production tests. Use canonical documentation hierarchy, root governance and [engineering instructions](../11-AI-Engineering-Instructions.md). No phase renumbering or automatic reopening; scope and phase changes require explicit approval.

## Accepted evolution index

- [Universal Worldbuilding Ontology and Governed Semantic Extensibility](universal-worldbuilding-ontology-and-governed-semantic-extensibility.md) — **Accepted Design v1.0**, [#72](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72), [approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72#issuecomment-7074159822), [ADR-007](../ADRs/adr-007-governed-semantic-extensibility.md). Research #71 reviewed; readiness and AE-01–09 pending.

Research #71 is reviewed evidence for #72: [Universal worldbuilding ontology study](../Research-Studies/universal-worldbuilding-ontology.md), [issue #71](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/71). Subsequent explicit individual approval recorded above. Research closure is not production completion.

- [Independent Editorial Draft and Publication Lifecycle](independent-editorial-draft-and-publication-lifecycle.md) — **Accepted Design direction v1.0**, [#77](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/77), [approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/76#issuecomment-7079741458), [ADR-014](../ADRs/adr-014-independent-editorial-release.md). A1–A5 readiness/verification outstanding.
