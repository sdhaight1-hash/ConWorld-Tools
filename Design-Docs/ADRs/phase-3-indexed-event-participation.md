# Phase 3 — Indexed Historical Event Participation

**Status:** Partial QRY-01/QRY-03 scalability slice, [issue #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4). Audit baseline `main` `992635ef9b4b97ccf7ccb55a65b5eaef78c866e9`: README, search/temporal/spec/acceptance docs, source/query, schema 0001–0012, tickets, screenshot standards and CI.

## Decision

Existing `EventInvolvement` versions already store event, participant, built-in role and custom role-definition ID in typed SQLite. Migration `0013.sql` adds one covering candidate lookup index on `world_id, participant_id, involvement_id`; existing `involvement_event_idx` is reused. This is **not** a new canonical table, FTS scan or cached results.

The two existing Rust `QueryFilter` modes `ParticipantEvents` and `EventRole` now gather distinct stable involvement IDs from indexed canonical version rows scoped to world and selected person/event, with a 10,000-ID hard candidate budget. The index only proposes candidate IDs. The authoritative `resolve_object` projection revalidates each involvement at world/branch/fictional day/author revision, role and visible participant/event kinds, rejecting obsolete, future, invisible or unrelated versions. The participant query deduplicates by stable event ID; event-role returns the correct involvement records. Results sort by label and stable ID and use existing 1–500 limit/truncation. The full 2,000-visible-object snapshot is no longer required for narrow role queries, and all non-author scopes remain refused.

This does not index unrelated text or change role, causal or membership semantics. Candidate overflow returns explicit budget errors, not misleading incomplete results; full 50k performance testing remains to do.

## Verification

Cover >2,000 unrelated objects, duplicate role involvements, role corrections, historical author revision, fork isolation, wrong-world and public refusal, schema-12 migration, backup/reopen and canonical index revalidation. Existing native person-role query UI is unchanged; no screenshot claimed without live native capture. Query pagination, cancellation, general Boolean graph/FTS planner, aliases, Canvas/Display and safe published search remain open.
