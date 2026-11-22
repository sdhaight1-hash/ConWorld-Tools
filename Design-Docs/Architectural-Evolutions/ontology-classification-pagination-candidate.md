# AE-05/08 — standalone classification pagination candidate

Baseline: held #74 at b6d4cfa, schema 43. This independent candidate is not a release or supersession of the held branch. No gates passed. The existing query planner materializes classification matches before paging and rejects above a 2,000-assignment scan limit. One-shot and composite queries retain that behavior.

## Isolated experiment (2026-11-22 UTC)

Python SQLite in-memory model, simplified schema, copied lineage and active classification conditions, synthetic integer identities and an independently enumerated expected-label set as a stand-in for reauthorization. No native Store or native security evidence. Label/ID candidates were checked for no omissions or duplicate accepted pairs, historical corrections, fork inheritance, branch withdrawal and sibling isolation across 1k/10k/50k and two profiles. First shape joining active assignments to versions and sorting all candidates took 9,419.3 ms for 49,500 dense results at 50k (100 SQL calls); the full branch experiment timed out. The subsequent label-ordered indexed seek with correlated subject/profile lookup passed 30 traversal parity cases.

| World objects | Sparse candidates/time | Dense candidates/time |
|---:|---:|---:|
| 1,000 | 10 / 2.8 ms | 991 / 4.2 ms |
| 10,000 | 100 / 28.8 ms | 9,901 / 43.0 ms |
| 50,000 | 500 / 211.1 ms | 49,501 / 472.0 ms |

Single traversal per shown main corrected view, 500 candidate pairs per SQLite call; the dense counts include one stale label. This is candidate SQL plus simple expected-set comparison, not per-page native latency, disk I/O, memory or fixed SLA. Sparse label scanning is slower than the first shape; measure before choosing whether a sparse fast path is necessary. Returned pair cap is not SQLite VM work cap.

## Native candidate

Add provisional schema 44 indexes, seek by the existing label/UUID cursor and reauthorize with resolve_object and has_profile in the same historical transaction. Retain scope, exact recorded view, cancellation and a 2,000 candidate-pair budget. If insufficient authorized candidates are found within that budget, report an error rather than invent a complete page. Only standalone ClassifiedAs pages use the new route; descriptor pages, composites, one-shot queries and other facts remain unchanged. No new public projection or external dependency.

Tests: existing historical classification regression now exercises pre-recording, inherited, withdrawn and public-denied pages. The native semantic workload must verify disjoint first two dense pages at 10k and 50k in addition to complete 1k traversal. Native compilation, fmt, Clippy, tests, migration/restore checks, real cancellation, duplicate labels and high-history density regression, and controlled latency still required. Current isolated environment has no Rust toolchain and no direct GitHub clone access. Do not merge until the held candidate baseline is verified and this candidate has its own exact-head results.
