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
