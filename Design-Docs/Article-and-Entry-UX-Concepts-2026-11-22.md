# Article and entry experience — design discussion, 2026-11-22

**Status:** Owner-requested record of UI/UX design ideas discussed with Lola. Non-normative concept direction, not a new accepted Architectural Evolution, implementation claim, or blanket feature approval. No changes to production code, schema, existing acceptance gates, or ontology integration. Review baseline `15448c4d63eae2249c58865fa9cd448fc613f978`; #75/#79/#80 remain independent implementation/review streams.

## 1. Product intent and authority

Build an encyclopedia that combines structured reference writing, flexible document composition, historical illustration, and interactive world exploration. A user should be able to create a two-paragraph note, illustrated character biography, archival dossier, interactive map article, or elaborate historical compendium without becoming a graphic designer.

An entry is the canonical identity; an article is its authored presentation. A world object can exist before any article. Preserve [06 — three-layer contract](06-Three-Layer-Entries-and-Canvas.md): Node = semantic meaning, Canvas = authored composition, Display = authorized reading. Writing/design/binding are modes over the same Canvas document, not additional data models. The world and historical context remain shared. Semantic profiles, Canvas templates, presentation themes and workspace configuration have different responsibilities. Never duplicate canonical facts to lay out a page.

Existing authorities: [encyclopedia audit](Encyclopedia-Audit-2026-11-22.md), [UI/UX audit](UI-UX-Audit-2026-11-22.md), [visual audit](UI-Visual-Design-Audit-2026-11-22.md), [comparative research](Research-Studies/comparative-worldbuilding-workspaces-and-encyclopedia-ux.md), [ADR-007](ADRs/adr-007-governed-semantic-extensibility.md), [ADR-014](ADRs/adr-014-independent-editorial-release.md). #72 ontology scope is unchanged. Independent saved draft/current release is governed by #77 and not implied by editor modes. Configurable author dashboards remain separately bounded by #78.

## 2. Article visual archetypes

Starting presets rather than article-kind restrictions:

| Archetype | Visual composition | Example use |
| --- | --- | --- |
| Encyclopedic | Lead, navigation, infobox, headings, citations, related entries | General reference |
| Narrative and immersive | Wide hero, atmospheric editorial typography, chapter-like sections, quotations, illustrations | Biography, mythology, lore |
| Archival dossier | Structured identification, evidence, dated records, correspondence, annotations | Institutions, intelligence, sources |
| Illustrated atlas | Maps, diagrams, nested locations, legends and geographical prose | Places, borders, exploration |
| Genealogical chronicle | Portraits, heraldry, lineage trees, marriages, succession and dated history | Houses, dynasties |
| Interactive compendium | Collections, cards, tables, charts, filters and summaries | Civilizations or organizations |

Authors may combine these styles. Existing nine built-in templates and version-pinned custom templates are the initial delivery mechanism; avoid a new ObjectKind for a presentation archetype. Multiple independently authored articles for one identity are not assumed by this concept and would require separate review.

## 3. Article anatomy

Illustrative Northbridge city example (synthetic data, not a running screenshot):

```text
[Breadcrumbs / optional collection]
[Wide cover image]
Northbridge                                  [Bookmark]
The Crown of the Northern Coast
[City] [Trading port] [Regional capital]

[Lead / introduction]                [Infobox panel]
[History]                            [Coat of arms]
[Culture and society]                [Founded / population]
[Quotation / marginal note]          [Government / region]
                                     [Map thumbnail]
[Historical timeline]
[Related entry cards: house / guild / region]
[Sources, backlinks, further reading]
```

Header, cover, subtitle, classification labels, lead, section outline, prose, infobox, media, timeline, references and related entries are independently composable. Title/identity envelope must come from the authorized page contract; tags and related data cannot reveal an inaccessible entry. Authored copy is not automatically rewritten when canonical data changes. Distinguish live date-aware values from static narrative and label the supported reader context explicitly.

## 4. Element library

The following is a proposed product inventory, not a claim that all components exist or are approved for production. Start with supported Stack/Grid/Panel, Text, ObjectLink, images/hotspots and existing query/table projections; add the smallest validated structured text and heading subset under Phase 4 before decorative expansion.

