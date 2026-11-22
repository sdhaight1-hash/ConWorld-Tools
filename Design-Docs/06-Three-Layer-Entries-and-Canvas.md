# 06 — Three-Layer Entry Model, Composable Canvas and Publication

**Status:** Normative entry and presentation contract.

## 1. Three representations, one canonical entry

Every eligible world object has an **entry identity**; it can acquire three complementary representations:

1. **Node Studio (semantic representation):** typed objects, edges, draft nodes, explicit links, inferred suggestions, graph layouts and relationship exploration. This is authoring of world *meaning*.
2. **Canvas Studio (compositional representation):** a separate, versioned layout document that arranges written content, image annotations, responsive layout primitives and reusable query-bound components. This is authoring of *presentation*.
3. **Display Studio (publication representation):** compiled, read-only output for an audience/branch/date/revision/perspective. This is *authorized viewing*.

An object may exist before it has a Canvas document; an entry without a published page remains discoverable to permitted authors. All three layers refer to stable world ObjectIds; never copy a person into a page-specific JSON shadow database.

## 2. Semantic node types and connection strengths

Common nodes: Person/Character, FamilyHouse, Household, Organization, Faction, Place, Asset, Concept, Event, Conflict, Relationship, Membership, Claim, Source, Population and optional GeneticProfile. Relationship instances, not only endpoints, can be node targets for annotations and provenance.

Separate:
- **Canonical relationships** backed by validated typed facts.
- **Explicit content references** created from rich text, cards, images and citations.
- **Discovered candidates** from search, alias or AI, which are proposals, not facts.

Node Studio can auto-expand already-authorized canonical neighbors when budgets permit; it may suggest ghost nodes for unknown mentions. "Promote" is an explicit typed action with review, deduplication and an undoable audit record. No silent creation of kinship, membership or guilt from a name match.

Graph coordinates, pins, grouping, collapsed clusters and colors are view metadata, not historical properties of entities.

## 3. Canvas design: layout primitives containing components

Layout primitives control flow/placement; components display/author information. Both have stable ElementIds and may have children through defined slots, but they are *not* semantic-world nodes.

**Primitive registry:** CanvasRoot (viewport, theme/breakpoints), Stack (vertical/horizontal), Flex (wrapping distribution), Grid (tracks/spans), Panel (padding/border/overflow), SplitPane (resizable regions), Overlay (anchored layers), Spacer, Divider. Use responsive flow for normal entries. FreeformBoard with absolute placement/zoom is a separate later primitive, not the baseline for every page.

**Component registry:** Heading, RichText, ObjectLink, DataField, Portrait, Image, Gallery, ImageHotspot, TagList, Citation, GenealogyTree, RelationshipTimeline, MembershipHistory, EventActors, ConflictNetwork/Map, GraphView, MapView, Timeline, SearchResults, QueryTable, QueryRepeater, Chart, Backlinks, ConditionalRegion and nested component slots.

Component contracts declare kind and schema version; validated props, slots, value/data schema, bindings, capability requirements, empty/loading/error states, HTML fallback and optional accelerated renderer. Allow custom components later only through an explicit capability and security review; untrusted arbitrary JavaScript is not default.

~~~rust
struct LayoutDocument {
    id: LayoutId, entry: ObjectId, template: TemplatePin,
    author_revision: AuthorRevision, root: LayoutElement
}
struct LayoutElement {
    id: ElementId, visibility: PublishPolicy,
    kind: LayoutKind
}
enum LayoutKind {
    Primitive { primitive: PrimitiveKind, props: LayoutProps,
                children: Vec<LayoutElement> },
    Component { component: ComponentKind, props: ComponentProps,
                binding: Option<QueryBinding>, slots: Vec<SlotContent> }
}
struct QueryBinding {
    saved_query: Option<QueryId>, inline_spec: Option<QuerySpec>,
    parameters: Vec<TypedParameter>
}
~~~

Pseudo-types only. Decide serializable schema details and editor commands in an ADR; protect older layout versions with migrations.

## 4. Templates and components

Templates specify default primitive trees, named slots, allowed component categories, metadata, reusable themes, optional conditional sections and responsive variants. Initial templates: Character Profile, Noble House/Family, Organization, Event, Conflict, Location, Genealogy Report, Historical Source and blank entry. A template is version-pinned; updates should offer migration diff instead of silently altering existing pages.

Components bind to **object IDs or parameterized typed queries**. Example: a two-column character layout has Portrait on left, biography plus nested genealogy and guild-membership timeline on right. QueryTable("members of selected guild in 1890") displays current world data at the selected context. Moving that table between Grid and Panel must not change its data binding.

