# 11 — AI Architect and Coding Agent Instructions

**Status:** Authoritative execution brief for AI-assisted implementation.

## Mission

Implement the ConWorld Tools / Living History Atlas architecture in this repository, following [Design-Docs/README](README.md) and the numbered GitHub **Roadmap Phase N** issues. See the [Phase Zero verification log](Phase-0-Verification.md) and current tests for implementation and runtime evidence.

## Mandatory preflight

1. Audit HEAD, tree, branches, open issues, existing tests, migrations and this documentation **before proposing commands for the user**. Avoid assumptions about prior work.
2. Read [01](01-Requirements-and-Invariants.md), [02](02-System-Architecture.md), [03](03-Domain-and-Temporal-Model.md), [08](08-Storage-and-API-Contracts.md), [09](09-Acceptance-and-Verification.md), [10](10-Delivery-Roadmap.md) and relevant specialist docs.
3. Identify active Roadmap Phase ticket and its parent dependencies. If no phase is agreed, begin at Phase 0. Avoid implementing feature families in the wrong dependency order.
4. State assumptions, constraints and unresolved ADRs; test the least complex architecture before adding complexity.
5. Plan small, traceable changes and confirm expected runnable checks without relying on imagined results.

## Architectural non-negotiables

- Rust domain/persistence/query engine owns world state; HTML/TypeScript/Tauri handles editor UI; wgpu handles *authorized view scenes* only. Never put canonical semantics in shader/React state or let a renderer directly mutate SQLite.
- World identity is stable and type-checked. Typed relationships, memberships, events, actors, effects, claims and conflicts are not interchangeable free-text tags.
- Historical valid time, recorded revision and alternate branch are distinct. Corrections keep author audit history.
- Claims, proposals and rumors do not silently become canonical relationships.
- Every data view receives ViewContext; no default author privilege. Visibility must be applied **before** results, caches, counts, backlinks, graphics scenes or exported HTML are materialized.
- Search/query is shared across Node, Canvas and Display; queries are versioned typed ASTs with bounded depth/cost. No raw SQL or arbitrary JS from author content.
- Canvas primitives control layout. Components render typed world data/queries; page edits never fork canonical people/events. Display compiles a safe projection.
- Genetics may be optional, but when implemented it is deterministic synthetic inheritance, clearly labeled and never a biological claim about real users or a substitute for cultural identity.
- Do not create independent, mismatched repositories or duplicate per-studio persistence.
- Record event/action/effect links for asserted causal explanations; do not infer causality merely from event ordering.

## Work loop

For each ticket:
1. **Inspect:** repo state, actual code, related design-doc links and failing/pending checks.
2. **Decide:** write/update necessary ADR before changing architecture; list assumptions and what tests prove them.
3. **Test-first:** translate ticket acceptance criteria into deterministic unit/integration/e2e tests, using Northbridge fixture.
4. **Implement vertically:** minimum Rust data and API changes, Tauri/HTML UI as required, renderer only if meaningful.
5. **Verify:** execute applicable commands; report passed/failed/not-run precisely. Build and run actual app when UI claims need runtime observation; capture logs, screenshots and environment.
6. **Review:** semantic drift, time/branch/perspective leakage, performance budgets, migration data loss and security bypasses.
7. **Document:** edit one canonical spec or ADR, update ticket checklist and evidence, link commits/PRs and remaining risks.
8. **Continue:** take next dependency-ready ticket; do not mark incomplete placeholder surfaces "done."

## Repository organization proposal

~~~text
/Design-Docs/                 canonical specs and decisions
/Design-Docs/ADRs/             accepted/rejected architecture decisions
/crates/atlas-types/
/crates/atlas-domain/
/crates/atlas-store/
/crates/atlas-query/
/crates/atlas-app/
/crates/atlas-canvas/
/crates/atlas-renderer/
/apps/desktop/                 Tauri + web UI
/tests/fixtures/northbridge/   canonical cross-layer acceptance fixture
/tools/                        optional dev utilities
~~~

Do not create all crates without a purpose; start with modular Rust source and split when boundaries are validated. Keep migrations, tests and example worlds in source control. Never commit generated binaries or unlicensed assets.

## Preferred Rust/API practices

- Newtypes for IDs and intervals; no stringly typed IDs or unwrap/panic on user data.
- Parameterized prepared SQL; transactional commands; typed structured errors; foreign_keys=ON on every connection.
- Versioned serialized query AST/layout AST; capability-limited DTOs; explicit schema migrations.
- Deterministic seeds with generator algorithm version; proper invalidation for derived data.
- Property/invariant tests for cycles, branch overlays, visibility/permission narrowing and import/export.
- Bound queries, cancellation, pagination and graph scene budgets.
- Preserve accessible HTML interfaces and non-WebGPU rendering fallback.

## Coding-agent deliverables

For each phase produce: design/ADR delta; implementation task list; actual commits/PR; test names + commands/results; migration notes; example fixture changes; runtime observation for UI; known limitations; issue references; next dependency. Ask only when truly blocked by missing credentials, private assets, product decisions impossible to safely assume or hardware-only reproduction.

## Red flags

- "Implemented" with only screenshots or mock data.
- A single global JSON blob storing every domain fact without typed validation.
- Fixed "current relationship" data overriding historical states.
- Secret displayed in raw component props but hidden with CSS.
- A published page executing an author-scoped saved query.
- An AI-proposed guild link directly becoming a membership.
- Genetics automatically determining someone's ethnicity/religion/personality.
- A relationship biography hallucinating causes.
- wgpu specified without a tested WebView/WebGPU boundary.
- "Full system" plans without a first working vertical slice.

## Stop conditions

If a change violates an accepted invariant, pause only the affected change; document the conflict/ADR and continue independent safe tasks. When testing requires unavailable user machine observations, provide exact minimum reproduction request **after checking existing repository logs and context**. Do not claim to have observed the Windows desktop app if only SQLite or library tests ran.
