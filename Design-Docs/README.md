# ConWoeld Tools — Design Documentation

**Working product name:** Living History Atlas  
**Status:** Reconciled architecture specification, not implemented software  
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

## Current repository state

This repository began empty. These are **design documents only**; no application, migrations, UI, GPU integration, or runtime behavior should be inferred to exist. Earlier v0.1–v0.4 files were source material, not independent competing specifications. Their requirements were rewritten, de-duplicated and traced in [12](12-Decisions-and-Reconciliation.md).

**First demonstrable milestone:** author two characters, establish a friendship, create a betrayal event with actor/claim/source, apply an explicit relationship effect, and switch the historic date and viewer perspective. Confirm semantic graph, Canvas component, public Display, and search agree about what is visible and why.
