# Phase 3 — Materialized mention-name prefix indexes

Status: merged into main in PR #44; exact development head `9795ec7f` verified by [persistence CI](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940354929) and [desktop CI](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/44940355016) on Linux and Windows. Scoped implementation; roadmap acceptance remains open.

Use SQLite expression indexes on the first three Unicode code points of canonical person labels and authored aliases, ASCII-lowercased by SQLite. The indexes are materialized by SQLite itself and maintained transactionally when a canonical version is inserted; no secondary writer, trigger, duplicate name table or asynchronous consistency gap. Rebuild explicitly reindexes both. A claim produces conservative prefix keys at every supported name-start boundary, including shorter keys, punctuation, whitespace and TEXT's NUL truncation behavior. Bind keys via JSON1, never interpolated SQL. Candidate query uses historical branch/revision ceilings and active, dated, latest main-line alias versions; canonical resolution and the original exact byte-span matcher remain authoritative. Case-insensitive ASCII and exact-case non-ASCII semantics are unchanged.

The bounded candidate set is per claim, not the entire dictionary. Existing collision grouping, deduplication, review resolution and cancellation stay unchanged. Historical obsolete name prefixes can still consume the conservative raw budget; no claim of unlimited scale or background candidate materialization. Name edits still conservatively invalidate all preview sources; the existing per-source delta plan remains the consumer boundary.

Regression: >10,000 nonmatching people, ASCII case, one-character name, non-ASCII exact case, alias collision/expiration, rename, historical replay, reindex and schema-20 upgrade. Existing mention review and fork fixtures remain required; exact-head CI passed.

Consolidation: see [unified baseline and remaining gates](../Consolidation-2026-11-21.md). Earlier pending-development statements in this record describe the implementation chronology, not the current CI status.
