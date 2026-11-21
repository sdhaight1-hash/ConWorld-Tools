# Phase 3 — Historical Event Participation by Person

**Status:** A bounded author-only structured query slice for [Roadmap Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4). Audited baseline `main` at `45377f9c99f1d1af517f5d1f72ba1aa618c5230a` after merged [claim FTS PR #26](https://github.com/sdhaight1-hash/ConWorld-Tools/pull/26).

## Decision

Reuse the canonical `EventInvolvement` objects and existing `QuerySpec` historical `objects_at` projection. A new typed `ParticipantEvents { participant: ObjectRef, role: Option<EventRole> }` filter returns **unique historical event identities**, not involvement-record labels or inferred involvement. The selected person must be an existing visible `Person`; only explicit dated involvement records can contribute. Optional `EventRole` supports built-in and author-defined role IDs, not fuzzy text guessing. World/branch/fictional day/author revision and `Scope::Author` are inherited unchanged.

Native Node UI adds a person-and-role search that reuses the existing query result list, linked event selection, revision invalidation and bounded `limit`. Rust owns the filter and serialization into the existing author search DTO. No SQL migration, speculative query language or new authority, and no inference from co-occurrence or source text.

## Verification and limitations

Tests cover multiple events/roles, deduplication by event, historical day, old author revision, correction, alternate fork, cross-world refusal, invalid non-person reference, result truncation and denied public search; a mock-native Playwright test checks actual controls, custom-role ID, query arguments and view-change invalidation.

This is a bounded, 2,000-object **author-only** snapshot filter; it is not indexed event participation at 50,000 nodes, a saved parameterized query, a Canvas binding or safe public search. No real native UI screenshot is claimed without a running capture. Phase 3 remains open for broader AST composition, scale, pagination, alias/index and publication gates.
