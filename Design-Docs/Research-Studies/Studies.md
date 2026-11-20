# Research Studies

This directory holds **research studies produced after a focused investigation** for ConWorld Tools. Research studies inform architectural decisions, the original roadmap, and potentially new-feature proposals. It is **not** a directory for speculative research requests or issue placeholders.

**Governing instructions:** [Root README — AI Systems Instruction (Lola) and ticket-creation policy](../../README.md) · [Design documentation index](../README.md) · [Architecture decisions and reconciliation](../12-Decisions-and-Reconciliation.md).

## Research tickets versus completed studies

- **Proposal:** Create a GitHub issue titled `Research: Focused Research Topic` following the [root README research-ticket rules](../../README.md#5-research-tickets--proposals-for-studies-not-completed-studies). The issue MUST specify a focused question, why it matters, the decision it would enable, hypotheses, boundaries, proposed methods, affected design/phase links, and **real preliminary citations with clickable source URLs**. These are starting references, **not findings**.
- **Investigation:** Conduct the actual literature/code/repository review, testing, benchmarks, or comparative research. Record what was examined, the methodology, dates, observed evidence, conflicting results, and limitations. Do not invent paper findings or claim to have run experiments that were not performed.
- **Study artifact:** Write the resulting cited Markdown study in this directory. Use a descriptive filename such as `large-graph-rendering-comparison.md`. Mark drafts as **Draft**, and distinguish them from **Reviewed** studies.
- **Review and close:** Cross-link the study to its originating Research issue and the issue back to the study. Evaluate its recommendations; update a relevant ADR, existing Roadmap Phase, or separately approved `New Feature:` proposal as warranted. Close the Research issue only after its research and review acceptance gates are satisfied.

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
9. **Impact map** — affected requirements, ADRs, Roadmap Phase issues, test cases, APIs, schemas, and any justified `New Feature:` proposal.
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

_No completed studies are listed in this directory at the time this index was established. Add verified study links here as work is performed._
