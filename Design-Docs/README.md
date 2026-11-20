# ConWorld Tools — Design Documentation

**Working product name:** Living History Atlas  
**Status:** Architecture specification with verified slices tracked below.

**Baseline:** 2026-11-20 · Revision 1  
**Location:** `/Design-Docs` is the source of truth for architecture and requirements.

## Read in this order

| Document | Purpose |
|---|---|
| [00 — Product Charter](00-Product-Charter.md) | Product mission, use cases, scope and assumptions |
| [01 — Requirements & Invariants](01-Requirements-and-Invariants.md) | Normative, testable behavior |
| [02 — System Architecture](02-System-Architecture.md) | Rust, HTML/TypeScript, Tauri, wgpu and module boundaries |
| [03 — Domain & Time Model](03-Domain-and-Temporal-Model.md) | IDs, typed facts, provenance, revisions, calendars and branches |
| [04 — Genealogy & Relations](04-Genealogy-Relations-and-Genetics.md) | Kinship, tags, histories, social bonds, organizations and synthetic DNA |
| [05 — Events & Conflicts](05-Events-Actors-and-Conflicts.md) | Agency, event affiliations, causation, factional conflicts and impact |
| [06 — Three-Layer Entries](06-Three-Layer-Entries-and-Canvas.md) | Node, composable Canvas and published Display |
| [07 — Query & Discovery](07-Search-Query-and-Discovery.md) | Search in all studios, typed AST, backlinks and proposals |
| [08 — Storage & API Contracts](08-Storage-and-API-Contracts.md) | SQLite schema, commands, queries, import/export and security |
| [09 — Acceptance & Verification](09-Acceptance-and-Verification.md) | Scenarios, fixtures, tests and measurable gates |
| [10 — Delivery Roadmap](10-Delivery-Roadmap.md) | Ordered implementation with cross-layer vertical slices |
| [11 — AI Engineering Instructions](11-AI-Engineering-Instructions.md) | Coding-agent workflow, non-negotiables, handoff and evidence |
| [12 — Decisions & Reconciliation](12-Decisions-and-Reconciliation.md) | Resolved conflicts, pending ADRs, prior-source map and change policy |

## Document authority

1. **Accepted decisions** in [12](12-Decisions-and-Reconciliation.md), then **invariants** in [01](01-Requirements-and-Invariants.md), then the domain/API contracts govern implementation.
2. Specialist documents elaborate the same model; they must not introduce a separate source of truth.
3. An unresolved item is labeled **OPEN** with a validation plan, not silently decided by an AI agent.
4. Historical **in-world events** are not the same as the application's **revision/audit events**.
5. Normative terms **MUST**, **SHOULD**, **MAY** follow their ordinary requirements meaning.

## Active roadmap issues

- [Roadmap Phase 0 — Architecture Baseline & Platform Feasibility](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/1)
- [Roadmap Phase 1 — World Persistence, Temporal Model & Provenance](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/2)
- [Roadmap Phase 2 — Relationships, Tags & Event-Driven History](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/3)
- [Roadmap Phase 3 — Universal Search, Queries, Backlinks & Discovery](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/4)
- [Roadmap Phase 4 — Three-Layer Entries & Composable Canvas](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/5)
- [Roadmap Phase 5 — Genealogy, Dynasties, Households & Organizations](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/6)
- [Roadmap Phase 6 — Event Studio, Actor Agency & Conflict Atlas](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/7)
- [Roadmap Phase 7 — Evidence, Perspectives, Branches & Privacy](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/8)
- [Roadmap Phase 8 — Synthetic DNA & Population History](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/9)
- [Roadmap Phase 9 — Release Hardening, Export & Optional NPC Integration](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/10)

All ten issues start **open** and link to their relevant design contracts and acceptance gates. The planned phase number is independent of the GitHub issue number.

## Current repository state

See the [Phase Zero verification log](Phase-0-Verification.md) for the desktop and renderer evidence, and the [Phase One foundation](Phase-Foundation.md) for the persistence slice. Earlier v0.1–v0.4 files were source material. Their requirements were consolidated and traced in [12](12-Decisions-and-Reconciliation.md).

**First demonstrable milestone:** author two characters, establish a friendship, create a betrayal event with actor/claim/source, apply an explicit relationship effect, and switch the historic date and viewer perspective. Confirm semantic graph, Canvas component, public Display, and search agree about what is visible and why.
