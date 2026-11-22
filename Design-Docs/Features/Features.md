# Post-Roadmap Feature Specifications

This directory contains design specifications for **new capabilities that fall outside the original ConWorld Tools roadmap**. It must not become a duplicate directory for work already defined in existing Roadmap Phases or core design documents.

**Governing instructions:** [Root README — AI Systems Instruction (Lola) and ticket-creation policy](../../README.md) · [Original roadmap](../10-Delivery-Roadmap.md) · [Product charter](../00-Product-Charter.md) · [Requirements](../01-Requirements-and-Invariants.md) · [Architecture decisions](../12-Decisions-and-Reconciliation.md).

## From proposal to feature design

1. **Check current scope first.** Inspect the repository, design documents, open/closed GitHub issues, and existing feature/research studies. If the idea is already specified by a phase, **use that Roadmap Phase issue** rather than filing a new feature. An issue about unverified feasibility may instead be a `Research:` proposal.
2. **Propose the actual out-of-roadmap addition.** Open a GitHub issue titled `New Feature: Descriptive Feature Name`, following the [root README new-feature rules](../../README.md#6-new-feature-tickets--strictly-outside-the-original-roadmap). Explain the user value, why it is outside existing scope, alternatives, dependencies, source links, architecture impact, expected behavior, risks, and measurable acceptance tests.
3. **Record explicit scope approval.** A `New Feature:` issue is **a proposal**, not a directive to implement. Its status is `Proposed`, `Under Review`, `Accepted`, `Deferred`, or `Rejected`. Do not treat AI enthusiasm, the creation of the issue, or research recommendations as implicit acceptance.
4. **Author a feature specification after acceptance.** Place a Markdown design document in this directory and link it to the accepted issue; link the issue back to its document. The spec becomes the feature's design contract, consistent with the existing architecture and accepted ADRs.
5. **Implement only against agreed acceptance criteria.** Track dependency-ready implementation tasks and validated test results via linked issues/PRs; reconcile affected core docs or ADRs deliberately rather than silently rewriting the original roadmap.
6. **Review and close only on evidence.** Feature completion requires the agreed behavior, migrations, UI integration, tests, permission checks, documentation, and runtime validation as applicable. An accepted design alone is **not** a completed implementation.

Material changes to accepted shared architecture in support of existing capabilities belong to the separately approved [Architectural Evolution lifecycle](../Architectural-Evolutions/Evolutions.md), not a disguised New Feature. Mixed proposals must distinguish the architectural delta from genuinely additional product scope, which still requires New Feature approval.

An AI acting as **Lola** follows the root README's think-before-coding, simplicity-first, surgical-change, and verifiable-goals standards. Push back on features that can be handled by existing capabilities or simpler additions.

## Required feature specification

A new feature design SHOULD include:

- **Title, status, version/date, owner/author**, link to the `New Feature:` issue and any supporting `Research:` issues or completed studies.
- **Problem, user value, scope and non-goals**, including the explicit justification for why it is not part of the existing roadmap.
- **Acceptance decision and constraints**, including the approving decision or issue comment, dependencies and affected accepted invariants/ADRs.
- **User stories and UX behavior** with representative workflows, empty/error states and accessibility requirements.
- **Domain architecture impact** — objects, relationships, events/actions, genealogy, source/provenance, historical time, branches, and knowledge/permissions where relevant.
- **Three-layer integration** — what appears in Node Studio, what Canvas layout primitives/components or bindings are needed, and what Display Studio may reveal or hide.
- **Search and discovery integration** — typed queries, backlinks, keyword/image references, suggested nodes and index/cache invalidation as needed.
- **Technical design** — Rust module/API changes, HTML/CSS/TypeScript UI, wgpu visualization only if justified, SQLite migrations, storage/export compatibility, privacy and performance implications.
- **Alternatives and trade-offs** — including reusing an existing system or not implementing the feature.
- **Implementation slices, dependencies, risks and migration/rollback plan**, with explicit boundaries rather than speculative extensibility.
- **Verifiable acceptance criteria** — unit, integration, end-to-end, security and performance tests suited to the change.
- **Source references and cross-links** to canonical design documents, research findings, ADRs, PRs, and related tickets.

## Naming and document control

- Use a descriptive lowercase hyphenated filename, e.g. `collaborative-world-editing.md`.
- Declare status as `Draft`, `Accepted Design`, `Implementing`, `Implemented and Verified`, `Deferred`, or `Superseded`. Document which claims are still assumptions.
- Never claim that a feature is implemented because its design document exists.
- An accepted feature does **not** automatically add or renumber a `Roadmap Phase`. Any roadmap change needs an explicit approval and updated dependency analysis.
- If research is needed to decide feasibility, create a properly sourced `Research:` ticket and put the eventual study in [Research-Studies](../Research-Studies/), not in this directory.
- Add approved feature specifications to the index below, linking their actual issue and current design file.

## Feature index

- [NestedLocations and Procedural Vector Cartography](nested-locations-and-procedural-vector-cartography.md) — **Accepted Design**, [New Feature #68](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/68), [owner approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/68#issuecomment-7073427305). Research #65/#66; dependency-gated implementation, verification pending; draft #67 held for review.
- [Composable Worldbuilding Transforms and Exploration Workflows](composable-worldbuilding-transforms-and-exploration-workflows.md) — **Accepted Design**, [New Feature #70](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70), [owner approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70#issuecomment-7073776197). Research #69; mandatory readiness Gate A, implementation and WT-01–10 verification pending.
- [Configurable World Management Workspaces and Dashboards](configurable-world-management-workspaces-and-dashboards.md) — **Accepted Design v1.0**, [#78](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/78), [approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/76#issuecomment-7079741458). B1–B5 readiness/verification outstanding.
