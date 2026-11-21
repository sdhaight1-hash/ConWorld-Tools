# Phase 3 — Indexed historical backlinks

**Status:** Draft for [Phase 3 #4](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4), stacked on PR #35. Schema 18 adds four ordinary reverse-reference indexes, reusing existing relationship, claim-subject, event, participant and reviewed-mention indexes.

Previously backlink lookup enumerated all visible world records and failed at the 2,000-object cap. The Rust query now requests distinct candidate object IDs through a fixed SQL union of typed incoming references and source citations; it combines those candidates with explicitly approved, exact-version mention links. Every candidate and its target is resolved at the selected world, branch, fictional day and author revision, and its current reference is rechecked. Reviewed mention decisions retain the existing main-branch restriction. No hidden-reader search, new inferred relationship, or duplicated canonical data is introduced.

The candidate budget is 10,000 distinct IDs for the combined set; exceeding it is an explicit error. Display truncation occurs only after authoritative resolution and stable label/UUID sorting. Existing review lookup retains its own bounded history scan. This is exact typed reverse lookup, not a general authored-block or media hotspot index.

Regression exercises a >2,000-object world, source citation, relationship target, claim subject, explicit approved mention, corrected source span, historical revision, display truncation, public denial, and schema-17 upgrade. Existing mention review and roadmap tests remain applicable. Exact-head CI required, no new native visual evidence claimed.
