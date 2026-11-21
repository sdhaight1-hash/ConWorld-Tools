# Phase 3 — Bound indexed candidates to the historical branch lineage

**Status:** Draft stacked on #38; tracks #4. No schema change.

The previous FTS and event participation candidate queries could consume their 10,000-candidate budget with versions recorded after the requested author revision. All four indexed query families (label/claim FTS, participation, paths, backlinks) could include unrelated branch versions and post-fork parent edits before authoritative resolution.

A fixed recursive SQLite CTE follows the selected branch and each immutable inherited revision, with the same world and recorded revision bound as ordinary parameters. The SQL candidate selects now apply the matching branch/revision ceiling before LIMIT; no user SQL is accepted. This is a conservative candidate superset, not authoritative valid-date or override resolution. Existing Rust version checks, visibility, deterministic ordering and hard candidate budgets still apply. Alias and approved mention metadata retain their existing main-branch restrictions.

Regression creates 10,001 matching main-line records after a fork: the earlier main snapshot and pinned child must each return the one inherited identity, the genuinely oversized current main view must fail explicitly, and rebuild must preserve the child result. Existing path, participation, backlink and claim search tests cover their temporal semantics. Exact-head CI required.
