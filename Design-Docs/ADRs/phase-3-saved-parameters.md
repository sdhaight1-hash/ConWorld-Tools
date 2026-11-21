# Phase 3 — Typed saved query parameters

Status: draft stacked on #40; tracks #4. Schema 19.

An author may opt into parameters for every leaf of a supported saved filter. The persisted literal AST remains the default definition. Each version has optional typed declarations (text, object kind, or source reference), named deterministically in pre-order by operator and leaf number. Defaults derive from the normalized AST. Legacy definitions have no declarations and are not implicitly made mutable. A revised definition may explicitly remove parameters.

Storage writes declarations transactionally with the immutable version. Reads verify exact correspondence to the AST. Binding checks author scope, world, recorded revision, names/duplicates/value types, source world and ordinary AST limits. Missing values use defaults. No operator, SQL fragment, context, or output shape may be supplied as a parameter. Binding materializes an ordinary QuerySpec so existing execution, deadlines, candidate budgets and context-bound cursors apply. No query results are persisted.

Regression covers defaults, override, duplicate/unknown/wrong type, scope/revision/world checks, historical definition replay after literal-only revision and reopening. Native/desktop integration to follow. No public query claim.