| Group | Candidate elements |
| --- | --- |
| Editorial | Title, subtitle, semantic headings, rich paragraph, lead paragraph, drop cap, quotation, callout, ordered/unordered/nested lists, footnotes, citations, generated contents, accordion, tabs, spoiler, marginalia |
| Media | Cover, portrait, crest, flags, insignia, signature, image, gallery/carousel, annotated illustration and hotspots, document image, audio/video where approved |
| Decoration | Divider, spacer, ornament, border, texture, icon |
| Semantic | Data field, infobox, tag list, entry link/card, collections, references and backlinks |
| Dynamic | Genealogy/pedigree tree, relationship table/timeline, event timeline, succession/organization chart, interactive map, network/conflict diagram, population chart, saved query table/repeater, synthetic genetics report where available |
| Layout | Root, responsive vertical/horizontal Stack, Grid, Panel, Flex, SplitPane, Overlay; possible later masonry and isolated FreeformBoard |

Component registry uses declared capabilities, typed props, versioned schema, supported slots, accessible alternatives and safe missing/loading/error/unsupported states. Spoiler or collapsed content is a presentation state, never an authorization barrier. No arbitrary HTML/CSS/JavaScript or unreviewed third-party executable components.

## 5. Responsive composition and reusable building blocks

Prefer a responsive-flow document with optional freeform containers over absolute positioning for every article. Think document editor + visual layout builder, not a mandatory graphic-design canvas. Text growth, narrow viewports and image changes should not require manual repositioning. Stack/Grid/Panel suffice for the first pass; overlays, split panes, masonry and isolated freeform boards are later capability decisions, not new requirements to finish Phase 4 immediately.

An infobox can be a reusable composition instead of a large purpose-built widget:

```text
Panel
  Portrait
  Title
  Divider
  Stack
    Bound birth date
    Bound residence
    Bound family
  Family tree (only when supported/authorized)
```

Containers have independently editable properties and nested children; stable ElementIds, bindings and author text survive reorder/reparent/resize. Provide visible drop targets, keyboard move/reparent, undo/redo and selection synchronization between outline and preview. Reject invalid cycles and incompatible slots. Template extraction copies presentation and its supported binding configuration, not world facts; instantiate new element IDs and explicitly rebind appropriate references. Version pin remains authoritative; never silently migrate existing articles.

## 6. Three editing modes, progressive disclosure

| Mode | Main experience |
| --- | --- |
| Write | Distraction-free article surface: direct prose, formatting toolbar, inline links, headings, insert menu/slash commands, citations and images. |
| Design | Element palette/tree, selection boundaries, drag handles, layout and style inspector, responsive preview. |
| Bind data | Compatible field/identity/query picker, labels of the bound source and date/revision, supported formatting/fallback, safe preview; no raw UUID required for ordinary work. |
| Read/preview | Actual authorized Display projection; not an editor mode with concealed author data. |

All editing modes use one working document and shared undo/redo history; switching mode preserves selection, scroll/focus, draft and context. Save/discard/cancel on destructive navigation; stale writes retain draft and report conflict. Keep existing published lifecycle behavior explicitly labeled until #77 delivers independent release selection; no implied autosave publication.

Basic authors choose a preset and fill in fields; intermediate authors add/reorder/resize elements; advanced authors nest containers, style elements and bind data. Do not force all controls on first-time users. Canonical changes triggered from an article must be explicit domain commands or reviewed proposals, never a side-effect of layout editing.

Conceptual workbench (not a runtime capture):

```text
[Current entry | World / supported branch / date / revision / perspective]
[Write | Design | Bind data]                 [Save | Preview | Release action]
[Element palette/tree] [Article canvas] [Properties / binding inspector]
[Status: dirty / saved / conflict / context limits]
```

Use collapsible side panels, keyboard shortcuts, logical focus order, visible focus, readable hit targets and clear empty states. Keep the main authoring surface large enough for 1080p and scaling. In Write mode hide nonessential panels; in Design/Bind mode expose only context-relevant controls.

## 7. Styling and design assets

Proposed deterministic presentation cascade: reader accessibility constraints > explicit element override > applicable container inheritance > article override > pinned article template defaults > world presentation theme > application fallback. Application chrome remains independently themed; world themes must not make navigation unreadable. Define inheritance per supported property; avoid arbitrary CSS selectors, external fonts/scripts or unbounded style overrides. Provide reset-to-inherited and indicate where a value comes from.

Style inspector candidates: typography (family, size, weight, line height, alignment), text/accent/background color, texture, spacing/padding/gap, border/radius/shadow, width/height limits, crop/aspect ratio, opacity and responsive behavior. Restrict values to validated tokens or bounded values. Presets can include archival parchment, contemporary reference and dark atlas; support high contrast and reduced motion. Existing visual audit's quiet teal/gold application chrome is a starting reference, not a hardcoded article palette.

