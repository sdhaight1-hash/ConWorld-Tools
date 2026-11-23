# Visual design and beautification audit — 2026-11-22 UTC

Companion to [UI/UX audit](UI-UX-Audit-2026-11-22.md). Baseline 30b6710, schema 44. Design recommendations only; no production implementation, asset import, new theme engine or runtime acceptance.

## Diagnosis

The archived captures inspected in the companion audit show uniform teal surfaces, repeated outlines around nested sections, nearly identical treatments for primary/secondary actions, large full-width controls, small dense inspector labels, prominent raw identifiers, and little visual differentiation of record types. Graph workspace is visually sparse for small worlds but lacks type encoding and label density control for larger ones. Current CSS (apps/desktop/src/style.css:1–47, 75–127, 132–166) uses a small palette, system font, Georgia hero title, many independently styled nested containers, and largely equal-weight buttons. Canvas2D graph drawing (main.ts:264–333) uses one ordinary node color, one selected color and straight edges; GPU graph geometry has separate constants in graph-benchmark.ts. Public reader (public-reader.ts:81–145) mostly emits paragraphs, buttons and panels without a distinct editorial presentation hierarchy. Current graph coverage and historical context limitations remain as described in UX-01/04/10, not solvable with CSS.

## Visual direction

Contemporary archival atlas: a precision desktop knowledge workbench with understated cartographic character, rather than medieval ornament or generic SaaS dashboard. Preserve the existing teal/gold identity; deepen neutral blue-green surfaces and reserve warm brass for selection and primary action. Let the fictional world provide imagery and personality; app chrome must be readable and visually quiet. No ornamental map background behind text, excessive gradients, glow, glass blur, textures or animation. Avoid replacing compact tools with cards for every item.

### Proposed color tokens (candidate values, sRGB)

| Token | Value | Use |
|---|---|---|
| shell | #101B20 | App chrome and graph ground |
| surface | #18272D | Sidebars, pane backgrounds |
| raised | #22353C | Controls, selected grouping |
| text | #E9E7DF | Main typography |
| muted | #A9BBC0 | Metadata and helper text |
| accent | #D8B77A | Active workspace, primary action |
| control-boundary | #6D858C | Necessary control outline, not every decorative divider |
| success | #95B9AD | Confirmed save or completed status |
| warning | #F0DFAA | Attention, pending review |
| danger | #E99A91 | Errors, destructive action |
| accent-ink | #18272D | Text on gold primary button |

Measured with WCAG relative luminance formula in JavaScript: text/surface 12.41:1, muted/surface 7.72:1, accent/surface 8.03:1, text/raised 10.33:1, boundary/raised 3.28:1, boundary/surface 3.94:1, boundary/shell 4.49:1, accent-ink/accent 8.03:1, success/surface 7.18:1, warning/surface 11.59:1, danger/surface 6.95:1. Previous control border/input fill was 1.43:1. These selected pairs are not complete accessibility verification. Check actual rendered focus, hover, selected, disabled, forced-color and semantic node states before adoption. Do not use color alone for identity type, action state, certainty, or publication audience.

### Typography, surfaces and controls

Use local system-ui for authoring, Georgia or local serif fallback for reader title and intentionally editorial headings. No network font dependency. Proposed desktop scale: 13px minimum routine labels/metadata, 14px body/control text, 16px section title, 20–24px entity heading, 28–36px reader title; support browser zoom and inherited sizing. Reader prose 16–18px, 1.55–1.7 line-height, 60–75ch measure. Long labels wrap or show deliberate ellipsis plus accessible full name; raw UUID/technical metadata in optional details, never hidden when needed for disambiguation. Spacing on 4/8/12/16/24px rhythm; 4–6px control radius, 8px major pane radius. Compact list rows around 32–40px, control targets at least 24px and ordinarily 32–36px; test actual sizes and zoom, not nominal CSS alone.

Reduce card-in-card borders; use surface grouping, headings and dividers; 1 primary action per task area, outlined/default secondary, text/icon tertiary, explicit destructive label. Align checks with label, give input/select/textarea consistent typography and padding, reserve tabular figures for dates/revisions/metrics. Strong visible focus independent of hover. Pending/error/success and readonly states retain meaning through text/icons and do not alter access policy. Prefer a small consistent set of locally bundled SVG icons or native DOM symbols; no external icon font or large library dependency by default. Tooltips are supplementary, accessible names required for icon-only actions.

