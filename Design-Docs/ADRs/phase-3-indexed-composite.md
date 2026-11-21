# Phase 3 — Bounded indexed compound predicates

**Status:** Draft implementation for [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4). Baseline `main f3af1b963a25fa76e082800ca9a7d8516d244020`; see the issue's 2026-11-21 structural audit. This does not close AT-22/23/24/25/35.

## Decision

Extend the shared Rust-owned strictly tagged predicate wire schema with literal label and claim-statement full-text leaves. The existing SQLite FTS5 functions supply **resolved historical versions**, not raw index hits, under a pinned world/branch/day/author revision. The existing versioned saved composite JSON envelope stores the leaves without a migration; legacy predicates retain their meaning. No SQL string is supplied by the caller.

Algorithm:
- Validate the entire AST (depth, width, node count, reference worlds and text length) before evaluation.
- Resolve each distinct indexed leaf with existing bounded FTS5 queries. Collect exact object IDs and canonical versions at the selected context. Limit aggregate resolved indexed memberships to 10,000; retain existing raw per-index candidate cap. Never treat an incomplete leaf as a complete set.
- Positive indexed AND children bound the universe by set intersection. OR can bound the universe by union only if every child has a positive candidate bound. NOT supplies no positive bound. If a complete bound is not possible, enumerate the existing 2,000-visible-object snapshot or return its explicit budget error.
- Evaluate the complete original predicate over the candidate universe, including nested NOT and source visibility. Resolve cited source IDs independently, since they may not occur among the indexed candidates.
- Sort by canonical label and stable ID, then apply the requested result limit. Do not truncate a leaf to the display limit before evaluating the full predicate.
- Keep public/character scopes denied until the authorized query projection exists. No result cache is introduced.

The author Node builder accepts both kinds of indexed phrase, and saved composite replay uses the same AST and query engine. The full desktop scene still has a separate 2,000-object snapshot gate; successful large-world store tests do not prove the large-world desktop workflow.

## Verification targets

Rust regressions: >2,000 unrelated objects with narrow indexed AND, all-index-bounded OR, NOT complement on a small snapshot with failure on an oversized one, selected fictional day, correction and old author revision, wrong audience, and durable saved AST replay/backup. The existing mock-native browser saved-query regression exercises the new claim phrase field. Exact-head Ubuntu and Windows CI must pass before merging. No native capture is claimed.

**Remaining:** independent focused-scene loading, indexed role/graph composition, paging/cancellation, typed query parameters, alias/source/media indexes, mutation dependency invalidation, all-studio permission-filtered projections and large-world benchmark evidence.
