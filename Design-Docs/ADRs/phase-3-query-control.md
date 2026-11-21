# Phase 3 — Cooperative query cancellation and execution deadlines

**Status:** Draft, stacked on #37, tracks [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4). No schema change.

A read-only QueryControl combines a monotonic deadline with an atomic cancellation flag shared across threads. The controlled query and page APIs check permissions before installing a temporary connection-local SQLite progress handler (rusqlite 0.37 hooks). The callback interrupts SQLite VM work; candidate resolution, traversal, review lookup and compound planning/evaluation check the same token. A scoped guard clears the handler and token on error or unwinding, so ensuing edits and reads do not inherit cancellation. No interrupted read returns partial results or a continuation.

These are cooperative deadlines, not hard realtime guarantees: waiting on SQLite's existing busy timeout, OS scheduling, and bounded final sorting can overrun the requested duration. No writes are interrupted. Explicit cancellation and expiry return distinct typed errors. Tests cover pre-cancellation, zero duration, capability denial, invalid query cleanup, successful controlled replay and subsequent canonical write/read. Native worker dispatch and cancel controls are the next integration step. Exact-head CI required.
