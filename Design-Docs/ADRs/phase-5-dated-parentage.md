# Dated accepted parentage

Tracks [genealogy #6](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/6) and [fictional inheritance #9](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/9). Schema 23 adds accepted parent links to the local author store. The proof fixture and native UI use the main historical branch.

Each link has a stable UUID, two person endpoints, a type, a start day, an optional exclusive end day and a recording revision. The revision carries the author's basis. A withdrawal adds an audit revision and closes recorded visibility. Reads intersect the selected fictional day and author revision. Old snapshots resolve the relative's label at that snapshot. A dated end describes a change in the world. Withdrawal corrects what the author accepts about the world.

Writes validate world, branch, scope, expected revision and endpoint type. Overlapping links of the same type and direction are rejected. Multiple distinct types can coexist. The cycle check carries the overlapping day interval through descendants, including future periods. Reaching the proposed parent with a nonempty interval rejects the whole transaction. A traversal exceeding 10,000 states also fails atomically. Focused family reads allow up to 2,000 matching links. Existing databases migrate with an empty family table. Free-text claims, aliases and generic relationship labels never populate it automatically.

The native inspector edits accepted links and displays parents and children. The inheritance comparison reads two historical sets of accepted biological parents and uses the existing seeded contributor model. Adoptive and legal parents remain visible in family history. Disputed accounts, primary-source object bindings, fork inheritance for kinship, households, organizations, general family trees and multigeneration population modeling remain open.

## Verification

The Rust fixtures check future three-person cycles across parentage types, disjoint periods, overlapping duplicates, stale writes, historical withdrawal, endpoint-scoped correction, reopen, reader denial and reproducible fictional intervals after a biological correction. In the native Linux Tauri app, four person records and four biological links produced a sibling comparison. A second process reopened the same SQLite database and reproduced the intervals. [Tests](../../crates/atlas-store/tests/roadmap_foundations.rs) · [Native capture](../../Development-Screenshots/2026-11-21_20-38-21Z_phase-05_family_saved-parentage-comparison_01.png)

The final native build also withdrew one of Ada's parent records at revision 9. Selecting read-only revision 8 restored both displayed parents. The full locked workspace suite passed 80 tests and strict Clippy passed across all workspace targets.