World-level presentation asset library proposal: crests, flags, borders, dividers, textures, icons/map symbols, imagery and templates. Reuse existing media authority and asset IDs where possible; separate design assets from semantic heraldry/ownership facts. Keep licensed/local assets, alt text, preview and crop handling, scoped access, archive replay and missing-asset behavior. No new independent asset database as a precondition.

## 8. Entry organization and reader experience

Canonical placement (e.g. physical containment or membership) is distinct from encyclopedia navigation. A monarch could appear in Monarchs, Dynasties, Wars and Government collections via one stable Person identity. Collections could be manually curated references or derived from existing authorized saved queries, subject to scope and feature review. Do not infer canonical relations from collection membership. Support author discovery of identities with no article; reader sees only authorized entries, counts, labels and snippets. First deliver a coherent directory and page envelope under ENC-01/02 before configurable collections.

Reader candidates: clickable linked portraits, accessible image hotspots, hover/focus previews, expandable genealogy and graph views, interactive maps, sortable tables, collapsible sections and history/date controls. Expansion opens the relevant authorized specialist studio/view without granting the reader author access. Preserve identity, context and logical focus on navigation/back. Where a projection or alternate branch is unsupported, explicitly deny or explain; no hardcoded generic query exposure. Marginalia may distinguish author notes, translated quotations, competing sourced claims or annotations; private notes never sent to reader.

## 9. Implementation order and scope reconciliation

| Priority | Deliverable | Existing owner / dependency |
| --- | --- | --- |
| 0 | Explicit shared context, draft guard, stable reference picker and navigation | #4/#5/#8, #79, #72; integrated review first |
| 1 | First-class encyclopedia home, authorized page envelope, grouped search/anchors | #4/#5/#8; ENC-01/02 |
| 2 | Small structured rich text, headings, stable inline links/citations and accessible editing | #5/#4; ENC-03; review serialization/migration |
| 3 | Write/design mode on same document and reliable nested composition; existing presets and template instantiation | #5; ENC-04; existing tree/template/media service |
| 4 | Bounded style inspector, world presentation defaults and asset reuse | #5/#10; review additional capabilities under governance |
| 5 | Bindings mode with compatible picker and authorized previews; add specialist widgets only when underlying domain service supports them | #5/#6/#7/#8/#68; #72, query budgets |
| 6 | Independent draft/release and optional dashboards | #77/#78 own readiness; independent delivery gates |

This is a proposed sequencing guide, not a roadmap amendment; defer feature expansion until ontology #72 and context safety consolidate. Additional multi-article versions, unrestricted layout/asset formats, public dynamic queries, new dashboard scope, or custom renderer/plugin systems require explicit review and separate tickets if materially outside current contract.

## 10. Proposed acceptance scenarios

- Create a canonical Person without an article; attach a template, write a biography, add an inline stable link and cite a source. Rename the target; link still works. Do not create facts merely from prose.
- Switch Write/Design/Bind with unsaved edits, preserve exact text, element IDs, context and focus; cancel navigation and recover from a stale-write conflict.
- Compose an infobox from nested primitives; move a bound child; render narrow and wide widths; preserve data binding and source version. Keyboard-only equivalent and screen-reader heading hierarchy.
- Render known zero, explicitly unknown, missing and inaccessible values appropriately. A private binding/child/asset/hover target cannot leak through titles, counts, index snippets, export or rendered DTO. Unsupported branches and projections fail closed.
- Create a saved template variation, instantiate for another compatible identity, review rebind and upgrade diff. Original entry and template pins remain unchanged.
- Change world presentation defaults then override one article/element; reset inheritance; verify reader contrast and no loss on save/reopen/archive.
- Find one authorized entry from more than one collection; open at matched section, expand supported widget and return with context. Do not produce duplicate world identities or reveal private collection members.
- Use native Windows and Linux, measured narrow/1080p layouts, keyboard and 125% scaling; record real captures using [Development-Screenshots guide](../Development-Screenshots/Dev-Screenshots.md). Mock-native browser and conceptual diagrams must be labeled; they do not certify native behavior. No new runtime captures were made for this documentation discussion.

## 11. Open design decisions

Validate task effort for modal design vs a contextual inspector; how far direct rich-text editing and inline insert controls should go; minimal style token schema and theme pinning; manual collection scope; public component safety and supported historical axes; whether freeform has enough demonstrated author value. Prefer the smallest working solution and stop when existing components meet observed author tasks.

The discussion's main recommendation: an expressive, responsive document canvas over stable semantic identities, with approachable writing controls and optional advanced composition and binding. This is not an ontology expansion.
