# Label collision indexing — #68 follow-up experiment

Date 2026-11-22 UTC. Independent research branch; extends prior cartographic prototype without production implementation or new schema. The accepted #68 scope and SP-07/08/11/14 gates remain unchanged. Earlier browser research in cartography-prototype/followup.py measures actual SVG label bounds; this experiment isolates the collision broadphase with synthetic already-measured rectangles, not actual font rasterization.

## Method
Python standard library; local Linux Python 3.13.5. Run python check.py in this folder. Seeded labels in a 1024x768 viewport, four candidate positions, priority then stable ID, exact AABB acceptance. Compare all accepted labels against a conservative 64-pixel uniform grid, still performing exact rectangle intersection. Compare exact placement and omitted/unknown lists, assert mutual nonoverlap, reverse input order and inject a hidden high-priority item plus an unknown anchor. The visibility flag stands in for upstream authorized projection; it is not an implementation or proof of access policy. No line geometry, font measurement, label rotation or viewport interaction tested.

## Observed single-run results
| Candidates | Kept | Naive exact checks | Grid exact checks | Naive ms | Grid ms |
|---:|---:|---:|---:|---:|---:|
| 1,000 | 253 | 230,860 | 8,150 | 24.01 | 7.62 |
| 10,000 | 353 | 2,871,977 | 98,664 | 296.15 | 91.20 |
| 50,000 | 423 | 14,688,929 | 503,801 | 1,548.86 | 538.20 |

Eight paired fixtures and all additional assertions passed. At 50k, approximately 29x fewer exact intersection checks and 2.9x lower elapsed time in this run. Timing is not a median or application SLA. Many labels are intentionally omitted: a 1024x768 viewport cannot show every label. Preserve authorized accessible lists and semantic navigation separately. A priority label can still be omitted when no collision-free candidate exists. Avoid recomputing 50k global candidates on each frame: viewport culling and work caps precede placement. Grid indexing is only a candidate technique; screen/font geometry, invalid inputs, cancellation, pan/zoom, real map scene, safe SVG and native timing remain untested. No screenshot is applicable to this headless numerical experiment.

Recommendation: retain a brute-force comparator for small fixtures; integrate only after Place and authorized scene contracts stabilize. Do not create a second semantic graph or infer location/geometry. No dependency added.
