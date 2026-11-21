# Phase 3 — Versioned FTS5 Label Index (ADR-008, partial)

**Status:** Implementation under review in Phase 3 [issue #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4).
**Baseline:** `main` at `436de9da87bad6a796eb8e675ccedba1560c7169` (2026-11-21 UTC).

## Context and constraints

The existing `Store::objects_at` reconstructs historical snapshots correctly, but its
2,000-visible / 10,000-candidate bound prevents useful general name search in larger worlds.
A plain FTS5 table keyed by `object_versions.rowid` would silently break after the
application's `VACUUM INTO` backup because ordinary SQLite rowids can change. Public/character
search is not authorized yet; the text index is **not** a publication index.

## Selected minimal design

- Migration `0008.sql` adds a disposable derived document table with an explicit integer
  primary key, stable across backup/vacuum, and external-content FTS5 indexing **only versioned
  object labels**. No authors, claims, sources or canonical object types are rewritten.
- Triggers synchronously index each new object version within its existing author-revision
  transaction. Source documents and index postings can be rebuilt atomically from
  `object_versions` via `Store::rebuild_search_index(Scope::Author)`.
- Typed `FullTextPhrase(String)` searches escape and quote the entire untrusted phrase:
  no arbitrary `MATCH` expression, wildcards or SQL operators enter the planner.
- Candidate lookup joins FTS5 to stable document identities **within the chosen world**,
  then resolves each through `resolve_object` for branch, fictional day and author
  revision. An obsolete but matching label does not make its latest correction a hit.
- Search remains author-only, returns distinct stable IDs and selected revision, and
  sorts deterministically by current label and ID. To prevent broad historical matches
  from consuming unbounded work, more than 10,000 raw matching versions produces an
  explicit `QueryBudgetExceeded` error; returned rows are capped by `QuerySpec.limit`.
- Existing exact/substring filters and their budgets remain unchanged. Node search gains
  an explicit **Full-text label phrase (indexed)** option, without altering the renderer.

## Alternatives rejected

Indexing mutable "current name" alone loses author revisions and fork history.
Searching the raw FTS5 index without resolving the historical snapshot resurrects
obsolete names and can leak private records. Using implicit rowid as a cross-table
key cannot survive `VACUUM INTO` reliably. A separate search engine or generic query
framework is unnecessary for this slice.

## Verification and remaining work

Tests exercise fresh migration and schema-7 upgrade/backfill, correction and archived
revision, future valid time, world isolation, author-scope denial, punctuation/operator
injection, identical labels, rebuild equivalence, backup/reopen and search of a narrow
phrase in a world exceeding the old 2,000-object projection bound.

This is a bounded, **label-only author index**, not full universal search. Source/claim
text and aliases, stable pagination, broader typed AST, incremental author/public
materialization, policy-safe search, indexing performance benchmarks at 1k/10k/50k,
and proposal approval remain open in [issue #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4).
Genuine development screenshots must show running software and must not be
manufactured from browser mocks or schematic renders.
