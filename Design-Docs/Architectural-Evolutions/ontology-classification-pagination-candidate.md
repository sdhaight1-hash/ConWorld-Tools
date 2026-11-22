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

## Descriptor identity continuation

The same standalone cursor path now accepts ClassifiedBy without a schema change. The descriptor must be a visible Tag definition in the requested world and recorded view; the list and page paths share that preflight. Each candidate is resolved at the chosen date/branch/revision and its matching classification rows reauthorized with the existing decoder. Multiple valid profiles on one object collapse to one label/UUID result; overlapping same-profile facts fail closed. A private descriptor, wrong kind/world, stale cursor context, and withdrawn or incompatible rows must not become results. No general boolean planner changes or public projection.

Executed a follow-up candidate-only SQLite experiment using the previous fixture with two descriptor IDs and a co-classified subject. 45 parity cases (five historical/branch contexts, three descriptor selections, three scales) passed. Corrected main dense traversal at 50k returned 49,500 objects in 99 calls, 226.7 ms; sparse 500 objects in 187.6 ms; absent descriptor candidate scan 202.8 ms. On the production path absent descriptors short circuit before scanning. These are single synthetic runs with an independent oracle, not native query latencies. A separate 10k synthetic revision-density probe with ANALYZE confirmed the label seek and subject index plan for both profile and descriptor. Index suitability under native history must still be measured.

New native tests exercise same-label UUID order, co-classification deduplication, rename replay, stale cursor rejection, wrong kind/world and public denial; existing fork/withdrawal tests exercise descriptor page parity. The scale example now checks two disjoint controlled descriptor pages at every scale. Exact-head Rust CI remains unverified. No new screenshot applicable; no claim of completed AE gates.

## Positive semantic conjunctions and saved replay

The desktop combined-search form emits a flat AND/OR and already uses the controlled page API; Canvas and other bounded consumers use execute_query. This candidate now shares the seek path for both entry points for standalone classifications and flat positive ANDs containing a profile or descriptor, optionally other classification, kind, exact-label or label-contains terms. All predicates are validated first; every descriptor is preflighted even if another descriptor is absent. A positive classification anchor generates ordered candidates; all active subject assignments are checked with the existing canonical decoder in the same snapshot. Membership sets distinguish stable descriptor identities and curated profiles, with existing local label/kind semantics. No joins to arbitrary payloads or alternate truth store. Unsupported boolean, text-index, area and source combinations continue through the original planner. The candidate-pair cap is retained; selective residuals can still exhaust it. The one-shot result reports truncation only after finding an additional authorized match, not merely because the SQL scan is full.

Schema 44 unchanged. New regression assertions cover co-classifications, current/historical parity, case-insensitive label matching, invalid descriptor/label validation before an empty result, and persisted definition replay. Native scale harness now requires bounded one-shot dense queries and two disjoint composite pages at 1k/10k/50k, with identical first-page rows from both entry points. These are newly authored checks, not executed outcomes. The container has no cargo/rustfmt and cannot resolve github.com; native CI remains outstanding. No UI changes or new screenshots required. Do not merge on source review alone.