### Semantic visual language

Use a stable icon/shape and restrained color by existing canonical kind family: person/lineage (portrait/person), organization/faction/office (institution), place (pin/building), event/conflict (calendar/crossed links), concept/definition (tag/diamond), asset (box), source/claim (document/quoted document), relationship/fact (edge/link). These are UI grouping labels only, not new ontology types; profile remains separate from kind. Do not imply a person is a founder, a claim is true, or a location is known using decoration. Natural language kind labels and safe stable ID disambiguation remain accessible.

Canonical graph edges solid with directional markers when applicable; suggested links dashed and explicitly labeled candidate; reported/alleged relations visibly distinct from confirmed; selected neighborhood highlighted while non-neighbors recede without losing accessible list. Use shape/text in addition to hue, edge labels on focus, consistent selection in Canvas2D and GPU paths, zoom-aware labels and collision/culling tests. Preserve bounded work and renderer fallback; no new inferred edges or hidden-neighbor counts. Map styling to follow #68 authorized scene contracts: restrained layers, clear scale/north only when calibrated, separate schematic and exact geometry, clear unplaced list. Do not implement a second graph/map renderer to achieve the visual theme.

## Surface treatment

- Node: quiet full-width graph/list/timeline area, compact left entity explorer, right contextual facts/evidence; visible selected entity title and short kind label, grouped relationship rows, directional cues and provenance on demand. Current 17rem inspector is not the full editor.
- Search: conventional compact result rows with icon, title, kind, secondary source/date; one Open action, other actions contextual. Claim quote and citation grouped, no repeated large action bars. Selection and query state preserved. Disambiguation required.
- Canvas: outline, actual preview canvas, properties; distinguish author draft from publication. Selected component border and handles never mutate world meaning; full preview width, container-responsive layouts; privacy/validation states visible. Existing component IDs and bindings retained.
- Display: editorial reading surface, heading, readable prose, concise metadata, inline authorized links, sources and related entries. Offer quiet paper-like presentation as a future reader style, not as a global theme engine. Separate reader and author tools; never render private author content then conceal it by CSS.
- Genealogy/event/organization: clear timeline, grouped record tables, distinguish biological/legal/social edges and claimed versus accepted facts. Portraits and coats of arms are optional authored assets, not arbitrary placeholders or generated historical evidence.
- Empty/loading/error: purpose-specific message and useful next action; skeleton only if justified, no flashing/reflow; honor reduced motion, no GPU required for navigation. Technical renderer/IPC diagnostics in an explicit developer area.

## Implementation plan and validation

V-01: after coordination with #75, introduce small CSS token layer, control styles, accessible states, typography and consistent spacing. Reuse current components. Check selectors against current tests; preserve IDs and command behavior.
V-02: implement compact rows/selection and contextual actions as part of UX-03/05/06/07/12/14, not a second navigation architecture. Verify all object kinds.
V-03: dedicated Canvas/Display in existing phase #5, preserve save guards, publication and template pins, fit preview widths.
V-04: graph kind encoding and label hierarchy under #4/#6/#68/#72; reuse proven geometry/collision research without widening scope. Validate native GPU and fallback equivalence of selection and labels.
V-05: capture exact-build before/after on Alder Basin once its fixture version is verified. Cover world selection, duplicate search, a sourced claim, a connected family, institution membership, author Canvas and public Display, with actual 460x540, 1080x760 and 1920x1080 windows, 200% zoom, keyboard, forced colors, no GPU and offline. Inspect no overlapping/clipped text, no horizontal page overflow except intentional bounded graph, controls remain accessible, actual text and boundary contrast, no lost focus/dirty state/policy leakage. Compare task interactions against original audit; no new phase exit or gate closure.

Do not change production code on the frozen consolidation candidate solely for visual work. Keep all palette/typography values proposals until rendered and tested. No fresh capture was made for this design-only audit; existing real and mock-native evidence retained with original labels. No external images, fonts or commercial assets added.

Track via #4/#5/#6/#10, #68 and #72. Beautification complements, not substitutes for, the behavioral UI/UX audit.
