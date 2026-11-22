# Composable Worldbuilding Transforms and Exploration Workflows

**Status:** Accepted Design — version 1.0; implementation readiness and production verification pending.  
**Date:** 2026-11-21 local / 2026-11-22 UTC. **Owner:** repository owner. **Design author:** Lola.  
**Authority:** [New Feature #70](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70), [explicit owner approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70#issuecomment-7073776197).  
**Research:** [#69 closed study](../Research-Studies/maltego-inspired-transforms.md), [closure review](../ADRs/phase-3-contextual-exploration-review.md), pinned baseline [62a39e0](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/62a39e05c62f46acdb6ffa340abb74d64945b055).

## 1. Problem, value and accepted boundaries

Worldbuilders repeatedly follow several connected historical questions, manually passing selections and parameters between queries. Reusable typed exploration recipes provide step-by-step evidence and repeatable traversal of authored fictional worlds. This is exploration and consistency support, not intelligence gathering or automatic world creation.

The original charter, [07](../07-Search-Query-and-Discovery.md) and #4–#10 already specify contextual queries, graph traversal, genealogy, saved parameterized predicates, mention discovery, review and authorized Canvas/Display. Those remain original scope. #70 adds finite ordered multi-stage composition, an inspectable execution ledger with intermediate provenance, compatible author configuration, versioned recipe definition and replay. #68's spatial/map recipes are independent; spatial exploration calls its verified read services only. No new phase or change to original exit criteria.

V1: author-only, offline, one or two curated finite recipes first; then small ordered editor using allowlisted compatible operations, versioned definitions, save/reopen/replay and portable archive. Single selection, and explicit pair selection only for a verified existing pair service. Existing typed services determine available domains. No mandatory new kind, inference engine or data generation.

Excluded: new canonical graph/database, arbitrary SQL/JavaScript/Python, plugin runtime, arbitrary DAG/loop/recursion, external network or OSINT, AI dependency, real-person lookup, speculative document ingestion or consistency algorithms, persistent raw result history, background autonomous execution, result-driven writes, public/character execution, live Canvas recipe execution, new GPU renderer. Dedicated lore workbench needs distinct scope review. Author approval of scope does not waive readiness Gate A or guarantee implementation if no author-value gap remains.

## 2. Readiness gate A (WT-01) — mandatory before recipe production work

Resolve/reverify the outstanding integrated baseline desktop caption-search deadline failure and examine current exact-head CI. Do not mistake old successful persistence runs or research fixtures for native product evidence. Existing #4/#5 contextual actions come first; select a verified read service such as participant events for a selected Person. Compare ordinary query, saved query and contextual actions using three actual author workflows, on the same pinned world/view and intended outputs: genealogy (family links/ancestry as available), institutional/event history (membership/participant events), and source/lore review (claim or focused mention query). Include realistic repeated tasks with changed selections and historical context; measure completed tasks, actual configuration/navigation steps, errors, elapsed time and evidence explainability. Record fixture, exact commit, platform and policy, observed failure modes, and what is already solved. Do not count missing underlying domain functionality as a benefit of recipes.

Before executing new feature slice B, record in #70 a reproducible gap that remains despite ordinary/saved queries and contextual actions (repeated manual frontier transfer or lost intermediate explanations, for example), compare against simpler extension, and establish measurable baseline and performance limits. If no meaningful gap, stop and record Deferred/Rejected disposition instead of adding a runner. Native usability and dense workloads were not verified in research #69. WT-01 is unpassed. This gate authorizes no changes to #67/#68 readiness or original phase obligations.

## 3. Author stories and UX

- Select a House at the current world, branch, day and revision. Choose a compatible “Dynastic influence” recipe; inspect conceptual House → recorded members → their institutional memberships → institution event involvement. Each operation must be actually supported, typed and sourced; no automatic inference of individual participation, biological parentage, allegiance or causality. If underlying service unavailable, do not offer operation as ready. Initial curated recipe chosen from available verified endpoints after A rather than implementing absent domains.
- Preview ordered steps, required role/lens/other parameters, input kind/arity, supported output, work caps and view; execute explicitly. Show progress, completion, empty, partial/truncated, cancelled or error. Result graph merges visible stable identity refs and original directed facts only; accessible list and per-step ledger include source and original intermediate path. Distinct assertions with same endpoints remain separate. Where service returns rows only, show list or labelled derived path, not fabricated fact edge. Undo removes view overlay only.
- Author composes allowed compatible linear steps in a modest keyboard-operable ordered editor, validates in Rust, saves named version and replays with another compatible selection. Reordering or parameter changes create explicit new version. Unsupported/stale dependency offers clear incompatibility; no guessed semantic substitution. Replay current authorized view by default or explicitly select an available historical pinned context. View or selection changes cancel/discard mismatched runs; no cross-view graph union. Existing explicit review UI handles source candidates, never automatic canon promotion.
- Empty operation catalog, wrong kind/arity/world, missing service/parameter, invalid transition, incompatible version, denied context, timeout/cap and cancel each have safe clear state. No raw unauthorized labels/counts. Keyboard focus, labelled selection/step actions, accessible list navigation and source links use existing studio conventions. No requirement for visual programming editor.

## 4. Technical design and domain integration

Use one Rust authoritative domain/store/query executor; reuse atlas-store QuerySpec/QueryControl and typed genealogy, network, evidence and spatial read services when verified. Existing source reviewed in study: query.rs (allowlisted filters and saved typed parameters, author-only request validation, bounded pages), discovery.rs (focused mention candidates), store/network.rs (network facts, persisted impact and separate succession preview/enact), store/family.rs, access.rs, apps/desktop/src-tauri/src/workspace.rs, apps/desktop/src/author-workspace.ts and family-history.ts. Avoid a general command bus exposed to arbitrary UI arguments. Operation availability reflects real implemented capabilities; no arbitrary user service names. Implementation selects concrete module location only after A, keeping changes minimal and not duplicating planner, access checks or graph storage.

Logical contract (not generated Rust or approved physical schema):
~~~text
OperationDescriptor = stable_id, semantic_version, input_kind_and_arity,
  typed_parameter_schema, result_kind_and_semantics, allowed_read_binding
RecipeDefinition = world, stable_id, version, title, input_contract,
  ordered_step_bindings, parameter_bindings, operation_or_query_pins
RunContext = authorized frozen ViewContext, selected ObjectRefs, shared limits
StepReport = step_id, input_refs, authorized frontier_refs,
  original_fact_and_source_refs, derived_path_or_candidate_reports,
  status, truncation
RunResult = context, definition_pin, ordered_step_reports,
  deduplicated_authorized_presentation_fragment, completion_status
~~~
Definitions and runs are metadata/projections, not semantic world objects. Use existing ObjectRef, source/fact IDs, temporal resolver, typed role distinctions, epistemic status and stable query version. Pair action supports exact arity and endpoint kinds only if underlying verified service. For v1 linear steps, a stage output frontier must match next input type and arity contract; no general Cartesian pairing or arbitrary graph-as-code. A candidate report cannot feed canonical traversal; explicit existing review command and new snapshot required.

~~~text
validate_definition(definition, capability, trusted_view, selection)
freeze authorized context once
for each ordered step:
    check shared cancel/deadline/work/byte budget
    validate compatible typed input & resolve each authorized ref
    call existing validated query/domain service with shared control
    check original source/fact/target access and result type before output
    union visible refs by stable identity; preserve distinct facts and paths
    record authorized step report; stop on error/partial/cancel
return safe result, explicit completion status and context
~~~

Do not reset budget per node or step. Bounds cover max steps, targeted calls, fanout, visited/frontier, rows/edges, output bytes, time and aggregate memory; all finite and Rust validated, shared across nested services. Exact numeric caps must be agreed from Gate A measured service costs and fixture requirements before writing runner; no fabricated target SLA or unsupported numeric universal limits. Start at 2–4 steps for curated recipes, bounded linear sequence only; use smallest sufficient caps within existing query service maxima. Deterministic iteration and ID tie-breaks, no duplicate calls for same step/input/pin; no truncation hidden as no match. Partial stops downstream and terminal completeness is explicitly withheld. Error and cancellation distinguished. Do not claim deterministic results when underlying service/version changes: validate pins and context, deny incompatible replay.

Authorization enforced by trusted Rust command context, not UI-provided audience flag. Validate world, branch, day, revision, actor capability, selection, saved definition dependencies, inputs/intermediates/edges/evidence before traversing, aggregation, counts, layout/export/cache or return. Author-only initial commands fail closed for Public/Character; don't run author workflow inside hidden Canvas component. Do not persist author output cache in published artifacts. Reauthorize each replay and any retained view fragment; mismatched context/permissions invalidates. No hidden intermediate path, source, count or metadata channel. Claims, rumors, uncertain times and derived paths remain distinct. Historical comparison uses two independent authorized snapshots displayed separately; no mixed union.

## 5. Three-layer, search and integration boundaries

Node Studio: contextual operation/recipe selection, existing focus and graph/list, bounded per-step inspection and original evidence navigation. TypeScript handles rendering and request IDs/cancellation only; Rust validates and executes. Canvas Studio uses ordinary existing typed saved query bindings, not hidden recipes; author navigation may follow authorized entity refs, no independent semantic data copy. Display Studio continues existing publication and reader search contracts, does not execute recipes, expose definitions containing author-only refs or show raw run output. No GPU change unless a separately demonstrated renderer need, outside v1.

Search/discovery use existing labels, aliases, source/backlinks, typed graph queries and reviewed ghost/mention candidates. No new search engine, no name-based merge, co-mention not affiliation, no automatic keyword/image interpretation. Recipe definitions are local author metadata and may be selected by title/filter, not inserted into public world-object index. Invalidate exploration results on view/revision/permission or pinned dependency change; rerun rather than cache stale graph. Saved-query or operation semantic version upgrade explicitly validates new version, never silently rewrites old recipe. #6–#9 domain semantics and #68 spatial queries are dependencies only: role-specific chronology, legal/biological ancestry, causal stances, evidence and lens separation preserved; no inferred spatial membership or ownership from proximity. Synthetic ancestry uses existing seed/version contract if service verified.

## 6. Delivery slices and dependency ownership

| Slice | Scope | Exit evidence |
|---|---|---|
| A | Baseline verification and #4/#5 context-action comparison | WT-01, concrete benefit or stop/defer |
| B | One or two ready curated author-only recipes, typed catalog, linear finite runner and ledger | WT-02–06 applicable subset, read-only native result/list; no persistence yet |
| C | Compatible ordered step editor and parameter binding | WT-02/03/05/06/07, invalid configuration and cancellation |
| D | Versioned definitions, save/reopen/replay/portable archive | WT-07/08, migration/rollback and version incompatibility |
| E | Integrate additional verified services only where useful; branches, scope, original fact/evidence assertions | WT-03/04/06; policy starts in B, not deferred to E |
| F | Native usage/scale/offline/fallback, full review | WT-09/10 and all prior gates |

#70 owns feature scope and gates. #4 search, #5 studios, #6 genealogy/institutions, #7 event/conflict, #8 branch/evidence/privacy, #9 optional population, #10 portability/performance and #68 optional spatial are integration prerequisites, not additional original exit gates. Existing-scope actions remain #4/#5 even if #70 stops. Cross-link PRs; tests before edits; reconcile exact-head baseline and read services at each slice. No feature dev or migration before A. Genuine native/mock evidence labelled with chronological UTC capture per [Development-Screenshots guide](../../Development-Screenshots/Dev-Screenshots.md); capture only actual executed UI, no screenshot needed for this approval.

## 7. Storage, migration, export and rollback

No storage change before B proves benefit; default runs ephemeral. For D persist only world-scoped recipe identity and validated immutable definition versions/pins in existing SQLite authority; use existing transaction/revision/conflict controls and archive. Avoid new database, copied world facts or stored raw run history. Actual schema/version allocation and API chosen against then-current baseline. Definition mutation cannot alter world facts. Validate malformed/incompatible or missing refs at save/load/replay, retain historical definition and show explicit incompatibility, author chooses upgrade. Backup before upgrade, migration and reopen FK/identity verification, archive roundtrip preserving definition IDs/version/pins and local references. Older version rejects incompatible schema; rollback by reviewed code revert, restore verified pre-upgrade backup to new destination when necessary, no loss of older canonical history. Separate author archive from public export; no hidden definition payloads in reader artifacts.

Risks: unbounded frontier/call reuse (global caps), loss of provenance (original fact refs), false inference (typed semantics and candidate separation), stale permission/view (check each stage and replay), unsupported service or recipe version (explicit incompatibility), needless complexity (Gate A stop condition), regression (exact-head CI and native smoke), archive incompatibility (atomic migration and backup). No new external dependencies planned; assess license/version/target impact if later justified. Webb-Database remains private reference only, no code/data transfer or runtime dependency.

## 8. Alternatives and acceptance

A: existing queries unchanged. B: context actions/presets over existing code, cheapest and chosen baseline. C: bounded reusable composition, approved conditional scope where B insufficient; only adds workflow controls/provenance/definition versioning. D: standalone engine or general script/plugins, rejected. Research #69 measured terminal-only SQL faster than chain retaining intermediate results: no performance advantage established. Measure actual native repeated tasks fairly against B, not weaker mock query UI.

All WT-01–10 from [#70](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70) remain unpassed. Test contracts:
- WT-01: latest baseline CI, three actual workflows, measured repeatable gap and agreed limits or stop.
- WT-02: compatible typed sequence, selection arity/type/world, operation/query pins, missing/incompatible refs and malformed/excessive input rejection.
- WT-03: read-only exact scoped parity vs underlying query/services; repeatable IDs/direction/role/source and distinct assertions; no world mutation, no fabricated edge.
- WT-04: valid-time boundaries, corrections, pinned fork and sibling isolation, uncertain/derived/claim/candidate separation; stale result discarded.
- WT-05: dense fanout, duplicate paths, global call/row/byte/visited/edge/deadline caps, cancellation/errors/partial without false completeness.
- WT-06: authoritative capability and author-only execution, forged/wrong scope/context, unavailable/hidden node/edge/source, result reuse/export and noninterference with hidden data; no bypass through publication or Canvas.
- WT-07: accessible native curated sequence, ledger/evidence navigation, ordered editor, save/version/replay, different selection/day, view-only undo, fair three-task author comparison; stop expanding if benefit absent.
- WT-08: atomic definition save/version, backup/migration if applicable, reopen and archive restore identity/pins, missing version errors, no copied canon or raw author publication.
- WT-09: actual native dense/sparse 1k/10k/50k fixtures, memory/work/latency/truncation, offline fallback and exact-head CI/capture; no SLA inferred from research.
- WT-10: verify all scope, docs and dependencies, owner review before closure; existing INV/QRY/AT/SP remain independently required.

Map WT-02/03 to INV-01/05/07/11/13 and QRY-01/AT-01/03/08/26; WT-04 to INV-02/03/04/06/14 and AT-02/09/11/14–16/21/35; WT-05/06 to INV-12/15, QRY-03 and AT-13/20/23/25/34; WT-07/08 to INV-09/10/13/16, QRY-02 and AT-19/22/24/27/33; WT-09/10 to AT-31/32/36 and applicable #68 SP-02/04–06/08/11–14 only when spatial integration present. These are dependencies/reuse, not new original phase gates. No research fixture passes any production gate.

## 9. Reference and approval record

[Owner acceptance](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70#issuecomment-7073776197) approves #70 boundaries and readiness condition. [#69 study](../Research-Studies/maltego-inspired-transforms.md) contains verified Maltego documentation, reviewed Webb-Database source links and exact synthetic [fixture](../Research-Studies/transform-prototype/check.py); all research criteria closed. #69 rerun demonstrates typed historical/scope parity and bounded recipe feasibility only; sparse medians 0.042/0.044/0.045 ms vs terminal SQL 0.015/0.015/0.017 ms, unequal output work; no native UI trial, full security or dense test. [07](../07-Search-Query-and-Discovery.md), [01](../01-Requirements-and-Invariants.md), [02](../02-System-Architecture.md), [06](../06-Three-Layer-Entries-and-Canvas.md), [08](../08-Storage-and-API-Contracts.md), [09](../09-Acceptance-and-Verification.md), [10](../10-Delivery-Roadmap.md), [12](../12-Decisions-and-Reconciliation.md), [#68 specification](nested-locations-and-procedural-vector-cartography.md). Acceptance is documentation only; no production changes or verified results at approval.
