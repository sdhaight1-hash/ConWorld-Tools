# Cross-phase development record · November 21, 2026

Development branch based on Phase 3 PR #47, schema 22. This record covers a working character-page workflow plus focused reads and recovery. All seven roadmap phases remain open.

| Phase | Delivered in this change | Remaining boundary |
|---|---|---|
| 3 · #4 | Open a person’s page directly from a large-world historical search result. FTS label and claim queries now read matching postings before document metadata, apply lineage and version checks, and sort resolved results. A persisted author-only layout query is replayed in the integration fixture. | Public full-text indexing and shared reader-safe query bindings. |
| 4 · #5 | Versioned SQLite layouts with stable element IDs. Native person inspector creates and reopens a two-block page, edits it, and previews its public projection. More complex layouts remain readable through the store API. | General Canvas designer, templates and public query components. |
| 5 · #6 | Dated pedigree validation resolves distinct referenced people in a single read transaction, independent of unrelated population size. | Durable typed kinship and organization authoring. |
| 6 · #7 | Confirmed impact resolves its starting event and referenced confirmed event/source records at the historical view. Existing date, source, cycle and traversal constraints still apply. | Persistent causal links, conflict editor and faction histories. |
| 7 · #8 | Each layout revision records its explicit publication state. Trusted public IPC loads the latest main-branch document. Hidden subtrees are omitted. Reader-visible author queries are rejected on publication. Historical author reads survive revocation. | The native author preview is the current reader surface. Standalone reader navigation, public indexes, asset grants and per-field policy remain open. |
| 8 · #9 | Compare two fictional people’s dimensionless model intervals using shared immediate biological contributors. Results are symmetric, recomputed from the supplied dated pedigree, and use the existing model seed/version. | Founder tracing, population simulation and durable match reports. |
| 9 · #10 | Verified archive restore to a fresh destination through the desktop CLI. The snapshot is copied, checked and migrated before installation. Replacing an existing destination fails. Added a reproducible focused-world benchmark. | Asset bundles, recovery UI, installer and full hardware acceptance. |

## Native and automated evidence

On Linux, the Tauri app created a world and a person, saved public text plus private author notes and reopened the page after restart. The public preview returned the saved text. Clearing publication and saving removed it from the preview. The screenshot shows the editor and preview at the same time. [Native capture](../Development-Screenshots/2026-11-21_20-07-42Z_phase-04_character_public-page-preview_01.png)

A separate test restored a page with a hidden query, verified the author could replay the original revision, checked that public historical replay was denied after revocation, and compared destination bytes after a rejected overwrite. The compiled CLI also restored a one-world database and refused a second restore to the same file. Larger-world tests exercised family and causal reads above the previous 2,000-object scene limit. [Integration tests](../crates/atlas-store/tests/roadmap_foundations.rs)

## Focused-read measurements

Run `cargo run --locked -p atlas-store --example focused_world -- 50000` from the repository root. The example writes deterministic object IDs and labels to a temporary SQLite file in batches of 1,000, reopens it, then checks every timed result. Counts include two events, one source and the remaining people. The query searches for a two-token phrase with one historical result. Family validation uses one legal parent link. Impact traversal uses one confirmed sourced edge. [Workload](../crates/atlas-store/examples/focused_world.rs)

| Total records | Fixture write | SQLite bytes | First label query after reopen | Median next four label queries | Median next four family reads | Median next four impact reads |
|---:|---:|---:|---:|---:|---:|---:|
| 1,000 | 0.45 s | 1,179,648 | 3.75 ms | 0.59 ms | 0.36 ms | 0.55 ms |
| 10,000 | 2.98 s | 8,159,232 | 3.38 ms | 0.76 ms | 0.32 ms | 0.43 ms |
| 50,000 | 16.77 s | 39,571,456 | 4.80 ms | 1.57 ms | 0.56 ms | 0.77 ms |

On the same development host, before changing the FTS join order and removing the redundant pre-resolution sort, the 50,000-record first read took 29.72 seconds. The 10,000-record first read took 3.92 seconds. These Linux debug-build timings include the Rust engine and SQLite. The operating-system file cache was not flushed. Concurrent host work can affect absolute times. These measurements cover one selective workload. Broader queries, multi-page UI latency, memory peaks and Windows target-hardware acceptance still require measurement.
