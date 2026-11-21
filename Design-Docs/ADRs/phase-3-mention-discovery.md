# Phase 3 — Historical Mention Candidate Discovery

**Status:** First read-only discovery slice (DSC-01 / AT-21 / AT-35 partial). Tracks [Phase 3 issue #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4).
**Audited baseline:** `main` at `8adb20f7d9e8f204e8479e42cbb121b37fb73bd3`, plus the complete repository file tree, README, `/Design-Docs` index and query/storage/verification documents, issue/PR state, CI and development screenshot index.

## Chosen boundary

Inspect **author-visible, versioned claim statements** for **visible person labels** in the exact world, branch, fictional day, and author revision. Claim identity and its precise version are preserved via `VersionRef`. Each candidate contains a stable person ID, exact UTF-8 source byte span, text, and collision count; the author decides what any ambiguous name means. Duplicate labels remain separate candidates. Sub-word matches are excluded. ASCII-only case-insensitive matching plus exact-case Unicode names retains faithful offsets: full Unicode case folding and explicit aliases need a later indexed normalization step.

The Rust store's existing bounded `objects_at` is the authorization and temporal boundary, not frontend text matching. Additional comparison, text-length and candidate budgets prevent pathological scans; output is capped at 100 with honest `truncated` metadata. Crossing a budget is an explicit error, never a partial result misrepresented as complete.

This slice has **zero writes, zero schema migrations and zero authority to infer** membership, parentage, guilt, relationships or backlinks. UI shows candidates as proposals only, offers graph navigation where the person is already present, and invalidates stale responses on world/day/revision changes.

## Alternatives and remaining work

Automatically accepting an exact name silently promotes ambiguous evidence; rejected. Reconstructing author history in TypeScript duplicates the canonical Rust authority; rejected. Storing guessed aliases or author data in an unfiltered public index is unsafe; rejected.

Next: durable **accept/dismiss/snooze** of candidates keyed by exact source version/span/target, author-approved stable ID links and provenance-aware backlinks; aliases and index-backed scanning at scale; image hotspots, draft ghost nodes, candidate ranking, public policy prefilter, full historical pagination and Canvas/Display integration. This code does not satisfy full DSC-01 or AT-21/35, and no fabricated native development screenshot is claimed.
