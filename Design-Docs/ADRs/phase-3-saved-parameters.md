# Phase 3 — Typed saved query parameters

Status: merged into main in PR #41; exact development head `3f8e82ac` verified by [persistence CI](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940145779) and [desktop CI](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940145843) on Linux and Windows. Scoped implementation; roadmap acceptance remains open.

An author may opt into parameters for every leaf of a supported saved filter. The persisted literal AST remains the default definition. Each version has optional typed declarations (text, object kind, or source reference), named deterministically in pre-order by operator and leaf number. Defaults derive from the normalized AST. Legacy definitions have no declarations and are not implicitly made mutable. A revised definition may explicitly remove parameters.

Storage writes declarations transactionally with the immutable version. Reads verify exact correspondence to the AST. Binding checks author scope, world, recorded revision, names/duplicates/value types, source world and ordinary AST limits. Missing values use defaults. No operator, SQL fragment, context, or output shape may be supplied as a parameter. Binding materializes an ordinary QuerySpec so existing execution, deadlines, candidate budgets and context-bound cursors apply. No query results are persisted.

Regression covers defaults, override, duplicate/unknown/wrong type, scope/revision/world checks, historical definition replay after literal-only revision and reopening. Native/desktop integration is described below. No public query claim.

## Desktop integration

The author explicitly opts into typed values on save. Selecting a parameterized definition displays typed default controls; replay sends the saved identity, pinned revision and typed values to a native command. The native worker resolves the historical definition from storage before binding, rather than accepting a replacement AST. Refresh recreates controls from persisted defaults; unsaved overrides are not silently saved. The existing cursor and cancellation paths are reused. Synthetic browser regression covers save, override, replay by ID and refresh/historical invalidation. Native and publication verification remain separate.

Consolidation: see [unified baseline and remaining gates](../Consolidation-2026-11-21.md). Earlier pending-development statements in this record describe the implementation chronology, not the current CI status.
