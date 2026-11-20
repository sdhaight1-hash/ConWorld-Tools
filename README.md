# ConWorld Tools

**ConWorld Tools** is a local-first fictional worldbuilding, genealogy, relationship-mapping, and historical knowledge system. Its architecture and original implementation roadmap live in **[Design-Docs](Design-Docs/README.md)**.

This repository uses GitHub Issues to manage **roadmap phases, proposed research, and genuinely new features**. The rules below are instructions for human contributors and **AI architects/coding agents**. They apply whenever an AI considers creating, revising, or closing an issue.

## AI ticket creation policy

### 1. Inspect the current project before creating anything

An AI **MUST**:

1. Read this README, the [design documentation index](Design-Docs/README.md), [accepted decisions](Design-Docs/12-Decisions-and-Reconciliation.md), [requirements and invariants](Design-Docs/01-Requirements-and-Invariants.md), [delivery roadmap](Design-Docs/10-Delivery-Roadmap.md), and the relevant specialist specifications.
2. Inspect the **actual current** repository tree, the existing [GitHub issues](https://github.com/sdhaight1-hash/ConWorld-Tools/issues) (open **and** closed), and relevant pull requests and studies. Do not assume prior conversations or earlier repository names reflect current state.
3. Check the two existing documentation areas: [Features](Design-Docs/Features/) for post-roadmap feature designs and [Research-Studies](Design-Docs/Research-Studies/) for completed research studies.
4. Search for duplicate or overlapping work by *concept and aliases*, not only by exact issue title. Prefer updating/linking an existing issue over duplicating it.
5. Classify the proposed work as **exactly one** of the three issue types defined below. If the work is already covered by a Roadmap Phase, put it under that phase rather than calling it a new feature.
6. Check scope, prerequisites, known decisions, existing evidence, and conflicts. Link authoritative documentation using current `ConWorld-Tools` paths.
7. Write a concrete, useful issue **before** creating it: scope, deliverable, dependencies, verification and links. Do not create empty placeholders or pretend a proposal represents finished work.

**Do not create a GitHub issue solely because an AI has an interesting idea.** Explain how it serves the product, what decision/work it unlocks, and how completion can be verified. An AI may propose research or a feature, but proposing must not be mistaken for automatic authorization to alter accepted architecture or expand implementation scope.

### 2. Allowed ticket types and exact title conventions

| Type | Required title format | When to use it |
| --- | --- | --- |
| **Roadmap Phase** | `Roadmap Phase N — Descriptive Title` | A phase of the established, dependency-ordered implementation plan. **N** is the roadmap phase number, not the GitHub issue number. |
| **Research** | `Research: Focused Research Topic` | **Propose** a focused investigation required to answer an unresolved technical, design, UX, scientific, or product question. The expected future output is a cited **research study**. |
| **New Feature** | `New Feature: Descriptive Feature Name` | Propose work genuinely **outside** the original roadmap and its existing feature requirements. Do not disguise normal roadmap implementation, bug fixes, or architectural tasks as new features. |

Keep the prefix, colon, and `Roadmap Phase N` numbering consistent. Give every issue a specific, searchable title. Never renumber or duplicate existing roadmap phases without an explicitly approved roadmap change. Labels and milestones are optional metadata; do **not** assume labels or milestones exist, and never rely on them instead of the exact title prefix.

### 3. Common minimum requirements for every issue

Every issue MUST contain:

- **Problem / purpose:** what is missing, unknown, or required, and why it matters.
- **Scope and exclusions:** what the ticket covers and deliberately does not cover.
- **Current context:** known state vs assumptions, with links to existing relevant issues/PRs and canonical design docs.
- **Dependencies:** what must be resolved or implemented first; link the dependency issue directly when available.
- **Planned deliverables:** observable artifacts, not vague instructions like "improve the system."
- **Acceptance / completion gates:** objective criteria for declaring that *this particular kind of ticket* is complete.
- **Risks and unresolved questions:** including required architecture decisions or potential conflicts with invariants.
- **References:** direct clickable links to relevant design sections, existing work, and verifiable external sources where appropriate.
- **Status integrity:** the body must distinguish *proposal*, *work performed*, *verified result*, and *remaining work*.

Use Markdown checkboxes for actionable deliverables. Prefer stable URLs to a specific file or document over generic repository-home links. Never fabricate citations, source results, testing outcomes, existing files, or completion status.

### 4. Roadmap Phase tickets

**Title:** `Roadmap Phase N — Descriptive Title`

The established phases **0 through 9** are defined in the [master roadmap](Design-Docs/10-Delivery-Roadmap.md) and tracked in [GitHub Issues](https://github.com/sdhaight1-hash/ConWorld-Tools/issues).

Before creating a Roadmap Phase ticket, the AI MUST confirm that the phase does not already have an issue. If it exists, update the existing issue or create clearly linked, smaller implementation tasks only when needed. Do not create a second `Roadmap Phase 4`, for example.

**Required issue sections:**

- **Objective and phase boundary:** why the phase exists and the capability it delivers.
- **Roadmap source and direct design-document links:** identify relevant requirement/acceptance IDs.
- **Prerequisites:** linked earlier phases and architectural decisions.
- **Design / ADR deliverables:** precise interfaces, data structures, invariants, migrations, and decisions needing validation.
- **Implementation deliverables:** narrow working vertical slices; no mock-only completion.
- **Acceptance tests and exit gates:** repeatable tests and observed UI/runtime behavior where relevant.
- **Non-goals and risks:** what is deferred and why.
- **Completion checklist:** reviewed design; implemented behavior; tests run; evidence; documentation and issue updated.

**Closure rule:** a roadmap ticket closes only when its required artifacts are implemented and verified, with commits/PRs and actual test evidence. A design proposal, passing isolated SQL experiment, or UI mockup does not prove a complete application feature. See [Acceptance and Verification](Design-Docs/09-Acceptance-and-Verification.md) and [AI Engineering Instructions](Design-Docs/11-AI-Engineering-Instructions.md).

### 5. Research tickets — proposals for studies, not completed studies

**Title:** `Research: Focused Research Topic`

A research ticket MUST define a *question worth investigating*. It is **not itself the research study**, and creation of the ticket MUST NOT imply that the investigation, experiments, paper review, or conclusions have already occurred.

Research may be proposed by the project owner or by a designing AI when it encounters meaningful uncertainty or competing approaches. Focus the scope narrowly enough to reach a decision; avoid tickets such as `Research: Everything About AI`.

**Required issue sections:**

1. **Research question / decision to unlock:** a precise question and why answering it matters.
2. **Context and motivation:** affected feature, phase, design doc and existing knowledge; explicitly distinguish established facts from hypotheses.
3. **Key research questions or hypotheses:** measurable comparisons, competing designs or unknowns.
4. **Scope / non-scope:** what to investigate and what is outside this study.
5. **Proposed methodology:** literature review, repository/source-code inspection, benchmarks, prototypes, usability evaluation, or other techniques; specify the evidence required.
6. **Preliminary citations, links and references:** real, verifiable starting references, cited in the body (e.g. `[R1]`) and collected in a **References** section. Give author/organization, title, publication or update date *when known*, URL/DOI or repository path, and what each reference supports. Link internal design specifications and related issues as well. These are **starting sources**, not completed findings.
7. **Research deliverable:** a future Markdown **research study** in [`/Design-Docs/Research-Studies/`](Design-Docs/Research-Studies/), with suggested descriptive filename, methods, dated sources, results, limitations, alternatives, recommendation and impact on the architecture.
8. **Completion criteria:** the actual study exists and is linked; claims are cited; conflicting evidence and limitations documented; recommendations map to affected decisions/roadmap issues.
9. **Follow-up decision:** describe how results will be reviewed and incorporated into ADRs, existing phase tasks, or a separately approved New Feature proposal.

**Citation rule:** every Research issue MUST include real clickable source links **and** explicit citations/reference entries in its proposal. Verify a small number of genuinely relevant sources **before** opening the issue, without pretending to have performed the full study. If trustworthy starting references cannot yet be verified, do not invent them; gather verifiable sources first or flag the draft for review instead of publishing an unsupported research ticket. Public papers, authoritative specifications, original project repositories, and measured primary evidence are preferable to unverified summaries.

**Study workflow:** creating the issue = **Proposed Research**; conducting research is subsequent work; writing the cited study under `/Design-Docs/Research-Studies/` = the primary deliverable; reviewing the study and recording its architectural consequences is the closure gate. The study must cite sources near substantive claims, include retrieval dates where relevant, and distinguish observations, inferences, speculation and recommendations. **No study should be marked complete merely because the research ticket was created.**

### 6. New Feature tickets — strictly outside the original roadmap

**Title:** `New Feature: Descriptive Feature Name`

This type is for a capability not already specified by the [product charter](Design-Docs/00-Product-Charter.md), [requirements](Design-Docs/01-Requirements-and-Invariants.md), specialist design documents, or [Roadmap Phases](Design-Docs/10-Delivery-Roadmap.md). New Feature issues can originate from the owner, a research study or an AI proposal; they remain **proposals** until the scope change is accepted.

**Required issue sections:**

- **Problem and user value:** user workflows improved, clear motivation, expected outcome.
- **Out-of-roadmap justification:** specific comparison with current scope; explain why an existing Roadmap Phase cannot own it.
- **Proposed behavior and UX:** observable interactions and an end-to-end user scenario.
- **Architecture impact:** Rust/domain types, historical/time model, Node/Canvas/Display representations, query/search/backlink behavior, permissions/publication, persistence/migrations, wgpu needs and integration boundaries as relevant.
- **Dependencies, trade-offs and alternatives:** including "do nothing" or "extend an existing feature."
- **Research and references:** link any relevant `Research:` ticket and completed study, plus canonical docs and supporting external references.
- **Scope and non-goals:** explicit v1 feature boundary and what remains deferred.
- **Acceptance criteria and tests:** verifiable behavior, errors, permission handling, performance and migration gates relevant to the feature.
- **Design deliverable:** upon acceptance, write or update a feature-specific specification under [`/Design-Docs/Features/`](Design-Docs/Features/) and cross-link it from the issue.
- **Approval/status:** state **Proposed / Under Review / Accepted / Deferred / Rejected**, with an actual recorded decision; do not assume acceptance from issue creation.

**Scope rule:** do not rewrite the original roadmap just to accommodate an idea. First compare it against existing requirements and issues. If it is covered, link or update the existing phase. If evidence is insufficient, open a `Research:` proposal instead. If genuinely new, use `New Feature:` and seek explicit scope approval before implementation or changing accepted requirements.

### 7. Ticket handling lifecycle

1. **Discover and classify.** Inspect actual repository, documentation, previous decisions and all relevant issues; reject duplicate tickets.
2. **Draft with source links.** Ensure the correct title pattern and appropriate type-specific fields. Link current paths under `ConWorld-Tools`.
3. **Create in GitHub.** Include all required sections at creation time; do not rely on future comments to fill an otherwise empty ticket.
4. **Cross-link.** Link prerequisites and parent roadmap phases, related research tickets/studies, feature specifications, and relevant PRs. Update existing indexes only when they are genuine indexes for that type.
5. **Track work honestly.** Use checkboxes, dated observations, commits and verified outputs. Distinguish `not run` from `passed`, and proposal from an approved/finished study.
6. **Review before closing.** Verify the exact completion gates, link resulting research/feature/design documents, reconcile accepted decisions and update dependent tickets.
7. **Avoid unsolicited scope expansion.** Research recommendations do not automatically authorize implementation; an interesting prototype is not a new roadmap phase.

### 8. Example title classifications

| Proposed work | Correct classification |
| --- | --- |
| Build the typed historical query system already planned for phase 3 | Existing **Roadmap Phase 3** (do not duplicate) |
| Evaluate large-graph layout algorithms and benchmark wgpu vs Canvas2D | `Research: Large-Graph Rendering and Layout Strategies` |
| Compare synthetic DNA recombination models before implementation | `Research: Synthetic Chromosome Recombination Models` |
| Add collaborative cloud editing if not in the accepted roadmap | `New Feature: Multi-User Collaborative World Editing` |
| Improve family-tree traversal defined in phase 5 | Existing **Roadmap Phase 5** (not a New Feature) |
| Investigate a contested paper's applicability to affiliation inference | `Research: Evidence-Aware Affiliation Inference` |

### 9. Canonical documentation and authority

- [Design documentation index](Design-Docs/README.md) — source of truth for current architecture.
- [Delivery roadmap](Design-Docs/10-Delivery-Roadmap.md) — established phase order and prerequisites.
- [Requirements and invariants](Design-Docs/01-Requirements-and-Invariants.md) — behavioral contracts.
- [Acceptance and verification](Design-Docs/09-Acceptance-and-Verification.md) — testable completion gates.
- [AI engineering instructions](Design-Docs/11-AI-Engineering-Instructions.md) — coding and validation practices.
- [Features](Design-Docs/Features/) — accepted/design-in-progress **post-roadmap feature specifications**.
- [Research Studies](Design-Docs/Research-Studies/) — **completed or explicitly draft** research documents, never confused with Research ticket proposals.

When documents conflict, follow the accepted decision hierarchy in [the design index](Design-Docs/README.md) and record an ADR/change proposal. New tickets must point to canonical documents, not become competing specifications. Never silently alter those contracts.

**Repository:** [sdhaight1-hash/ConWorld-Tools](https://github.com/sdhaight1-hash/ConWorld-Tools).
