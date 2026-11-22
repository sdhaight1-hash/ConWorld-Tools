# World Bible-first authoring — direction and feasibility review

**Status:** Owner-directed UX exploration, 2026-11-22. Non-normative until comparative prototype review and reconciliation. No new schema, ontology scope, phase or acceptance-gate changes. Baseline main 5777f0f, review 033dc63, consolidation 30b6710. #72 is still open; latest c68c2c9 persistence and scale passed, desktop failed on the fact resolver selector; test-only fix 1aa9b0b is pending verification. Do not merge overlapping unverified schema branches.

## Product direction

The World Bible is the primary product, not merely the published destination. Its author view is the complete permitted working directory, including identities without articles; its reader view is the authorized encyclopedia. An author opens an entry, reads it, edits its semantic properties or visual article, inspects its relationships, hierarchy, evidence and history, and returns without losing context. Genealogy, graph, mapping, chronology and discovery support this experience. Do not force users to construct nodes in a separate studio before writing an article.

Retain the normative [three representations](06-Three-Layer-Entries-and-Canvas.md): canonical semantic identity and facts, separate Canvas layout, authorized Display. Node authoring may be exposed inside the entry editor and graph rather than mandatory independent navigation. This supersedes the *exploratory UX recommendation* of four dedicated studios or mandatory Write/Design/Bind modes, not the underlying contract. The [earlier article concepts](Article-and-Entry-UX-Concepts-2026-11-22.md) remain a component and styling reference.

## One entry editor, multiple contextual tools

- World Bible home and directory: searchable types, subtypes and tags; author distinguishes identity-only, draft and released; reader sees authorized released pages only. Selecting an entry opens its shell, not a raw graph inspector.
- Edit: semantic authoring panels and editable bound components alongside Canvas text, imagery, layout and styling. Lightweight properties-only mode and quick creation from search/graph preserve identities without pages. Never create facts merely by adding a component.
- Connections: one contextual relationship editor supports any compatible typed endpoints, including people, groups, families, institutions, titles, offices, locations, assets, concepts, events and sources. Keyword searchable stable identity picker; type/predicate filtered by registry and actual endpoint kinds; directional sentence preview; dates, role, evidence and scope only as supported. Shared validated domain commands; distinct parentage, membership, placement, title holding and other specialized invariants. A relationship cable and form edit the same record, not independent copies. Derived kinship is not a duplicate editable fact. Remove component does not delete fact; withdraw/end/correct a semantic fact explicitly with revision validation.
- Hierarchy: nested and cross-nested views through typed relations, not folders or a generic parent field. Multiple views of one identity; distinguish biological/legal descent, house/household, organization membership vs constituent hierarchy or authority, spatial physical vs administrative containment, ownership, and political control. Do not infer transitivity across incompatible lenses; strict cycles validated with co-effective historical intervals, not universal cycle rejection. See [04](04-Genealogy-Relations-and-Genetics.md), [13](13-Spatial-and-Nested-Location-Model.md), #6/#68. UI grouping/reparenting is not a canonical mutation without explicit typed author action.
- Evidence/history/discovery: contextual tools use same current world, supported branch/day/revision, authority; preserve correction vs historical change and claim vs canon. Graph/workspace recipes remain #70/#78, not new engines.

## Component architecture

Entry type selects stable kind and compatible semantic profile; user-defined tags and governed classifications can coexist. Religion/culture are Concept classifications where supported, not a universal enum or automatically a membership. Article template selects initial components and layout. Theme controls font, color, spacing, decoration and imagery. Independent configuration; changing template/theme never changes identity or facts. Existing nine version-pinned templates and custom template system first.

Two author-facing component families: (1) semantic controls/projections e.g. identity, measurements, classification, relationship, genealogy, membership, title/office, location, history, evidence, (2) presentation e.g. rich text, heading, panel/grid/stack, image, map, tree, timeline, styling. A semantic control may open the native editor or explicitly mutate a supported fact; binding alone remains read-only and presentation removal non-destructive. No generic entity-component storage or arbitrary field/predicate runtime. Prepopulate bindings from compatible authorized data, not copied values. New components recommended, not silently inserted. Authored narrative never overwritten or silently time-traveled; proposed generated text requires review. #77 governs independent draft/release, not implemented by this UX. Reader projection cannot run author-only queries, expose unreleased data via counts or labels, or bypass supported historical axes.

## Audit-to-delivery map

| Existing finding | Revised resolution | Owner |
| --- | --- | --- |
| UX-01/02, ENC-05/06/12 | shared honest context/capability, draft retention, explicit release and revocation | #4/#5/#8/#10/#77; #79 first |
| ENC-01/02/09/11, UX-05 | World Bible shell, authorized directory/page envelope, grouped search and back/focus | #4/#5/#8 |
| UX-03/06/09 | shared compatible identity picker, focused quick creation and editing without full scene | #4/#5/#6/#72 |
| UX-04 | focused multi-kind graph with typed original fact IDs, bounded expansion and accessible list | #4/#5/#6/#68/#72 |
| ENC-03/07/10 | minimum safe rich-text/heading/inline cite/reference subset | #4/#5/#8 |
| ENC-04/08, UX-07/08 | same full-width Canvas tree/template; prefills compatible blocks, semantic edit actions, responsive and keyboard tests | #5/#8 |
| Cross-nesting | compatible hierarchy lenses derived from original typed domain services, not a parallel graph | #6/#68, #72 |
| Advanced workspace and transforms | optional contextual tools, independent readiness | #70/#78 |

No rewrite of store, migrations, geometry, genealogy, validation, search/FTS, graph renderer, Canvas IDs, queries or publication compiler. Reuse entry-editor.ts, network-studio.ts, canvas-tree.ts, canvas-templates.ts, public-reader.ts and author-workspace.ts through a common selection/context shell; do not move code speculatively. Keep original acceptance requirements. Changes beyond existing accepted contracts require separate reviewed proposal.

## Feasibility prototype

[Open the standalone prototype](Research-Studies/world-bible-prototype/index.html) locally in a browser; isolated synthetic fixture only; no Tauri, SQLite, external media, network, production writes or security certification. It compares a world directory + contextual Create/Edit/Connections/Hierarchy/Read experience against mandatory studio switching. Typed relation choices are sample fixtures (not registry parity). In-memory changes reset on reload. Author-only mock projections demonstrate the design intent, not real permissions. Current legacy release-selection behavior and #77 are not implemented here.

Validation tasks: locate one entry, add a compatible relationship using search and predicate, navigate through nested/overlapping structures, edit a bound article using the same identity, create identity without article, preview released context without private entries. Check navigation/draft retention and display vs semantic removal separately in production. Run the isolated browser test against this HTML; capture actual prototype with UTC filename and clearly marked prototype. Compare native baseline and proposed shell using observed steps/time/errors at 1080p and small window before full rewrite. Stop if a smaller contextual editor meets tasks.

## Delivery order

1. Correct and verify #80 and reconcile #75/#79/#80, exact head native/persistence and #72 gate review; do not conflate UX concept approval with ontology closure.
2. Review prototype before production layout changes. Implement smallest directory/article shell against authorized APIs with existing context guards and selection; keep current semantic editor access.
3. Integrate shared relationship picker/edit action into entry context; reuse graph services; distinguish semantic commands and presentation bindings. Validate no-article identity and private projection.
4. Improve structured text, template/presentation and safe bound components only on existing accepted scope; measure native UX. Add richer styling/collections/hierarchy only as demonstrated and reconciled.