Canvas supports drag/drop, reparent, reorder, resize, copy/paste, binding inspector, constraint diagnostics, responsive preview, local undo/redo, keyboard alternatives, accessible content editing and autosave. Repeated blocks/slots use immutable keys to preserve identity across layout edits.

## 5. Rich hyperlinks, backlinks and images

- A typed explicit link stores source entry/block and text span or image hotspot; target WorldId/ObjectId; human label as presentation only; link kind/visibility/history. Rename never breaks it.
- Images can have crop/annotation/hotspot shapes with multiple labeled targets and per-hotspot visibility. Hotspots cannot expose secret link destinations in public image metadata.
- Names, nicknames, titles, transliterations and aliases map to candidate IDs with temporal scope and locale. Mention indexing shows snippets and ambiguity; a mention does not equal membership or parentage.
- Backlinks are context-aware reverse *references* plus permitted semantic links; provide provenance "linked directly", "mentioned in text", "suggested", or "accepted fact".
- Draft/ghost node proposals can be surfaced in Node Studio or a Canvas discovery panel. User acceptance invokes a validated domain command.

## 6. Visibility and compiled publication

Visibility rules attach to entry, field, block, primitive subtree, component slot, edge, document source, link, image hotspot and query binding. Policies combine audience/role, author or public mode, selected branch, fictional date, recorded revision, perspective/knowledge, explicit publish status and inherited restrictions.

**Security invariant:** compile from an authorized server/native-side query projection. A DOM CSS "display:none" toggle is not a privacy mechanism. The result must exclude secret content from:
- rendered HTML and scene DTOs;
- titles, tooltips, nav paths, backlinks, previews, graph counts and FTS suggestions;
- serialized source, image metadata/hotspots, exported indexes and downloadable assets;
- nested saved queries and conditional components.

More restrictive child policy wins unless an explicit reviewed override is permitted. Preview and export use the **same** publication compiler, not different filters. Public and character modes never grant author capability by default. Deep links to hidden objects return an appropriate indistinguishable inaccessible response.

## 7. Compilation steps

1. Load requested entry and pinned layout version at author revision.
2. Construct validated ViewContext and authorized object/field set.
3. Resolve chosen template + layout migration schema.
4. Walk layout tree; prune unauthorized subtrees **before** resolving nested bindings or generating excerpts.
5. Evaluate allowed QueryBindings in the same/no-more-powerful context.
6. Resolve explicit links, image variants/hotspots, source labels and accessible fallbacks.
7. Render HTML view model or static export; publish only scoped search/backlink indexes.
8. Record source revision/dependency signature; invalidate on relevant revisions or policies.

Do not let a hidden field influence an aggregate count, conditional branch or generated visualization.

## 8. First usable Canvas subset

Implement CanvasRoot, Stack, Grid, Panel, Heading, RichText, ObjectLink, DataField, TagList, RelationshipTimeline, QueryTable and Citation; author/public preview; manual layout JSON editing only as diagnostic, not the delivered UI. Later add responsive freeform, genealogy trees, complex map/graph charts, component marketplace and animation.

## 9. Acceptance gates

- Create a character Node without a page; later attach template and bound widgets.
- Change a canonical guild membership; Canvas widget refreshes and Display agrees.
- Move an existing bound component from Stack to Grid without duplicating world data.
- Different aspect-ratio preview changes layout only, not facts.
- Hidden primitive subtree, its children, nested query, image hotspot and backlinks never appear in public projection.
- A rename preserves direct links; an ambiguous alias creates a review candidate.
- Page export/import retains stable IDs and template version pins.
- Disallowed or unknown component schema renders a safe diagnostic, not executable arbitrary content.

## 10. MapView and ConflictMap integration

[13](13-Spatial-and-Nested-Location-Model.md) defines shared MapSpec, LayerSpec and scoped MapScene contracts. Node creates/queries Places; Canvas binds map recipes, themes and versioned assets without copying world data; Display consumes scoped immutable scene with accessible list and stable authorized links. Exact and explicitly schematic modes first; candidate geometry distinct, promoted only by explicit semantic command in author context. Nested maps may use unrelated frames; raster hotspots are links not geometry. SVG serialization is typed/escaped with no scripts, external resources or raw XML. Apply policy before map composition, not CSS hiding.

## Accepted ontology evolution #72 — contract reconciliation

[Accepted Architectural Evolution #72](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72) ([design](Architectural-Evolutions/universal-worldbuilding-ontology-and-governed-semantic-extensibility.md), [ADR-007](ADRs/adr-007-governed-semantic-extensibility.md), [approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72#issuecomment-7074159822)). AE-01–09 production gates remain unpassed; original phase/feature gates unchanged.

Semantic profiles guide compatible existing Node controls, optionally suggest layouts, never create canon by selection. Canvas and Display use original authorized facts and supported bindings; profile is not template. No standalone schema editor approved.
