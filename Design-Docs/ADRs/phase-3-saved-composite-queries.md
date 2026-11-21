# Phase 3 — Versioned Saved Composite Predicates

**Status:** Partial QRY-02; tracks [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4) after merged [PR #28](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/28). Audit baseline `ec132b6f25c90b4dbf932101ab8e66b1fb7b8852` including README, specs, issue/PRs, CI, migrations/tests and screenshot policy.

## Decision

Migration `0012.sql` extends the existing append-only `saved_query_versions` allowlist to typed `composite` predicates and increases only their text allowance to 4096 bytes; stable saved IDs, author audit revisions and legacy label/claim-text definitions remain intact. No query results, author datasets, or separate studio caches are persisted. Composite predicates serialize through a **shared Rust-owned, strictly tagged serde DTO** with named variants (`contains`, `exact`, `object_kind`, `has_source`, `and`, `or`, `not`) and allowlisted object kinds.

Both the direct Tauri query path and saved-query decode use the same DTO conversion into `QueryFilter`. Every save validates world-scoped source references and the full tree with the same 4-depth/32-node/8-child budget as execution; bad input rolls back without changing author revision. Every replay runs the same Rust author-only engine under the caller's **selected world/branch/day/author revision** and current historical data. Saved sources resolve by stable ID and do not imply evidence or relationships. Public/character replay is refused.

Node Studio offers **Save combined query** using the existing named-search and revision workflow, and **Run saved search** can replay the exact stored AST without reconstructing it from the current form. Existing legacy saved search controls still work and old history remains selectable. Pinned past author revisions remain read-only.

## Verification and limitations

Rust tests must cover schema-11 migration and older label definitions, composite history/rename/replace, corrections and historical replay, wrong-world and public refusal, backup/reopen and integrity checks. Browser mock-native tests cover saving, replay of the original typed AST, pinned revision and existing label search compatibility. Authored layout/Canvas/Display bindings, parameter slots/defaults, migration of AST schema beyond v1, indexed cross-primitive planner, safe published search and pagination remain open. No real native screenshot claimed absent actual capture.
