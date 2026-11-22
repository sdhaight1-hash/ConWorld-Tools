# Research Studies

This directory holds **research studies produced after a focused investigation** for ConWorld Tools. Research studies inform architectural decisions, the original roadmap, and potentially architectural-evolution or new-feature proposals. It is **not** a directory for speculative research requests or issue placeholders.

**Governing instructions:** [Root README — AI Systems Instruction (Lola) and ticket-creation policy](../../README.md) · [Design documentation index](../README.md) · [Architecture decisions and reconciliation](../12-Decisions-and-Reconciliation.md).

## Research tickets versus completed studies

- **Proposal:** Create a GitHub issue titled `Research: Focused Research Topic` following the [root README research-ticket rules](../../README.md#5-research-tickets--proposals-for-studies-not-completed-studies). The issue MUST specify a focused question, why it matters, the decision it would enable, hypotheses, boundaries, proposed methods, affected design/phase links, and **real preliminary citations with clickable source URLs**. These are starting references, **not findings**.
- **Investigation:** Conduct the actual literature/code/repository review, testing, benchmarks, or comparative research. Record what was examined, the methodology, dates, observed evidence, conflicting results, and limitations. Do not invent paper findings or claim to have run experiments that were not performed.
- **Study artifact:** Write the resulting cited Markdown study in this directory. Use a descriptive filename such as `large-graph-rendering-comparison.md`. Mark drafts as **Draft**, and distinguish them from **Reviewed** studies.
- **Review and close:** Cross-link the study to its originating Research issue and the issue back to the study. Evaluate its recommendations; update a relevant ADR, existing Roadmap Phase, or a separately reviewed `Architectural Evolution:` or `New Feature:` proposal as warranted. Close the Research issue only after its research and review acceptance gates are satisfied.

An AI operating as **Lola** MUST follow the root README's think-first, simplicity-first, surgical-change, and testable-goals instructions. A proposed study does not authorize implementing its recommendations or changing accepted design contracts.

## Minimum research-study content

Each completed study SHOULD contain all relevant sections below, with **unknown, untested, and not-applicable items stated explicitly**:

1. **Title, status, date, author/research agent, original Research ticket**, and affected roadmap/feature/design links.
2. **Research question and decision context** — what the study set out to resolve and what is already established.
3. **Scope, hypotheses, alternatives, and success criteria** — what would distinguish the approaches.
4. **Methods and research log** — source selection, search terms, repository versions/commits, date accessed, experimental setup, and reproducible commands when experiments were performed.
5. **Evidence and findings** — factual results supported by citations and/or observed reproducible results; separate reported claims from independently validated observations.
6. **Comparison and trade-offs** — feasibility, accuracy, performance, maintenance, complexity, platform compatibility, failure modes, and security as applicable.
7. **Limitations and counter-evidence** — gaps, uncertainty, negative findings, version drift, and what was not tested.
8. **Recommendation and decision implications** — preferred path, rationale, rejected alternatives, confidence, and what must still be decided.
9. **Impact map** — affected requirements, ADRs, Roadmap Phase issues, test cases, APIs, schemas, and any justified `Architectural Evolution:` or `New Feature:` proposal.
10. **References** — stable clickable primary/authoritative sources, citations near substantive claims, dates where available, URL/DOI/repository commit, and retrieval date for mutable sources.

### Citation practice

Use visible reference identifiers such as `[R1]` beside the relevant assertion and a corresponding entry in **References**, for example:

`[R1] Organization/Author, *Document Title*, publication/version date (if known), https://example.org/source (accessed YYYY-MM-DD). Supports: the documented API contract.`

The example URL is a **format illustration, not a usable citation**. Before opening a Research issue or writing a completed study, replace it with sources actually verified. Prefer original specifications, primary research papers, original repository code and experimental outputs over unsourced summaries. Cite conflicts as well as agreement.

## Study filename and status conventions

- Use descriptive lowercase hyphenated names ending in `.md`; avoid generic `research1.md`.
- At the top, label the document `Draft`, `In Review`, `Reviewed`, or `Superseded`; record the related issue.
- Keep original source observations separate from design recommendations; use an ADR to adopt an architectural change.
- Do not create an empty study file merely because a `Research:` proposal was opened.
- Update this index with a link and short decision-oriented description when substantive studies are added.

## Study index

- [NestedLocations — Hierarchical, Spatial and Historical Place Modeling](nested-locations.md) — **Reviewed**, [research #65](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/65). Recommends typed temporal adjacency, optional geometry, bounded queries, scoped maps and cross-phase integration; includes reproducible isolated SQLite tests. Architecture direction accepted in [ADR-013](../ADRs/cross-phase-spatial-cartography.md); production work remains pending.
- [Procedural Vector Cartography from NestedLocations Data](procedural-vector-cartography.md) — **Reviewed — research complete**, [#66](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/66), companion to #65. Source comparison, executed isolated Chromium/geometry prototype, 1k/10k/50k SVG vs Canvas2D microbenchmarks, actual capture and proposed design gates. Direction reconciled in [ADR-013](../ADRs/cross-phase-spatial-cartography.md); follow-up locked-boundary/unknown and measured-label experiments complete; production checks remain open.

Feature handoff: [#68 accepted design](../Features/nested-locations-and-procedural-vector-cartography.md) links both studies and governs implementation; #66 research closure evidence is recorded in section 11; production acceptance remains pending.

- [Maltego-inspired transforms for fictional world exploration](maltego-inspired-transforms.md) — **Reviewed — research complete**, [#69](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/69). Compares Maltego, private Webb-Database architecture (no private data copied) and Atlas pinned source; independent SQLite synthetic feasibility test and worldbuilding workflow analysis. Context actions first within existing scope; recipe feature #70 subsequently accepted, with readiness/stop condition preserved. [Closure review](../ADRs/phase-3-contextual-exploration-review.md). No production implementation.

Subsequent explicit owner approval: [#70](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70) and [approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/70#issuecomment-7073776197) accept finite typed recipe composition/replay scope under the [accepted feature design](https://github.com/sdhaight1-hash/ConWorld-Tools/blob/main/Design-Docs/Features/composable-worldbuilding-transforms-and-exploration-workflows.md). Readiness Gate A requires verified baseline and demonstrated author-value gap versus existing contextual/saved queries; stop/defer if insufficient. This is separate feature scope, not a change to #69 findings, original phase exits or #68. Dedicated lore workbench remains deferred. No WT production gate passed.

- [Universal worldbuilding ontology, entity classification and domain coverage](universal-worldbuilding-ontology.md) — **Reviewed — research complete**, [#71](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/71). Formal ontology and five principal game/TTRPG case studies, two supplemental studies, candidate classification and integration matrix, synthetic competency and dense classification comparison. Existing core first; constrained semantic profiles are a possible separate proposal, pending owner review. No production implementation.
  - [Deeper product-value review](universal-worldbuilding-ontology.md#11-deeper-evaluation-marginal-value-to-the-actual-product): compares completed original scope, curated and author-defined profiles; 12-task value matrix, historical and sci-fi scenarios, independent semantic validation fixture and proposed scope/stop criteria. In Review, no new feature accepted.
  - Governance update: [Architectural Evolution ticket category](../Architectural-Evolutions/Evolutions.md) explicitly accepted by the owner; #71's shared ontology recommendations are candidates for a separate proposed evolution, with any genuinely new product capability still separately classified. Research remains In Review, not approval of a specific change.
  - Final disposition: [accepted Architectural Evolution #72](../Architectural-Evolutions/universal-worldbuilding-ontology-and-governed-semantic-extensibility.md), [ADR-007](../ADRs/adr-007-governed-semantic-extensibility.md), [owner approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72#issuecomment-7074159822); see [study §13](universal-worldbuilding-ontology.md#13-final-research-review-and-architectural-disposition). All seven research gates complete, AE production gates open.

- [Comparative worldbuilding workspaces and encyclopedia UX](comparative-worldbuilding-workspaces-and-encyclopedia-ux.md) — **In Review**, [#76](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/76). Eight-product primary-source comparison, public reader navigation evidence, pinned Kanka source/license review and 12-item adoption register mapped to prior audits and existing roadmap. Three deeper implementation reviews, 48 isolated model assertions and executed SQLite article-search/authority-ranking/Unicode fixtures (1k/10k/50k blocks) examine projection, lifecycle, reference hydration and retrieval; interactive authoring comparison and owner disposition outstanding.
