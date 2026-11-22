# Comparative worldbuilding workspaces and encyclopedia UX

**Status:** In Review — comparative desk research and source inspection performed; native authoring comparison and owner decision outstanding.
**Date:** 2026-11-22 UTC. **Reviewer:** Lola. **Research:** [#76](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/76).
**Atlas baseline:** schema 44 [30b6710](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/30b6710a583550df68e8ba88cb26c6ad01b26e56), draft #75; main 5777f0f schema 36. Both branch heads unchanged when checked. Exact head CI runs 45019544281 and 45019544285 green (6 jobs). This is evidence of Atlas's tested subset, not an all-feature usability certification.

## 1. Decision and method

Question: which patterns of Notion, Realm Works, Kanka, World Anvil, LegendKeeper, Obsidian, Obsidian Portal and Campfire can make our typed historical world model accessible without creating another engine? Compare against the [interaction audit](../UI-UX-Audit-2026-11-22.md), [visual audit](../UI-Visual-Design-Audit-2026-11-22.md), [function audit](../Functionality-Audit-2026-11-22.md) and [encyclopedia audit](../Encyclopedia-Audit-2026-11-22.md). Requirements 01, 06, 07, 09, 13, ADR-007 and approved #68/#70/#72 govern scope. Full recursive Atlas inventory (267 entries), README, research index, screenshot guide, open/closed tickets, current CI, relevant code and prior audit evidence were reviewed. Four audits are pinned to the same candidate rather than treating historical main snapshots as the latest state.

Methods: opened vendor feature/help pages and original Realm Works announcement; followed public showcase/demo links and inspected retrieved page structure, headings, navigation, links, examples and metadata; inspected Kanka upstream repository tree and eight actual files at exact commit a291b50ee4d5a2ced469c4cca3747ccc125174a4. All mutable sources accessed 2026-11-22 unless otherwise stated. No accounts created, software purchased, private data uploaded, or external content incorporated into production. Source evidence classes: **D** documented/vendor description, **O** observed public page structure or link traversal via retrieved page (not rendered interactive usability), **S** inspected source, **A** prior Atlas native/test evidence, **U** unknown/unverified. No external native binaries or authenticated authoring sessions executed; no measured human task timing, accessibility certification, current full plan parity or external app internal security conclusions. Do not equate missing documentation with missing implementation. No fresh Atlas UI changes, screenshots or prototype presented as production proof.

## 2. Comparative findings

### Notion — compositional authoring and query presentation

[D: R1–R4] Blocks can be inserted with a keyboard menu and rearranged; saved database views choose layout, property visibility, filtering, grouping and sorting, and records open as side, center or full-page views. Related record properties are displayed and summarized via relations/rollups. Notion's current offline help explicitly describes individual downloads on desktop/mobile, first 50 rows of a downloaded database's first view, and permission changes requiring connection. Offline capability exists, but is not synonymous with a wholly local authoritative store. Do not rely on outdated claims that it has no offline support.

Atlas fit: LayoutElement and stable ElementId already give the basis for a writing-first block interaction; SavedLabelQuery and parameter binding already provide a typed starting point for views. Implement editing in place, small component insert palette, preview/peek in context. One query runs through Rust, presentation selects compact list/table first; gallery/timeline only when authorized DTO fields exist. Do not create secondary Notion-style user schemas or formula runtime. Treat rollups as explicitly authorized derived computations with time, cardinality, unknown/truncation and provenance, not a blanket user-programmable database. Distinguish selection of view columns from read permission.

### Realm Works — structured knowledge and granular reveal

[D: R5–R7] Historical categories specify topic/article sections and snippets. Product documentation describes content linking, storyboards, map navigation, full-text search and Fog of World. Designer diary describes linked story/topic hierarchies and separate GM/player map variants. Adopt the principle of granular document content and navigable knowledge hierarchy, not the proprietary format or product. Categories/templates are presentation unless a governed classification supplies canonical meaning; links are not facts. A published snippet may refer to a hidden target but must not expose it.

[D: R8] Sept 28 2019 owner announcement suspended product development, citing resources/commercial sustainability; historical promise to maintain servers is not a current uptime guarantee. Product page describes Windows installer/account activation and server access for sharing to Player Edition. Current binary installation, activation and service availability untested; no claim of complete shutdown. Lesson: focus scope, local authority, documented file-format recovery and export, no dependency on proprietary licensing service. Player reveal is not a historical character's knowledge. Keep author/public/character and effective day/revision separate. Explicit preview and safe directed release can be implemented under existing policies; arbitrary audience-group/reveal-state engine requires scope review.

### Kanka — domain organization and shared identity

[D: R9–R11] Configurable module visibility, dashboards, rich-text mentions, relation management, eras/timelines, multiple calendars and map layers/pins, campaign/entry/attached-content permissions. Feature page distinguishes free core from premium visual relationship exploration/family trees and multiple role dashboards; do not claim all these are free. API publishes separate object endpoints, entities, mentions, posts, connections, dashboard widgets and pagination/visibility contracts.

[O: R12] Axbrid public dashboard contains grouped World/Time/Game/Other sidebar destinations, bookmarks, onboarding tips and recently changed entries. Retrieved snapshot includes July 2025 maintenance notice; treat as older cached interface observation, not current screenshots or uptime. Characters navigation link was present, but destination retrieval failed (cache miss). No logged-in editing.

[S: R13] Reviewed exact owlchester/kanka develop commit a291b50ee4d5a2ced469c4cca3747ccc125174a4. Entity.php:133–193 holds common fields and typed morph child; Relation.php:110–162 models owner/target and explicit mirror relation; CampaignDashboardWidget.php:56–130,169–281 exposes entity/type, dashboard, tags, layout positions and widget configuration; HasMentions.php:25–75 separates outgoing and incoming mention edges; EntityPolicy.php:19–112 and RelationPolicy.php:19–59 separate policy checks. This is partial structural inspection, not proof of temporal, privacy or transactional behavior. LICENSE is Commons Clause condition, with no right to sell covered software. Public source is not automatically unrestricted open-source code. No copying/reuse of implementation or branding. Conceptual transfer: one common identity reference; focused type modules; relation summaries, dashboard views, linked and unlinked mentions; policy before data leaves trusted host. Atlas already has stronger intended historical invariants; keep a single fact identity with reversible directional presentation, not physically mirrored independent facts. Use workspace filters, not disabling domain data/foreign keys.

### World Anvil — encyclopedia editorial depth

[D: R14] Product navigation offers worldbuilding templates, maps, timelines, chronicles, family trees, charts, search, auto-linking, access and publishing. Treat plan-specific availability as unverified unless explicitly documented; no complete tier comparison claimed.
[O: R15/R16] Opened Cathedris public homepage then followed a city-state article link. Home has a setting primer, curated entry points, named types, recent articles and additional exploration links. Destination has hierarchical headings, illustrated geography, alias/location metadata, narrative history, timeline, quotes attributed to in-world voices and further links. Strong editorial example; no assertion that its timeline or content is canonically computed from typed facts. Attribution/license on homepage retained in source, no copying of art/layout/text. Atlas should allow an authored starting page, visible subject categories, article title/overview, context-linked infobox, heading TOC, citations, curated related entries. Use existing Canvas templates and authorized Display output (ENC-01/02/03/04/07/08); maps and live relations only once their scoped APIs meet requirements.

### LegendKeeper — seamless geography and prose navigation

[D: R17] Vendor describes linked lore, private secrets, reusable templates, offline-then-sync/export and nested pinned maps. Not independently verified that all offline/export modes preserve complete historical semantics. [O: R18] Opened publicly accessible Selenia demo; project search with Ctrl+K hint, project/map switching, name/tag filter, hierarchical entries and Lore/Map/History tabs. Followed NPC directory link and retrieved destination [R19]. No actual map pan, editable state or offline session executed. Adopt the continuity of selection and contextual map/article switching. Map pins and document references are not canonical spatial containment, coordinates or routes; exact/schematic and unplaced distinctions from #68 remain mandatory. Do not build another atlas database or raster-only geometry authority.

### Obsidian — local records, links and multiple views

[D: R20–R23] Bases specifies views over Markdown note properties, including list/table/cards and other view types; link documentation supports wiki syntax, autocomplete and rename update; Backlinks distinguishes linked and unlinked mentions; local Graph view has depth, filters and direction/display controls; Canvas provides a freeform visual note board using JSON Canvas. Atlas can adopt command search, linked/unlinked reference inspector, stable target selection and bounded local neighborhood. Use existing typed Rust facts rather than making filename references authoritative or mistaking document links for social/causal edges. Obsidian Canvas is freeform board, our Canvas is responsive document composition: do not conflate. No plugin runtime or competing Markdown canonical store. Consider safe Markdown/HTML export within IMP-01, not authoritative dual-write.

### Obsidian Portal — reader-facing world entry point

[D: R24/R25] Campaign website organizes wiki, characters, adventure logs and scheduling; help documents GM-only wiki page visibility. Fictional calendar, author revision and real session date must not be conflated. [O: R26] public listing showed Sylvangarde and The Final Reaping campaign links; both specific campaign homepage retrievals blocked by site human-check. No theme-level or private workflow observations claimed. Transfer: welcome/primer, named wiki sections, people directory, chronicle/entry navigation and a consistent landing point distinct from editing chrome. No session scheduler, player matching or RPG rules manager needed in present scope.

### Campfire — modules and configurable panels

[D: R27/R28] Authoring ecosystem includes novel/story and worldbuilding modules for characters, geography, species, timelines/calendars, culture/religion, philosophy, magic, items; documented movable/resizable panels, collaborator modes and spoiler controls. Limited free authoring and paid expanded access noted on vendor homepage, no purchases or account tests. Vendor showcases published books with world material. Transfer only the useful interaction pattern: profiles with tailored default panels, optional sections, templates that can be customized without adding a canonical type per subject. Ontology and scoped components remain distinct. Avoid expanding into novel writing, marketplace, collaboration SaaS or rules simulation.

## 3. Capability matrix

D/O/S are external evidence, not proof of backend parity. U means unknown/not established by inspected evidence, not absent. Atlas column A/S refers to candidate code and prior exact-head verification.

| Product | Structure and linking | Authoring and presentation | History / geography | Disclosure / portability |
|---|---|---|---|---|
| Notion | D: pages, blocks, relations, rollups | D: block editing, alternative views, peeks, filters | D: timeline/calendar views; effective-date fact/fork semantics U | D: sharing, limited downloaded offline works; full local authority U |
| Realm Works | D: categories, topics, snippets, auto-links | D: topics/articles, storyboards, navigation | D: maps/story structure; historical fact branching U | D: Fog of World, player delivery requires server; 2019 development suspension; current service U |
| Kanka | D/S: common entities, categories, relations, mentions | D/O/S: dashboards, entry editor, type navigation | D: eras, calendars, layered pin maps, premium family trees; exact temporal fact history U | D/S: roles, entry/child policies, API; full local offline U |
| World Anvil | D/O: categorized articles, links | D/O: editorial templates, public worlds | D/O: map/timeline/family tools, authored timeline example; unified dated facts U | D: access/publishing; full offline/history archive U |
| LegendKeeper | D/O: linked wiki and project tree | O: search, tabbed lore/map/history | D: nested pinned maps; canonical geometry/forks U | D: secrets, offline sync/export claims; full scope parity U |
| Obsidian | D: Markdown, properties, wikilinks, backlinks | D: Bases views, editor, freeform Canvas | D: local link graph; typed dated world-model U | D: local files; Atlas-equivalent reader scope U |
| Obsidian Portal | D: campaign wiki and characters | D/O: campaign listing, reader website; example detail inaccessible | D: adventure log and session scheduling; fictional temporal model U | D: GM-only page; offline and corpus export U |
| Campfire | D: specialized modules and panels | D: customizable writing/worldbuilding, publish | D: maps/calendars/timelines; typed history/fork semantics U | D: spoiler controls, collaboration; complete local export U |
| Atlas | A/S: stable typed 22 kinds, 24 fact kinds, historical visibility and recorded revisions | A/S: plain Text, nine built-in + custom pinned layouts, separate search/forms, reader projection | A/S: calendars/events/causal/parentage/containment foundations; GUI and map incomplete | A/S: local SQLite, scoped projection, archive; public dynamic component and offline full product gates outstanding |

Competitive differentiator is the *combination* of historically versioned, source-linked typed relations, explicit character perspectives and a single world model consumed by composition and reader. No verified claim that all competitors lack those capabilities. Our integration/usability currently lags the demonstrated visual and organizational patterns; avoid feature-count rankings.

## 4. Observed public workflow and limitations

| Task / equivalent endpoint | Observation | Count/verification boundary |
|---|---|---|
| Discover a world then open named article | World Anvil official showcase -> Cathedris home -> Russin article retrieved | Two followed links from vendor home; article includes location metadata, headings and further internal links. No stopwatch. |
| Browse project by subject | LegendKeeper vendor demo -> Selenia -> NPC directory retrieved | Two followed links after example landing; public search and map tab present, action beyond read-only navigation untested. |
| Discover modular entries | Kanka Axbrid dashboard -> Characters destination attempted | One link attempted, destination unavailable; sidebar groupings observed, no successful character list claimed. |
| Find a public campaign | Obsidian Portal listing -> two campaign links attempted | Both failed human-check; no bypass. |
| Add/rename linked entity, create sourced relationship, publish/hide, rearrange block | Vendor docs describe mechanisms, Atlas prior native/mocked tests cover selected actions | No equivalent cross-product authenticated editing access. No comparative click count, time, error or success-rate invented. |
| 50k collection, offline reopen, authorization under branch change | Atlas exact-head store scale and native smoke evidence, documented external features only | Not comparable across products; no external benchmark. |

Paper task decomposition (design hypothesis, not observed timings): current Atlas locate world -> set day -> query entity -> open record editor -> add separate link/component -> select target -> save -> open preview. Proposed keep one workspace context, open article directly from search, insert link with compatible target picker, save draft, preview release in current allowed context. Compare actual action count, context switches, validation errors and preserved context on same Alder Basin tasks before claiming time savings. Use ordinary saved queries and contextual actions as #70 comparator, not a deliberately handicapped workflow. The expanded Alder Basin fixture 91b1e18 remains staged without exact-head verification; original 8a451ac passed six jobs but does not establish linked article corpus. Build/verify article fixture before measuring.

## 5. Adoption register and architecture impact

Effort: S = localized existing control/DTO work; M = multi-surface interaction plus tests; L = substantial new projection/layout/interface contract plus migration/security review. Relative estimates, not schedules. A = adopt principle, Adapt = reinterpret on existing semantics, Defer = prerequisite, Reject = contrary to goals. All recommendations pending review, not approved code.

| ID / disposition | Adaptation, impact and existing ticket | Verification / risk / effort |
|---|---|---|
| RQ-01 A | Context bar + explorer grouped by existing kinds, focused workspaces, contextual create/palette. Kanka/Obsidian/Notion. UX-01/03/05/06/14, F-01/02/03/04, ENC-01. index.html, author-workspace.ts, network-studio.ts, workspace.rs. #4/#5/#8/#72 | Same stable selection across views, pinned context and revision-safe create, all 22 supported kind filters; unknown/unsupported visibly disabled. No new schema. M |
| RQ-02 A | Quick search and side peek / full page editing, back/forward and keyboard focus. Notion/Obsidian/LegendKeeper. UX-07/15, ENC-02/09/11. entry-editor.ts, public-reader.ts, author-workspace.ts. #4/#5/#8 | Preserve search/selection and dirty draft, reauthorize navigation; no stale public exposure. M |
| RQ-03 Adapt | Minimal writing-first block insert/menu and reparent, semantic headings/inline text links/Citation. Notion/Realm Works/Campfire. ENC-03/07/10/12, UX-02/08. canvas-tree.ts, entry-editor.ts, layout.rs, store/entries.rs. #5/#4/#8 | Stable ElementId, preserve old Text and layout pins, no raw HTML/JS, compatible target ambiguity/citation scope, undo/reopen. Template/schema changes require reviewed compatibility. L |
| RQ-04 Adapt | Saved views over existing query definitions, list/table first, side preview. Notion/Obsidian Bases/Kanka. F-04/07, UX-03/04/09, ENC-09. query.rs, workspace.rs, author-workspace.ts. #4/#5/#72 | Same result IDs for identical context; renderer-only changes do not write world data. Bounded page/cancel; unsupported output explicit. M |
| RQ-05 Adapt | Canonical infobox/read projections and small explicit rollups derived from typed data. Notion/World Anvil. ENC-07/08, F-05. layout.rs and Store services. #5/#6/#8 | For first approved membership summary, same scoped fact IDs/labels at chosen day; source-aware, unknown vs zero, truncation indicator, hidden-edge/count noninterference. No generic formula/EAV. L |
| RQ-06 A | First-class encyclopedia home, directory, topic browsing, authored primer, title/heading/TOC, display style. World Anvil/Obsidian Portal/Realm Works. ENC-01/02/04/07/11/14; V-01/03. public-reader.ts, entries.rs, layout.rs. #5/#4/#8 | No page title or category count leak; current release only until scope support verified, link/revoke safe, export same authorized DTO. M/L |
| RQ-07 Adapt | Grouped authored links/mentions/canonical facts/discovered candidates; inline suggestion and local neighborhood. Realm Works/Obsidian/Kanka. UX-04/06, ENC-10. discovery.rs, query.rs, canvas-tree.ts, network-studio.ts. #4/#5/#6/#72 | Stable reference and exact text span/revision; alias collision picker; reviewed link never implies fact; 50k bounded traversal. M/L |
| RQ-08 Adapt | Contextually integrated modules and reusable profiles/templates. Kanka/Campfire/Realm Works. UX-05/14, ENC-04, F-03. canvas-templates.ts, store/templates.rs. #5/#6/#7/#72 | Existing semantic kinds, author can hide irrelevant tools without deleting facts; template update pinned/explicit. Prefer custom template over new object kind. M |
| RQ-09 Adapt | Audience preview and explicit reveal of selected permitted content. Realm Works/Kanka. UX-01/10, ENC-05/06/08, F-01/05. access.rs, layout.rs, store/entries.rs, workspace.rs. #8/#5 | Author/public/character and effective date vs author revision explicit; same authorized search/backlinks/export, not CSS hiding. Existing scoped policy only. General participant roles/reveal state deferred pending review. L |
| RQ-10 Defer until prerequisites | Article–map–location navigation, nested geometry, timeline and family visual views. LegendKeeper/World Anvil/Kanka. UX-04/13, ENC-08/13, F-04/06. #68/#6/#7/#5 | SP gates, exact/schematic/unknown, local coordinate frames, typed genealogy and provenance. No second storage system. L |
| RQ-11 A | Portable local workflow, clear archive vs reader output, offline no services. Obsidian plus Realm Works sustainability lesson. F-08/ENC-14. archive.rs, store.rs, Tauri bundle config. #10/#8 | Native install/recovery, asset integrity and authorized projection export; no proprietary dependency. M/L |
| RQ-12 Reject/defer | SaaS collaboration/marketplace, arbitrary custom schema/formulas/plugins, campaign scheduling, manuscript platform, freeform board as default editor, broad recipe runtime. | Not required to deliver agreed tasks. Preserve WT-01 and independent feature approval; stop rather than duplicate services. |

## 6. Recommended workbench

Three layers stay intact; not a fourth encyclopedia store. Proposed information architecture:

```text
World | fictional date | branch | revision | audience | save status
Search / create           Node | Canvas | Display
Explorer / saved views    Active workspace              Contextual inspector
- People                 Node: focused graph/list       Typed properties/facts
- Families               Canvas: editable article      References/evidence
- Institutions           Display: authorized reader    Contextual actions
- Places                 Same selected ObjectRef       No raw IDs by default
- Events, concepts...
```

Default common workflow is browse -> select -> view/edit in place -> reference -> save -> authorized preview. Keep technical diagnostics optional. Use design tokens from visual audit (#101B20 shell, #18272D surface, #D8B77A accent; system font/editorial serif reader). Workspace presets are presentation filters, not data-model definitions. Repeatable views refer to existing query ID/version and selected context; where unsupported, explain rather than fabricate empty results. Proposed pseudocode boundary (not new approved API):
```text
context = validated existing ViewContext + allowed capability
rows = Rust existing query / focused service(context, budget)
authorized = existing scoped projection(rows, context)
render list/table/approved component from authorized DTO
write only by existing or separately reviewed typed command(expected revision)
```
Real implementation must authorize before serialization, and aggregate only within allowed projection; this diagram is not permission to use unrestricted author rows for reader. Do not infer facts from link text, layout placement or hidden related counts. Keep semantic, presentation-reference, discovery, candidate and claim distinctions. For reader, current release and selected fictional day are different things; historical prose not magically rewritten. Explicitly distinguish current publication from date-limited sections.

## 7. Follow-up and closure

1. Preserve green #75 for owner integration review; reconcile independent branches in dependency order. Research does not authorize merging or closing AE gates.
2. Complete revision-safe creation and dirty/context guards, then shell/explorer/quick search/compact controls (RQ-01/02). Update existing #4/#5/#8 tasks rather than create duplicates. 
3. First encyclopedia vertical slice: one authorized title/directory/article navigation flow; minimal block writing, inline stable link and citation, default template. Verify in current layout compiler, no general schema engine.
4. Saved views and one explicit bounded live binding, only after scope works. Compare plain list/table before adding arbitrary view and aggregate options; stop if simpler path meets need.
5. Finish #6/#7 and spatial prerequisites before visual genealogies/maps; #70 remains conditional. Portability and UI acceptance continuously.
6. Native comparative author tasks on Northbridge and validated expanded Alder Basin: create duplicate identities, link correctly, create person/house/place/source article, record dated sourced membership, preview author/public/character, unpublish, reopen, move bound component, restore. Capture exact build and actual actions/time/errors, keyboard, 460/1080/1920 widths, offline and no-GPU, 1k/10k/50k mixed corpus. No fabricated user-study results. Need actual interactive access to compare all external editor actions; if not available, keep as documented-only and owner review scope. 
7. Report sources, evidence and adoption disposition to owner. Study compiled, eight products covered, public reader navigation attempted and selected source inspected. Full plan/interactive/benchmark comparisons and owner acceptance not finished; #76 stays open. No production changes, no new ADR or feature ticket required solely from recommendation.

## References

All accessed 2026-11-22 (mutable documentation unless dated). Source claims paraphrased; no reproduction of proprietary material.

- [R1] Notion, *What is a block?* https://www.notion.com/help/what-is-a-block
- [R2] Notion, *Database views, filters, sorts & groups*. https://www.notion.com/help/views-filters-and-sorts
- [R3] Notion, *Database relations & rollups*. https://www.notion.com/help/relations-and-rollups
- [R4] Notion, *Use Notion pages offline*. https://www.notion.com/help/use-pages-offline
- [R5] Lone Wolf Development, *Realm Works*. https://www.wolflair.com/realmworks/
- [R6] Lone Wolf Development, *Game-Specific Content Structures*. https://www.wolflair.com/realmworks/designers-diary/004-game-specific-content-structures/
- [R7] Lone Wolf Development, *Making the Most of Official Content*. https://www.wolflair.com/realmworks/designers-diary/003-making-the-most-of-official-content/
- [R8] Lone Wolf Development administrator, *An Update from Rob*, 2019-09-28. https://forums.wolflair.com/threads/an-update-from-rob.63258/
- [R9] Kanka, *Worldbuilding Tool Features*. https://kanka.io/features
- [R10] Kanka, *Permissions*. https://docs.kanka.io/en/latest/features/permissions.html
- [R11] Kanka, REST API navigation/modules. https://app.kanka.io/api-docs/1.0/campaigns/modules
- [R12] Axbrid Kanka public dashboard (retrieved older cached content). https://app.kanka.io/w/278068
- [R13] Owlchester, Kanka, exact develop commit [a291b50](https://github.com/owlchester/kanka/tree/a291b50ee4d5a2ced469c4cca3747ccc125174a4); inspected LICENSE, app/Models/Entity.php, Relation.php, CampaignDashboardWidget.php, Concerns/HasEntity.php, Concerns/HasMentions.php, app/Policies/EntityPolicy.php, RelationPolicy.php. [License](https://github.com/owlchester/kanka/blob/a291b50ee4d5a2ced469c4cca3747ccc125174a4/LICENSE).
- [R14] World Anvil, official feature navigation. https://www.worldanvil.com/
- [R15] Stormbril, *Cathedris* public world homepage. https://www.worldanvil.com/w/cathedris-stormbril
- [R16] Stormbril, *The Coastal City-state of Russin*, public article. https://www.worldanvil.com/w/cathedris-stormbril/a/the-coastal-city-state-of-russin2C-russin-article
- [R17] LegendKeeper, official site. https://www.legendkeeper.com/
- [R18] LegendKeeper, example project and Selenia reader. https://www.legendkeeper.com/example-project/ ; https://www.legendkeeper.com/p/cl98b4ybqjd8909902o0vb1cl/cku5kwapk000203ccuwml4023/cl98b4zvs0003039ciskujx5c/
- [R19] LegendKeeper, Selenia NPC directory. https://www.legendkeeper.com/p/cl98b4ybqjd8909902o0vb1cl/cku5n6gwz007903cciyx7z03b/
- [R20] Obsidian Help, *Introduction to Bases*. https://obsidian.md/help/bases
- [R21] Obsidian Help, *Internal links*. https://obsidian.md/help/links
- [R22] Obsidian Help, *Backlinks* and *Graph view*. https://obsidian.md/help/plugins/backlinks ; https://obsidian.md/help/plugins/graph
- [R23] Obsidian Help, *Canvas*. https://obsidian.md/help/Plugins/Canvas (lowercase /canvas failed)
- [R24] Obsidian Portal, official home. https://www.obsidianportal.com/
- [R25] Obsidian Portal Knowledge Base, *Players Can't Edit the Wiki*, last updated 2020-05-14. https://help.obsidianportal.com/article/157-players-cant-edit-the-wiki
- [R26] Obsidian Portal, public campaigns. https://www.obsidianportal.com/campaigns ; attempted https://king-melchior.obsidianportal.com/ and https://thefinalreaping.obsidianportal.com/ (human-check blocked).
- [R27] Campfire, official homepage. https://www.campfirewriting.com/
- [R28] Campfire, worldbuilding modules and panels. https://www.campfirewriting.com/worldbuilding-tools

## 8. Deeper implementation review — 2026-11-22

Opened additional official help on Kanka quick creation, LegendKeeper templates and map tools, World Anvil article templates/categories/secrets, Notion database templates/view opening, Obsidian Bases view configuration and syntax, and ProseMirror document/transaction architecture [R29–R39]. These are documented workflows, not observed logged-in author actions. The older community-maintained World Anvil Codex explicitly warns of outdated material; use current Learn documentation in preference. The current Secrets guide independently confirms that a secret-associated tab is visible even when the viewer cannot read any of its contents. This is a documented design distinction, not a claim of exploit or current implementation testing.

### 8.1 Four independent structures, not one tree

World Anvil documents nested article categories, parent articles and tags; Kanka quick creation creates referenced entities without navigating away, while LegendKeeper can select templates during mention creation. [R29–R33] These suggest that presentation organization, identity, reference, and real-world containment must be separate. Example: Alder Guild can appear under Institutions, in Trade, under a curated chapter about Riverhold, and on an authorized map. None of those navigation choices establishes territorial jurisdiction, physical location or membership.

- Canonical typed graph: existing ObjectRef and validated temporal fact identity.
- Explorer grouping: initial allowlisted kind filters, saved queries and selected labels (no persistence expansion required for first slice); later curated collection references only after demonstrating need and reviewing storage. A category folder is not a Place.
- Canvas document: LayoutElement tree, stable ElementId, versioned pinned templates. A section may reference other identities but is not an entity automatically.
- Authorized reader TOC/breadcrumbs/backlinks: derive entirely from released, scoped output. Do not include hidden category names, private article counts, placeholders for secret sections or inaccessible ancestor paths.

A topic hierarchy should allow one identity to appear through multiple noncanonical navigation routes instead of duplicating records. No general multi-parent folder engine before the simpler directory/saved views work. No change to #68 physical/admin containment contract.

### 8.2 Writing-first creation and typed snippets

Kanka quick creator documentation describes a persistent create entry action/keyboard shortcut and bulk name entry; LegendKeeper help describes creating template-backed pages during mention composition. World Anvil's tailored prompts are optional and do not force authors to fill every field. [R29–R31] Notion templates provide repeatable record/page structure and side-peek opening [R34/R35]. Transfer: the smallest useful entry can be a named identity before anything is known about it. Allow create action from a compatible picker without losing text, with confirmation of kind and date semantics; preserve stable ID and caller revision. If incompatible, refuse or leave as unlinked text; never automatically create a fact.

Inspect current Atlas layout.rs:1–109, 548–700, access.rs, canvas-tree.ts:27–75, canvas-templates.ts:1–54. Version 1 LayoutElement has ElementId, audience and content. Global max 512 elements, depth 16, 16 queries; text 16384 bytes; template v1 and custom pin; new blocks Author; reparent in layout.rs restricts inherited permission, and frontend records bounded 32 snapshots. Keep those constraints. Plain Text encodes no headings or inline reference identity. Three alternatives:
1. Existing Text + ObjectLink blocks with improved layout editor. No compatibility burden; fastest shell/authoring improvement; not adequate for genuine in-paragraph links/TOC.
2. Minimal structured typed runs in new supported component variant (recommended research candidate): heading levels 2–4, paragraph runs text / object link / citation; optional emphasis limited to explicit validated marks. Block ElementId persists; references at run level may need stable IDs for indexing and navigation. No nested arbitrary DOM, raw user markup or arbitrary executable code. Existing Text and ObjectLink remain readable and not implicitly upgraded. Review schema/version policy, serialized size/depth/element limits and backward compatibility before production.
3. Full editor schema/runtime library, e.g. ProseMirror documents, steps/transactions, mapping and NodeViews [R36]. Potentially useful for IME, clipboard, selection, history and structured editing once proven necessary. Do not choose it solely because mature products support blocks; map to authoritative typed Rust format, audit transitive dependencies/licenses, accessibility, WebView compatibility, zero-network packaging. No local integration test performed; defer library adoption until a small editor spike shows that existing DOM controls cannot meet required UX.

Pseudo-contract proposal, not approved wire format:
```text
Heading { level, text }
Paragraph { runs: [Text(text), Link(run_id, target ObjectRef, caption), Citation(run_id, source ObjectRef, caption)] }
DisplayParagraph = authorized projected runs only
```
Why runs rather than raw UTF-8 spans: collaborative editor positions, browser UTF-16 offsets and Rust byte offsets differ; an inserted Unicode prefix can invalidate offsets. Structured runs retain target reference outside unchanged link; full selection editing, Unicode/IME, split/merge, duplicate run IDs, paste, stale revisions and historical reference pins still need test. No hidden run caption fallback unless it is independently authored and explicitly authorized; removing link/citation run can alter grammar, editor must preview exact projection. The compiler and index must share authorized content. Maintain canonical facts separately. No silent rewriting or promotion of mentions.

### 8.3 Query definition versus view versus results

Notion views support side/center/full page opening, filters, sorting and grouping. Obsidian Bases explicitly distinguishes a base's data, per-view configuration, filters, ordering, grouping and embedded views [R35/R37/R38]. Current Atlas QuerySpec (query.rs:18–74) has version/filter/limit; PredicateInput is allowlisted, and saved binding pins world/query ID/definition revision with typed parameters (layout.rs:143–174). Dynamic queries are currently author-only (layout.rs:332–362, 687+). Do not pretend saved definitions already support arbitrary grouping, ordering or display columns. Start with existing result shape (ObjectRef,label) and list/table views; alternate styling needs no new query. A view descriptor can be ephemeral first, later persisted as presentation metadata if repeated-use evidence warrants; never store copied canonical rows.

```text
query definition + typed parameters + existing authorized context -> bounded rows + continuation
view options -> presentation of those rows (same selected ID)
whole result aggregate -> separate explicitly supported bounded server computation
```
If a gallery needs portrait, a timeline needs intervals, graph needs accepted edges, an infobox needs source-backed facts: obtain exactly those authorized fields through tested domain service, not guessed joins or frontend enrichment with author-only payload. The list renderer cannot count an entire paginated query from its first page; label that value as visible page subtotal, or explicit incomplete/unknown. Missing field != unknown fact != zero != hidden. Scope before count or grouping, not after; no hidden counts or ordering derived from unavailable fields. Use deterministic tie-breaking and cancellation. Only one first narrow membership-style projection should be reviewed under #5/#8 before arbitrary aggregation; no formula interpreter.

### 8.4 Fine-grained disclosure and visible-existence leaks

Current World Anvil Secrets documentation [R39] distinguishes the visible Secrets tab from the restricted content inside it. Atlas should deliberately reject a comparable existence signal in restricted mode, because INV-15 includes counts, titles, paths and conditional components. This is stronger than a mere editor setting. Current Atlas access.rs already intersects inherited scope: author sees all, public sees only public, character sees public and own character content; conflicting character constraints yield Author. Compiler prunes subtree before query evaluation; published layout gate denies ordinary dynamic query bindings. Reader current release and selected day are separate. No public arbitrary query exposure, and no actual production reveal-history subsystem. Realm Works player disclosure is not a permission template to copy literally. A new reveal timeline or audience group infrastructure would need approved semantics and owner review.

Required negative tests: insert author-only heading, nested author query, hidden link, private image hotspot; compare serialized reader page, TOC, count, search matches/snippets, backlink output and export byte content before/after. Revoke target page and recompile: remove link and secret caption and clear stale result/focus. Move private nested block into public container: must remain restricted, no type/ID change. Earlier effective day cannot silently change manually written prose. Revoke or invalidate cached page and reauthorize navigation; timing side channels are not solved just by equivalent DTOs. Distinguish author-facing secret indicators (allowed in author mode) from public reveal hints (deny).

### 8.5 Executed isolated semantic prototype

[check.js](workspace-projection-prototype/check.js), [actual results](workspace-projection-prototype/results.json), [method/boundaries](workspace-projection-prototype/README.md). Executed exact JavaScript function in V8 isolate. 25 checks passed. Nested headings and runs, separate scopes author/public/two fictional characters, explicit permitted links, half-open dates, synthetic row arrays. Source and results committed separately. No UI, rendering, SQLite, branch/revision resolver or actual policy calls, no performance claim.

| Raw synthetic rows | Authorized at day 12 | House A / B full oracle | First 50 subtotal A / B |
|---:|---:|---:|---:|
| 1,000 | 686 | 229 / 457 | 17 / 33 |
| 10,000 | 6,857 | 2,286 / 4,571 | 17 / 33 |
| 50,000 | 34,286 | 11,429 / 22,857 | 17 / 33 |

Each size verified identical list/card page IDs, exact authorized grouping, page != whole count, hidden-only injected row noninterference and half-open date exclusion. Nested document verifies hidden query not evaluated, public DTO unchanged by secret heading addition, distinct character headings, revoked target/caption omission, inherited audience after moving a block, stronger intersection of two character scopes, author visibility, and Unicode prefix insertion without retargeting structured link. This simple reference model illustrates why data-view polish cannot be implemented only by CSS or frontend filtering. No 50k runtime latency measured and no production gates satisfied; whole-row materialization here is an oracle, not suggested production algorithm.

### 8.6 Refined implementation sequence and stop conditions

- S0 (existing contracts): F-01/02 and UX-02 protected context/draft; one shared selection; complete kind filter and compatible picker. Test competing write, no implicit date shift and no draft loss. No schema.
- S1 (first useful workbench): persistent explorer, search/open/peek, compact list/table, directory and authored article title; existing layout renderer and current explicit release. Test native at 460/1080/1920, keyboard focus, duplicate names, 50k focused operation. Do not build custom dashboards yet.
- S2 (encyclopedia writing): improve block editing using existing stable layout; narrow heading/inline link/citation trial, explicit compatibility check before any new layout version. Keep text-only diagnostic fallback; no library chosen by this study. Compare editing 10, 100, 512 blocks, insert/move/undo/save/reopen, IME, copy/paste, source selection. Check no leaked body or captions. No arbitrary contenteditable HTML as canonical authority.
- S3 (derived presentation): one approved typed membership read view and explicit bounded count if needed; list/table equality then optional cards/timeline; context/date/branch/audience pin and parity against current Rust services. Do not adopt aggregation if author just needs existing member list, or enable public components before Rust policy validated. 
- S4 (specialized exploration): domain views and maps from #6/#7/#68, portable reader output from #10; #70 still requires comparative task gap; no new feature/architectural evolution ticket solely from precedent.

Prioritize actual workflow improvement and existing defects over new frameworks. Source-review/isolated evidence now deeper, external interactive authoring benchmarks, native prototype and owner review still open; no production changes or capture applicable to this numerical test.

### Additional references (all accessed 2026-11-22)

- [R29] Kanka, *Quick Creator*. https://docs.kanka.io/en/latest/features/quick-creator.html
- [R30] LegendKeeper, *Help*, Templates and Maps. https://www.legendkeeper.com/help/
- [R31] World Anvil, *How to Use Article Templates*. https://www.worldanvil.com/learn/article-guides/article-templates
- [R32] World Anvil, *Feature Guide to Articles*. https://www.worldanvil.com/learn/article-guides/articles
- [R33] World Anvil, *Guide to the Old Version Edit Interface* (explicitly historical interface). https://www.worldanvil.com/learn/article-guides/guide-articles-editing
- [R34] Notion, *Using database templates to help cement your team's process*. https://www.notion.com/help/guides/using-database-templates-for-teams
- [R35] Notion, *Database views, filters, sorts & groups*. https://www.notion.com/help/views-filters-and-sorts
- [R36] ProseMirror, *Guide*. https://prosemirror.net/docs/guide/
- [R37] Obsidian, *Views*. https://obsidian.md/help/bases/views
- [R38] Obsidian, *Bases syntax*. https://obsidian.md/help/bases/syntax
- [R39] World Anvil, *Feature Guide to Secrets*. https://www.worldanvil.com/learn/secrets/secrets

## 9. Lifecycle, terminology, template semantics and reusable knowledge — further research

**Date:** 2026-11-22. Same pinned schema-44 candidate 30b6710, main 5777f0f and research #76, audited tree 267 entries. Reopened README, evidence guide, study/index, existing ADRs for custom templates, typed saved parameters and linked encyclopedia, current Store templates/entries/query and desktop editor/reader/IPC. No change to frozen candidate. New primary sources R40–R51 below; documentary research, not authenticated editor trials. Executed isolated JS experiment in a V8 isolate, exact source and results preserved.

### 9.1 The vocabulary is part of the architecture's usability

[D R40] On March 10, 2026 Kanka described confusion caused by specialized terminology and renamed Entities to Entries, Posts to Articles, Attributes to Properties, Attribute Templates to Property Kits, Entity Templates to Archetypes, and the Assets subpage to Media. [D R41] Obsidian exposes properties as atomic structured information in a GUI with keyboard navigation and optional source view. These show that a powerful data model need not be exposed in its implementation vocabulary. Atlas should not rename Rust or database types for the sake of copy.

Recommended visible names: ObjectRef = Entry reference; QuerySpec = saved search/filter; ClaimAudience = who can see this; AuthorRevision = recorded version; LayoutElement = content block; parameter binding = saved filter value. Context must preserve distinctions: fictional date versus recorded version, article versus canonical entry, evidence versus established fact, draft proposal versus accepted relation, classification versus template. “Node/Canvas/Display” can remain the existing studio names with explanatory subtitles Explore, Write & design, Read & publish; do not create another fourth layer. Keep technical IDs available in diagnostics/reference disambiguation. No invisible semantics changes to improve wording. Test first-use tasks with plain labels and explicit capability explanations, then expert keyboard actions. UX-06/13/14 and F-02, #4/#5/#8/#72.

### 9.2 Separate structure, content, values, and live references

[D R42] Notion distinguishes database templates (new copies with initial content/properties; a preset relation can deliberately refer to the same existing record for every new page) from synced blocks (live shared content, original-access requirement, separate unsync; original deletion can affect copies). [D R43] LegendKeeper templates initialize new pages. [D R44] Kanka property kits add repeatable properties on save; reapplication after edits is manual, nested kits can include children, and category auto-application applies to ordinary form creation but not quick creator/new mention. [D R45] World Anvil custom templates alter layout/default content of existing types, not arbitrary text fields, and are tier-limited. [D R46] Campfire's panels can be rearranged, added or removed, with default domain templates. These are distinct reuse contracts, not interchangeable “inheritance.”

Atlas currently supports immutable v1 presets and custom snapshots pinned to exact template revision (ADR phase-4-custom-template-library). Store::save_custom_template serializes the saved presentation; instantiate_custom_template reads exact pin, creates fresh element/hotspot IDs, wraps instance Author, rebinds a limited set of source-bound components. No automatic upgrades. Query definitions are pinned not rows; default literals and overrides use deterministic typed saved parameter contract. Four possible reuse modes:

| Mode | Meaning | Atlas decision |
|---|---|---|
| Template snapshot | Copy presentation, prompt/default blocks and bindings into new instance | Keep existing pinned semantics, improve insert preview and human-friendly slot descriptions. |
| Canonical property/relationship | Display authoritative typed fact at current view | Reuse existing domain service, with authorized fields/provenance; do not copy current value into prose. |
| Pinned excerpt | Authored quotation or selected source version | First use source/claim and explicit citation link; optional snapshot content must be distinguished from a live reference and reauthorized. |
| Live shared section / transclusion | Resolve separately authored block from another entry at render time | Defer general support; useful for repeated articles but cross-page policy, cycles, fanout, revocation, history and export complexity. Start with ordinary link and narrow existing query/statement components. |

A reusable kingdom template can show a monarch field derived from current accepted office/title facts, an optional history prompt, and citation to a literal charter. Applying a template must not create a king, invent missing population/wealth, write relationships, duplicate factual values, or assume a referenced charter is the current article. Kanka-like kits should initially be UI panels over governed capabilities, not a user-defined EAV schema. If standard create and inline quick creation differ, either use the same typed command or explicitly label the minimal unfinished entry; no inconsistent defaults/identity dates. World Anvil currently documents 28 article templates and specific diplomacy/family/dictionary tools; some older features still use legacy edit interface [R47]. We should prioritize a concise, curated preset library over multiple type-specific editors with feature parity drift.

### 9.3 New concrete source finding: template rebinding is shallow

At 30b6710, crates/atlas-store/src/store/templates.rs:158–200 fresh_elements recursively traverses layout containers, regenerates IDs and rebinds source-equal ObjectRefs for FamilyTable, NetworkTable, ImpactTable, PublishedReferences, PublishedStatements and *top-level* supported QueryFilter variants (ParticipantEvents, EventRole, RelationshipPath, WithinPlace, Backlinks). QueryFilter::And/Or/Not children are not traversed by its query match. SavedQuery bindings, ObjectLink targets and image hotspot target identities are not rewritten by this routine. This is source-confirmed coverage, not a claim of data corruption; ADR promises supported self-bound components and explicit saved-query versions, so literal references may intentionally stay fixed. New Boolean filters and typed profile/descriptor values broaden possibilities but the current saved parameter types are not generic self slots. Do not silently reinterpret existing templates.

Reproduced the top-level versus nested matching boundary in a small JS model, not a Rust test. Example: an inline Query with And(Kind(HistoricalEvent), ParticipantEvents(source person)) remains bound to the original person when cloned by the source-shaped match. Merely recursively replacing all references equal to source is wrong: a literal biography reference to the original author/example or source may be intentional.

Recommendation: first expose a template preview showing supported automatic rebinds and preserved literal/pinned references, and regression test actual nested filters with explicit expected current behavior. If repeated-use evidence warrants a new reusable self binding, review a narrow explicit typed self/literal reference contract, not inference by UUID equality. Resolve allowed slot types before writing; preserve immutable pins and literal references; reject incompatible destination; leave old template semantics unchanged unless explicitly migrated. Proposed pseudocode only:

```text
template revision + destination identity + supported binding declarations
 -> validate destination kind + same world + selected view
 -> resolve explicitly declared self slot; retain literal targets
 -> fresh presentation IDs, private instance, validate
 -> preview; author explicitly saves
```

A later template update should be a deliberate version diff, not overwrite authored text. No bulk reapply or automatic inheritance until there is a tested user workflow. Related RQ-03/08, ENC-04, #5/#72. No new template feature approved here.

### 9.4 Draft, release, restore and historical context are different operations

[D R48] Notion exposes revision comparison, restores prior content while maintaining current database column permission rules, and plan-dependent history. [D R47] World Anvil distinguishes draft/read mode and autosave indicator. Atlas source entries.rs:8–64 saves document and published flag in one transaction, then updates public and backlink indices; read_layout_at:429–477 takes latest permitted version (current world revision for reader); entry-editor.ts:253–265 submits document and publication checkbox together; workspace_save_layout returns unit, not revision. This is explicit current behavior, not a verified defect. However, editing a previously published page and saving with publication still checked updates reader content; a draft-only edit while keeping the old public page live is not represented by a separate current release pointer in this path. Never imply current system has independent draft/release heads.

First, clarify existing semantics with distinct explicit actions: Save unpublished, Save and publish, and Withdraw; use dirty-state guard, revision checks, exact outcome, failed draft preserved. The simple unpublished save is revocation under current contract, so tell the author; never present it as retaining previous release. Fix UI without schema changes. If #5/#8 user workflow requires a pending revision while old release stays accessible, request specific lifecycle design review before changing storage (possible separate immutable draft and release selection, NOT two independent world stores, not an assumed accepted feature). Define two time axes, publication state, author edit history, and live bindings separately. A restored historical layout must not resurrect revoked grants or old reader release. Review current policy before deciding if past released static text renders current authorized facts or pins snapshot date/revision; preserve existing current-publication selector until supported. Old reader exports cannot be recalled, clearly disclose that limitation.

Proposed state model for research: save/edit draft appends revision but not release; explicit release validates draft and current authorized policy, then advances release; revoke clears release; restore old authored content creates NEW draft, no automatic republish. Optimistic expected revision, no delete/rewind canonical history. Separate transport response with matching context/draft generation and committed revision so a late save result cannot mark another editor clean. Exact regression expected: stale release rejected, complete state unchanged; revoke/restore remains unavailable; separate author undo and persisted corrections; reader navigation rechecks release.

### 9.5 Stable anchor navigation before arbitrary transclusion

[D R49] Obsidian links to headings and explicit block identifiers and embeds note/heading/block content; block-link syntax is tool-specific, not standard Markdown. [D R42] Notion synced blocks need original access and distinguish copies. Atlas already has ElementId and entry-target references, plus current non-recursive entry_link_target_visible in entries.rs:130–151: a link must be authorized without recursively compiling destination. Current public reader navigates by page ID rather than anchor. First support jumps to authorized stable ElementId within an article, not an arbitrary display string or label. A deep link is an entry ID plus existing element ID and context routed through actual authorized reader; absent/secret target generic unavailable. Use permitted current title; never send hidden anchor ID or caption in a reader DTO. On stale/revoked search hit reauthorize and avoid loading cached private content. Stable IDs remain on move/rename, fresh on copy. Ordinary cyclical wiki links are valid; only recursive rendering needs cycle protection.

General embed alternative would have a separate ref with target entry/block, explicitly chosen pin/live policy, no caller-chosen audience override, host AND target authorization, path-specific cycle detection and shared depth/expanded-work/bytes budgets. A repeated approved source in two different branches is not itself a cycle; a global visited set would incorrectly drop the second instance, while only path detection allows exponential expansion unless globally bounded. Hidden reference captions/anchors must not appear in TOC, count, search, export or error. Failure in hidden content must not disclose its existence by budget/error; raw work may still need deadline. Existing MAX_ELEMENTS 512 and MAX_DEPTH 16 are local layout budgets, not proof that general embeds are sound. Search/index invalidation would require dependency tracking across host/source contexts, including revoke and source update; no implementation until explicit approved need and tests. Prefer regular link, published references/statements or narrow data binding. This is a defer decision, not new category of approved component.

### 9.6 Portability: distinct outputs for distinct purposes

[D R50] A 2016 Realm Works development announcement describes separate full re-importable realm export and compact external consumption format. This is historical roadmap evidence, not a claim current export/activation works. [D R51] Campfire documents an element library across projects and sharing/export options. Cross-project reuse is attractive, but Atlas world boundary is a deliberate invariant. Do not share IDs across worlds silently: export/import of approved content requires explicit identity mapping, target verification and provenance; no automatic copying canonical facts from template or linking worlds. Three deliverables: verified author archive (current Store foundation); scoped reader artifact through same authorized DTO (pending #10/#8); future reviewed interchange for selected entries/templates, no generic importer now. Source-linked article references must resolve within actual selected export scope; filenames and assets must not leak secrets. No external proprietary dependency or code reuse. Local-first recovery precedes a public presentation feature.

### 9.7 Executed follow-on study

[workspace-lifecycle-prototype/check.js](workspace-lifecycle-prototype/check.js), [actual output](workspace-lifecycle-prototype/results.json), [limits](workspace-lifecycle-prototype/README.md). Executed exact function in V8. 23/23 assertions passed: 10 source-shaped/reusable typed-slot template assertions, 5 optimistic draft/release/restore assertions, 8 bounded embedding assertions. Includes nested filter limitation, literal-versus-self intent, incompatible type rejects, exact pin/fresh ID, current draft not release, stale rejection, revoke remains revoked after draft restore, repeated shared source, cycle/global budget checks, private source/host omission and revocation on recompile. The simplified embed model has author/public only, absent real ancestor/revision/branch/index/cache logic. It counts a hidden target reference against its work budget, so availability noninterference is unproven; do not claim it solves security. These are candidate contract examples, not validated Atlas APIs. Together with section 8, 48 isolated assertions have passed in two experiments. No native or external editing timings, no performance/scale claim from this second model, no UI screenshots appropriate.

### 9.8 Decision refinement / next evidence

Rank near-term: terminology/context/draft guards (no schema), template/picker preview + accurate supported/literal labels, authorized directory/anchors, article editor and typed reference implementation under existing design; later review independent draft release and explicit reusable slots only if workflow requires them. Narrow view/infobox first. General synchronized blocks, cross-world library, arbitrary property kits, bulk inheritance and plugin/formula runtime defer/reject. Native acceptance: real Rust regression of shallow/literal cases; actual published article edit/revoke/restore paths; late asynchronous save/focus; same entry quick vs standard creation context; template copy retains private audience/pins; scoped search/backlink/anchor revocation; current public layout behavior verified. Existing tickets #4/#5/#8/#10/#72, no duplicates, no AE/SP/WT closures. Still need actual author-task timings and owner review. Research #76 stays open. No production code changes.

### Further primary references (accessed 2026-11-22)

- [R40] Kanka, Jay, *Version 3.9 – New terminology & new text editor*, 2026-03-10. https://blog.kanka.io/2026/03/10/version-3-9-a-new-language/
- [R41] Obsidian Help, *Properties* and *Aliases*. https://obsidian.md/help/properties ; https://obsidian.md/help/aliases
- [R42] Notion, *Database templates*, *Synced blocks*. https://www.notion.com/help/database-templates ; https://www.notion.com/help/synced-blocks
- [R43] LegendKeeper Help, Templates. https://www.legendkeeper.com/help/
- [R44] Kanka, *Property Kits*. https://docs.kanka.io/en/latest/entries/property-kits.html (older Attribute Templates page points to renamed feature).
- [R45] World Anvil, *Guide to Custom Article Templates*. https://www.worldanvil.com/learn/article-guides/custom-templates
- [R46] Campfire, *Items Module Tutorial*, *Timeline Module Tutorial*. https://www.campfirewriting.com/learn/items-tutorial ; https://www.campfirewriting.com/learn/timeline-tutorial
- [R47] World Anvil, *How to Use Article Templates*, *Feature Guide to Articles*, *Anatomy of an Article*. https://www.worldanvil.com/learn/article-guides/article-templates ; https://www.worldanvil.com/learn/article-guides/articles ; https://www.worldanvil.com/learn/article-guides/anatomy-article
- [R48] Notion, *Delete & restore content*. https://www.notion.com/help/duplicate-delete-and-restore-content ; https://www.notion.com/en-gb/help/duplicate-delete-and-restore-content (detailed history/permission caveat).
- [R49] Obsidian Help, *Internal links* and *Embed files*. https://obsidian.md/help/links ; https://obsidian.md/help/embeds
- [R50] Lone Wolf Development Forums, *What's New in Realm Works*, circa 2016, historical export announcement. https://forums.wolflair.com/threads/whats-new-in-realm-works.56988/
- [R51] Campfire, *Help Center*, *Frequently Asked Questions*, *Using the Element Library* (last updated 2026-05-26). https://www.campfirewriting.com/learn/help-center ; https://www.campfirewriting.com/faq ; https://www.campfirewriting.com/learn/library-tutorial

## 10. Discovery architecture, stable navigation, and empirical article search

**Investigated:** 2026-11-22. Still pinned main 5777f0f and schema-44 candidate 30b6710. Review branch only; full candidate tree 267 entries, README/evidence conventions, ticket inventory and prior sections rechecked. Additional current primary help, accessibility/SQLite/Unicode specifications, six Kanka source files at previous exact commit, and an executed local SQLite study. No external authenticated editor trials or Atlas production changes.

### 10.1 Search is a family of tasks, not a single box

Notion workspace search exposes recent destinations and keyboard navigation [R52/R53]. Kanka documents typed mentions with exact-name syntax, separate captions, subpage and anchor references, and saved sidebar destinations including filtered entry lists [R54/R55]. Logseq's documentation separates page/block references, filtered backlinks and queries [R56]. These support a more focused design than a universal result list mixing commands, text, facts and proposals.

At Kanka a291b50 (R13, R57), CommandSearchService.php:15–73 uses SQL name/alias lookup (eight results) plus administration-page matches for name mode, and a distinct Meilisearch snippet path for full text (including maintenance status). MentionService.php:35–167 has separate entry/post/attribute/new results, availability/type filtering, alias join and exact-name handling. RoutingService.php:26–94 routes bookmarks to entry/subpage or filtered index/dashboard/random destination. These are observations of source structure; not a full permission audit, no code copied. The source-visible Kanka implementation and vendor user documentation may describe different deployed versions. Atlas already has typed historical Store queries, indexes and reader projection; do not introduce Meilisearch as another service merely because Kanka uses it.

Proposed modes on existing services, not new engines:
- Go to entry: name/alias/exact ID, compatible kinds, compact context-labelled results. Name collisions remain separate IDs. Recent or bookmarked eligible destinations as explicit local history (author mode first). No silent automatic creation or merging.
- Find in world: text and structured predicates, filters by existing kind/classification/source and fictional/recorded date. Author query results distinguish claims, entries, passages and existing facts.
- Find in article: search authoritative compiled document for the active scope, jump to exact ElementId. No unapproved indexing of author annotations in reader.
- Insert reference: picker restricted to accepted reference kinds, stable target plus caption, show current label only if allowed. Preserve draft and focus. New entry action requires explicit type and date/context validation.
- Run action: allowlisted contextual UI commands; not an unrestricted script engine, not automatic writes and not #70 transform composition.

Recommendation: a shared search affordance with explicitly chosen modes and existing compact typed adapters; do not create a new universal AST just for the palette. First use simple buttons/tabs and native labels; defer custom palette if existing search+picker meets measured workflow. Use existing QueryControl and request cancellation; hard result caps, deterministic ranking, stable disambiguation. Remembered destinations contain only stable IDs/context in author local preferences, not private full payload. Reader recents/bookmark labels reauthorize or disappear. No persisted reader histories containing unapproved names.

### 10.2 A real corpus experiment: article-level diversity

At Atlas entries.rs:319–417, search_public_pages loads candidates from current released static text, caption and dynamic dependency indexes, ordered by entry_id; compiles the page in scope, then fills one global limit of matching block hits (default UI 50). find_text_blocks:486–523 walks in layout order. author-workspace.ts:575–606 renders one passage and generic Read published page button per block; hit.element is present but click calls open(hit.entry.id), not an anchor. This is current source, not a newly observed UI bug. A densely matching first article can exhaust the budget; the remaining articles can be absent from result set. Deduplication after the first 50 cannot recover missing candidates. No native cursor for public article continuation; the prior author cursor contract is a useful precedent, but not the same historical/revocable current reader context (ADR phase-3-query-pages).

[Article search experiment](article-search-prototype/check.py) executed on local Python, SQLite 3.46.1, three synthetic block corpora. First 75 rows belong to one article and match river; subsequent rows each belong to another article, one in ten matches. Already-authorized data for this test. Compare ORDER BY article,id LIMIT 50 versus GROUP BY article, minimum matching ID representative, keyset continuation article > last. Verified first + second pages equal exact Python unique article oracle without duplicates. Nine warm in-memory query executions per measurement; no application compile or UI included.

| Blocks | Matching articles | Distinct first 50 blocks | Distinct first 50 grouped | Median blocks ms | Median grouped ms |
|---:|---:|---:|---:|---:|---:|
| 1,000 | 93 | 1 | 50 | 0.055 | 0.062 |
| 10,000 | 993 | 1 | 50 | 0.277 | 0.322 |
| 50,000 | 4,993 | 1 | 50 | 1.574 | 1.669 |

Second page counts 43,50,50. Actual output [results.json](article-search-prototype/results.json). Deliberately skewed corpus, not representative world completion, no benchmark claim for shipped Atlas. The modest SQL grouping overhead is encouraging but real candidate compilation, dynamic statements, caption release, short-term scan, FTS index rebuild, memory and cancellation remain to measure.

Recommended reader results: count **distinct authorized articles** and sample a few actually visible passage matches per article with stable anchors and an explicit more-matches affordance, not a count across hidden blocks. In first slice, candidate stable ID order (or deterministic authorized local title/exact group later) and bounded page; title/kind only when an authorized envelope exists. Compile/recheck before counting, not raw SQL candidate groups. New page contract return article ref, permitted identity envelope, matching authorized element IDs/snippets, bounded completeness status, opaque continuation (if complete and safe); no unrestricted author query. Reauthorize on click and context change; current public revision changes invalidate continuation instead of replaying an obsolete release. Do not pre-limit raw candidate IDs if it can starve visible results. Existing 512 visible-page budget and cancellation need appropriate regression; no automatic cap increases. For full-article find, keep current block result API as separate task. ENC-02/09/11 and RQ-02/06, #4/#5/#8/#10.

### 10.3 Relevance ranking can disclose hidden corpus changes

SQLite FTS5 documentation [R58] describes BM25 using global corpus statistics (document count, phrase frequencies and average length), plus configurable column weights. A SQL WHERE clause excludes returned rows but does not recalculate full-text corpus statistics just over the authorized subset. Tested with SQLite 3.46.1 default FTS5 unicode61, two visible documents, query river OR charter, title weight two; inserted 500 author-only synthetic rows containing river into mixed index, visible data unchanged.

| Condition | Result order |
|---|---|
| Mixed corpus before | charter, river |
| Mixed corpus after hidden insertion | river, charter |
| Isolated public corpus before and after | charter, river |

Thus merely filtering rows after ranking can produce an observable ordering change when inaccessible documents change. It is not a proven current Atlas leak: entries.rs orders candidates by entry_id, and existing public_index_text explicitly excludes restricted blocks and destination-dependent captions; caption index is an internal candidate index, not public corpus. New ranking must not use BM25 over mixed world/scope/revision/date/caption corpora and assume a row WHERE clause is sufficient. Even globally indexed released pages may be inaccessible on a given date or in another world. No one-index-per-character/day explosion recommended. First use deterministic rank buckets computed only from permitted per-result fields (exact title, exact permitted alias, prefix, text), stable ID tie; use internal FTS as candidate narrowing, never expose raw corpus scores. Any scoped corpus ranking must demonstrate equal result ordering/count/snippets under hidden/cross-world/date-inapplicable insertions; raw deadline variation is a separate challenge. No need to implement new sophisticated ranker until relevant article navigation works. INV-15, QRY-03, #4/#8.

### 10.4 Constructed languages and identity ambiguity

SQLite's trigram index does not match fewer than three Unicode characters as a MATCH term; current Atlas explicitly falls back for short/NUL public search [R58, phase-3-authorized-caption-search]. In actual experiment query Io returned no FTS rows, literal lower-case fallback returned Io and Iona. Composed Élodie and decomposed E + combining accent + lodie matched separately in the tested trigram; NFC+casefold candidate comparison retrieved both with different UTF-8 lengths 7/8. Atlas discovery.rs:321–375 deliberately uses byte-stable exact non-ASCII case and ASCII case-insensitive boundaries; this remains a documented matching boundary, not accidental loss of a claimed Unicode feature. For fictional scripts, transliteration, diacritics, same names and one/two-character labels: normalize candidate lookup only with reviewed Unicode semantics; preserve original stored name and stable ref; never normalize historical source byte spans without exact offset mapping. Unicode UAX29 [R59] establishes grapheme/word boundary principles; a grapheme is not necessarily one codepoint/byte. Start with literal exact plus existing aliases and disambiguating context; allow user-created historical spellings; no automatic identity merge, no language inference. Future enhanced matching must test equivalence, casefold expansion, combining marks, CJK/RTL, apostrophes and emoji, malicious pasted markup, IME and stale source generation. Do not use generic Unicode normalization to promote a canonical relationship. Query matching and link-review spans have distinct contracts.

### 10.5 Actual editor implementation lessons from Kanka

[R57] Inspected pinned entityLoader.js (92 lines) and entityLoader.test.js (135 lines), both under resources/js/editors/tiptap/extensions/mentions; also CommandSearchService.php, MentionService.php, searchAvailability.js and Bookmarks/RoutingService.php. Loader collects unique unknown IDs; tracks pending/unavailable, hydrates the *current* ProseMirror document, retains custom labels/configuration, avoids adding display metadata updates to undo history, checks destruction, leaves a failed request retryable. Test source includes duplicate pasted mentions inside table/prose, preserving bold text and custom label/config after hydration, no extra undo, edits during pending request and deletion during pending load. Inspected test source, did not execute upstream dependency suite. No conclusion on all authorization behavior or cache privacy. The code license remains Commons Clause; take design principles only.

Atlas application already guards late public/search responses (author-workspace.ts:545–619, public-reader.ts:47–78) and Canvas local undo snapshots. New inline reference editing should store target identity and authored caption, not cache mutable entity payload in canonical runs. Separate ephemeral decoration/hydration from authored changes. Batch permitted refs per actual context when justified, stop on detached editor or context change, resolve existing mentions by stable run ID against latest draft, never recreate deleted nodes or undo user typing. Context key includes world/day/branch/revision/scope plus draft generation; negative cache scoped to exact authorized snapshot, invalidate on relevant revision/revocation. Do not use a global permanent unavailable set across historical contexts. Prefer resolve current draft links through one bounded typed service; existing current author picker may suffice. Save response must report committed revision and match original dirty generation. Test out-of-order reply, removal, moved run, duplicate same target, rename with intentional caption, revocation and uncleared cache; old private display data never serialized or reused in reader. RQ-03/07/09, UX-02/07/12, ENC-03/10.

### 10.6 Navigation should preserve the user's work and context

Existing public-reader.ts:47–78 replaces DOM with loading status, stores a 64-ID history, clears it on unavailable target, ignores late responses; keep reauthorization and no cached private content. Existing author-workspace.ts has independent query/snapshot generations; do not discard those. Kanka filtered bookmarks and Notion history/keyboard shortcuts suggest a small navigation session contract [R52–R55]. One workspace selection and contextual peek first, not arbitrary split docking: history entry = route key (stable world/entry/optional element), validated context key, studio and local scroll/focus locator; no content snapshot. Back/forward re-resolve and restore logical focus; missing anchor generic unavailable or offer authorized top-level page, failed target must not wipe unrelated history. On scope transition clear/reauthorize labels, stale views and in-flight results, not simply keep prior rendered DOM while loading. On dirty state, Save/Discard/Cancel before switching; keep context-bound draft rather than save under new selection. Do not hold unauthorized page content for rollback. Current reader is current main/latest; no unsupported branch route fabricated. Bookmarks can initially be existing saved query links, not another persisted collection schema.

Use WAI-ARIA APG combobox/treegrid/tabs [R60–R62] as interaction guidance: distinguish focus from selection; Escape returns to input; Enter accepts an allowed identity; arrow navigation does not accept a match or trigger a write; only opt into complex treegrid when plain list/table no longer meets task; manually activate potentially expensive tabs rather than trigger a query on each focus move. Validate native keyboard operation and screen reader, not just ARIA labels. Avoid implementing global shortcuts while text IME composition is active; preserve editor focus and caret on cancel. Actual standards conformance not established by this study.

### 10.7 Implementation targets and stop rules

1. Existing-scope, no-schema work: complete picker coverage/context-safe create, stable navigation/dirty guard, authorized result DTO title/anchor where scope supports; improve labels and one compact author list. Reuse existing query controls. #4/#5/#8.
2. Article-level public search: after source compilation and scope tests, grouped result cap and stable bounded continuation, verified on actual mixed published corpus (short terms, repeats, duplicate names, private nested text, revoked/hidden caption, dynamic references, another world/date), compare 1k/10k/50k true article/block mixtures; include online/offline local native, query time, compilation time, first useful result, total bytes, peak memory and keyboard task. Do not promise timing from synthetic SQLite.
3. Only then consider rank buckets/quick palette/hydration, preserve original alias and span semantics. Relevance ordering must be independent of unavailable corpus; do not add vector DB, external indexed search service or AI ranking now. User-facing article links are not derived facts.
4. Native evaluation six workflows: query exact duplicate identity, open match anchor/back, write/refer while async metadata arrives, switch time/branch while editor dirty, revoke a result and reopen, resume saved filter. Use validated actual fixture; log action counts, elapsed, errors and context switches, no invented competitive timings. Keep prior prototype boundaries. No screenshots for a purely numerical experiment; capture authentic UI only when executed. All recommendations refer to existing phase gates; #76 stays open for owner and external interactive comparison.

### Additional references (accessed 2026-11-22)
- [R52] Notion, Search in your workspace. https://www.notion.com/help/search
- [R53] Notion, Keyboard shortcuts. https://www.notion.com/help/keyboard-shortcuts
- [R54] Kanka, Mentions. https://docs.kanka.io/en/latest/features/mentions.html
- [R55] Kanka, Bookmarks. https://docs.kanka.io/en/latest/advanced/bookmarks.html
- [R56] Logseq, documentation index (page/block refs, linked refs, embeds, query builder; only index reviewed). https://docs.logseq.com/
- [R57] Kanka exact source a291b50ee4d5a2ced469c4cca3747ccc125174a4, reviewed app/Services/Search/CommandSearchService.php, app/Services/Search/MentionService.php, app/Services/Bookmarks/RoutingService.php, resources/js/components/search/searchAvailability.js, resources/js/editors/tiptap/extensions/mentions/entityLoader.js and entityLoader.test.js. https://github.com/owlchester/kanka/tree/a291b50ee4d5a2ced469c4cca3747ccc125174a4
- [R58] SQLite, FTS5 extension, tokenizers and BM25. https://www.sqlite.org/fts5.html
- [R59] Unicode, UAX #29 version 18.0.0 revision 49, 2026-09-01. https://www.unicode.org/reports/tr29/tr29-49.html
- [R60] W3C WAI-ARIA APG, Combobox pattern. https://www.w3.org/WAI/ARIA/apg/patterns/combobox/
- [R61] W3C WAI-ARIA APG, Treegrid pattern. https://www.w3.org/WAI/ARIA/apg/patterns/treegrid/
- [R62] W3C WAI-ARIA APG, Tabs pattern. https://www.w3.org/WAI/ARIA/apg/patterns/tabs/
