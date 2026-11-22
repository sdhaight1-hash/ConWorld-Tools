# Independent Editorial Draft and Publication Lifecycle

**Date:** 2026-11-22 UTC. **Version:** 1.0 (accepted scope, detailed wire/schema readiness pending). **Reviewer:** Lola. **Owner decision:** [explicit approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/76#issuecomment-7079741458). **Research:** [#76](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/76), [study](../Research-Studies/comparative-worldbuilding-workspaces-and-encyclopedia-ux.md), [original disposition](../Research-Studies/comparative-research-ticket-disposition-2026-11-22.md). **Pinned baseline:** main schema 36 5777f0f, candidate schema 44 30b6710; #75 not modified. No implementation or production verification.

**Owner ticket:** [#77](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/77).


**Accepted Design direction — high priority, readiness A1 remains open for native workflow evidence.**

Problem and baseline: study §9.4, §9.7; store/entries.rs save_entry_layout writes document and published flag together, replaces public index/captions/dependencies and entry links transactionally; read_layout_at resolves latest eligible layout; workspace_save_layout returns unit; entry-editor.ts save sends checkbox, expected revision. A published page edited and saved while checked becomes the new publication. Saving unpublished withdraws current reader access. No separate currently released version survives unpublished work on same entry through this path. This is current behavior, not proven corrupt data. Normative 06 §§1,6–8 demands authorized versions/compiler, autosave and separate representation; 08 lists edit_canvas and publish_entry commands; ADR-010 edit/merge policy for published/derived records is open. A clearer pair of existing save/publish/withdraw actions is routine #5/#8/#10 work and MUST happen first. Only the new dual-head coexistence and changed publication/head-selection semantics require evolution approval. If existing contracts and explicit actions meet actual owner workflow, retain them and withdraw proposal.

Proposed delta: separately identify immutable latest author draft version and explicit current release selection, **for the same single entry and world store**. Author edits/save/restore append history, do not silently change current release. Publish explicitly validates draft/layout, current policy, same world, revision and selected supported context and atomically updates release/index; withdrawal revokes release independently. Current reader gets authorized current release, not latest draft by accident. Prior release content may have dynamic typed components evaluated at selected fictional date under existing scoped policy; exact time/pin contract must be reconciled with 06, INV-13/15 and #8, do not promise that authored prose is automatically historical. Revocation still authoritative even when viewing old versions; restoration of draft cannot revive revoked grants. Current main-only implementation retained until existing branch/perspective gates support widening. No extra ontology type, reader privileges, editorial approval roles, generalized workflow engine, cloud, collaboration, auto-publish or historical release time-travel. Archived or previously exported reader artifacts cannot be remotely recalled.

Before/after:
- Current single newest layout with publication flag; explicit release change coupled to save.
- Proposal latest authored layout separate from current release pointer/event. Same compiler, version IDs and authoritative Rust transactions; current release may lag draft.
- A save result returns committed version/context and generation; frontend marks only matching draft clean, no stale reply overwrites.

Ownership and dependencies: #5 Canvas/Display (layout/persistence/author commands), #8 policy and current release selection, #4 search/backlinks/anchor reauthorization, #10 archive/recovery/offline native test. #75 exact-head consolidation and review precede migration. #72 unchanged. Reconcile 06 §§1/6/7/8, 08 edit/publish and persisted layout API, 07 publication dependency model and ADR-004/005/010. Existing AT-17/18/19/20/22/25/34 unchanged. Accepted scope; see reconciliation below.

Alternatives: retain combined current state with honest Save unpublished (withdraws) / Save and publish; or explicit host-level draft UI without durable staging (doesn't meet saved-edit/reopen scenario). Do not add separate database or copy canonical data. Need real native author task demonstrating continued reader access while revised author page is saved/reopened. Independent editing of released article adds publication/index migration burden; no implementation unless value established; specific bounded scope approval is recorded above.

Migration/compatibility: pin accepted baseline, examine latest layout per entry. Proposed backfill release to same existing latest published version only if currently published, otherwise none; never select a previously published but now withdrawn layout. Stable ObjectId/ElementId, layout/template pins, provenance, branch history, archive hashes. Transactional migration and verified pre-migration protected backup; check old schema refusal, rollback on failure, restore exact prior archive; rebuild publication indices exclusively from current release selection, no draft leakage. Exact schema/table design and handling legacy revisions/invalid pointers to be reviewed, not fixed here.

Proposed gates (not passed):
- [ ] A1: native workflow demonstrates need versus current explicit action path; owner approval recorded; verify native workflow need and finalize schema/wire/time semantics and reconciliation before production.
- [ ] A2: save revised draft and reopen while public body/link/index unchanged; explicit publish updates authorized page/search/backlinks atomically; stale expected revision fails without partial mutation. Restore historical draft does not republish. A withdrawn release cannot be served from stale cache, cursor, anchor or export build.
- [ ] A3: verify private subtree, hidden target/caption/count, character-context and cross-world denial, no unavailable fields, current audience revocation under old draft pin. No broader unsupported branch publishing.
- [ ] A4: upgrade actual prior release database, verify same publication, withdrawn page remains withdrawn, template pins/element IDs/source preserved, rebuild parity, backup/restore, failed migration no data loss. Native Linux/Windows, keyboard draft conflict, archive and actual screenshots at exact build.
- [ ] A5: docs reconciled, review and existing phase gates preserved. Stop/defer if A1 absent or policy/compatibility fails.

Evidence: [study §9](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/8cfb18643ff1871d6b4fbc465bf4ab0396814c3e/Design-Docs/Research-Studies/comparative-worldbuilding-workspaces-and-encyclopedia-ux.md#9-lifecycle-terminology-template-semantics-and-reusable-knowledge--further-research); [synthetic 23 checks](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/db1bdf39571e6c252f4eb40560c2d411e53bce82/Design-Docs/Research-Studies/workspace-lifecycle-prototype) (not Store/native verification); Notion history https://www.notion.com/help/duplicate-delete-and-restore-content ; World Anvil article workflow https://www.worldanvil.com/learn/article-guides/articles.

## Reconciled decision and authority

[ADR-014](../ADRs/adr-014-independent-editorial-release.md) records the accepted dual-selection delta. This design supersedes the latest-layout-is-current-release assumption only for the future #77 implementation. Until A1 and reviewed migration, actual runtime stays as documented. Stable ObjectRef/ElementId and existing ViewContext continue. Conceptual logical state (not final SQL): newest author layout version, optional current release version referencing an immutable saved layout for same world and entry, and versioned release/withdraw record; publish/withdraw append audit history atomically with index changes. No historical release browsing capability added. No claim that draft restoration restores a grant.

| Affected contract | Accepted refinement | Preserved / readiness |
|---|---|---|
| 06 §§1/6/7/8 | Draft layout selection separate from reader release selection | Same scoped compiler, current release reauthorization; precise date/branch binding semantics reviewed at A1 |
| 08 commands/layout persistence | Save draft vs explicit release and withdrawal, revision-checked | Atomic Rust authority, exact immutable layout pins; final migration and wire contract pending A1 |
| 07 §§5/7/8 | Publication dependencies and index derive from current release, not latest draft | Bounds, reauthorization, deterministic rebuild; no secret corpus scores |
| 12 / ADR-004,005,010 | ADR-014 extends these open decisions for release selection | Original invariants, phase exits and other ADR responsibilities unchanged |
| 01 INV-01/10/13/15/16, 09 acceptance | No altered invariant or original AT gate | A1–A5 add evolution-specific evidence only |

## Evidence and implementation boundary

Approval recorded. A1 native comparison and finalized schema/wire contract not run. A2–A5 not run. No accepted production migration number, no new runtime API claims. Use existing #4/#5/#8/#10 owners and preserve exact #75. Stop at readiness failure and revisit decision; do not make existing draft safety fixes wait.
