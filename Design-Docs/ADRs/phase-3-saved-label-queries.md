# Phase 3 — Durable Saved Label Queries

**Status:** Partial saved-query implementation; tracks [Roadmap #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4). Built on the merged [FTS5 index #22](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/22).

## Contract and scope

A saved query is **a versioned definition, not a saved result set**. A stable UUID is
world-scoped in `saved_queries` and each new or corrected name/filter is an append-only
`saved_query_versions` row, backed by an ordinary `author_revisions` command and
stale-revision check. Query IDs are distinct from canonical object IDs. The saved
definition has no independent truth, world state or publication permissions.

Migration `0009.sql` saves a strictly typed schema-1 subset:
`LabelContains`, `ExactLabel` and `FullTextPhrase` with name, filter text, and
1–500 result limit. There is no freeform SQL, raw MATCH expression or JavaScript.
The world/author revision is required for every read. Saved definitions are currently
**local author-only metadata** in the main-branch audit stream; retrieving a saved
definition at an older author revision returns the version known then.

When a user runs a saved definition, the normal `Store::execute_query` still
receives the **currently selected** world/branch/fictional day/author revision/scope,
performing ordinary historical resolution and authorization checks each time.
No query output, permissions, dates or pointers are cached in the saved record.

## Acceptance and non-goals

Test the stable ID through rename/re-save, prior recorded revision, reopening,
cross-world refusal, author-scope enforcement, invalid typed filter, expected
revision conflict, duplicate command, and migration from schema 8. A desktop
test uses a mock-native Tauri IPC bridge for the form and replay, not a claim
about live Windows search interaction.

The following remain open in Phase 3: parameterized saved queries and type
descriptors, broader typed AST and graph/event predicates, per-binding dependency
invalidations, Canvas/Display bindings, public search policies, and pagination.
No genuine development screenshot is claimed without a running native capture.
