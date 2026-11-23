# Ontology consolidation acceptance review — 2026-11-22

## Frozen verified baseline

Parent `27daf6c2e3876d9f3f9becb378a1c0aeca7af2ce` (schema 44), main `5777f0faea7dd5c4aa93908d35d3cfd7cfea773f` (schema 36). Both complete [persistence](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45016648951) and [desktop](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45016648995) runs green, six jobs; 102 Store tests and 112 workspace tests per operating system, strict fmt/Clippy/build, Linux browser and Windows native. The combined baseline is 60 commits ahead of main, contains the full histories of #73/#74, and retains the spatial foundation without merging #67's conflicting experimental migration. Keep #68 SP and #70 WT acceptance independent. Recovery/network pilot 9c9bfdc remains separate and unverified.

At 50k Linux first sparse/dense samples 148.846/95.615 ms, composite/Boolean exercises 288/291 ms, peak process RSS 96540 kB; Windows first sparse/dense 280.561/186.103 ms, composite/Boolean 588/582 ms. Setup 173477/247216 ms respectively; cancellation/deadline/absence permutations pass. These are release-example native measurements, not a guaranteed SLA or a GUI usability study. Windows peak RSS not measured by this harness. Native Windows three author-task observations 235.1 ms/4 submissions, 138.6 ms/4, 192.0 ms/3. [Inspected chronological native capture](https://github.com/sdhaight1-hash/ConWorld-Tools/actions/runs/45016648995/artifacts/14751662688), see Development-Screenshots index.

## Gate-by-gate review

| Gate | Evidence on frozen baseline | Outstanding |
|---|---|---|
| AE-01 | All nine A/B/C cases pass per OS with source/history; B suffices culture-language, C adds exact reusable category (2 vs 0 of 3) and typed area. Preserve stop rule; no new domain scope. | Distinguish API command comparisons and native C UI tasks from comparative human UI cost. |
| AE-02 | Native fact family/wrong-world/author restrictions, historical annotations/withdrawal, rename/source checks, backlinks and stable reference archive replay. | Reconfirm at consolidation head. |
| AE-03 | Four curated profiles, compatible version pins, local retirement, dated co-classification and branch-local withdrawals; no inference. | Reconfirm at consolidation head. |
| AE-04 | Area unknown/zero/units/dates/correction/replay, typed CultureLanguage/WorkingMaterial and existing validators. | Reconfirm at consolidation head; no generic schema authoring. |
| AE-05 | Boolean/descriptor pagination, cursor context, saved query pins, cancellation, invalidation, bounded discovery, scope denial and 50k parity/absence workloads. | Reconfirm at consolidation head; global historical context remains separately tracked. |
| AE-06 | Actual Windows author forms and endpoint selection; Linux browser Canvas/Display privacy and keyboard interaction; typed reader bindings deny unsupported access. | Reconfirm; accessibility not a blanket product certification; independent SP/WT gates. |
| AE-07 | Reconstructed schema-36 migration, rollback, incompatible lineage rejection and archive restore regressions pass. | Live startup previously migrated in place without recovery copy. True predecessor binary fixture and protected migration verification required. |
| AE-08 | Both native 1k/10k/50k and exact-head desktop green, real capture inspected, no-GPU fallback verified. | Confirm new head, review observed interactive performance and budget; Windows process memory and full offline validation not established by current logs. |
| AE-09 | Accepted ADR-007 and scope retained, dependencies linked, this review. | Final owner review/closure after outstanding evidence; no automatic acceptance. |

No formal gate is closed by this record. Main remains untouched until safe consolidation.

## Migration safety correction

Live desktop now uses Store::open_with_migration_backup rather than the bare fixture/archive Store::open API. During startup the existing SQLite immediate transaction reserves the writer slot, BEFORE any DDL of a non-new database, and holds it through migration. A separate read-only connection makes VACUUM INTO staged sibling copy. The copy is checked with existing read-only archive integrity (legacy 1/2 databases use app ID, version, quick_check and FK checks because full archive format starts at 3), synchronized and published non-overwriting as `worlds.sqlite.pre-schema-<old>-<uuid>.sqlite`. No backup on fresh/current-schema open, no automatic removal of previous backups, and no destructive downgrade. Reject copy failure before migration; keep completed recovery file on later rollback. This is file-sync and staged no-clobber behavior, not a guarantee of directory persistence on every platform/power-loss case. User can restore a verified copy to a separate destination with the compatible tool; do not overwrite the live file or use old binaries on schema 44.

Actual local Python SQLite 3.46.1 probes confirmed a separate read-only VACUUM INTO can copy the old committed row at schema 36 under the held BEGIN IMMEDIATE in rollback-journal and WAL mode; competing writer received database-is-locked until migration committed. This is not native Rust execution. New Rust tests verify v1 compatibility and automatic backup/restore on reconstructed 36, corrupt-source refusal, unchanged backup on current-version reopen. New cross-platform workflow compiles a fixture generator against exactly 5777f0f, checks original membership/parentage IDs, source and historical rename after automatic migration and restoration. This is a deliberately authored representative fixture, not a user's production database. No new schema version.

## Integration procedure

Keep original 27daf6c evidence frozen. Run new exact-head validation on a single consolidation PR; do not independently merge #73/#74/#67. Main advances only after migration and remaining gate review. The recovery/network hardening prototype is outside this PR. Test results not yet available for this correction. No new visual surface changed.
