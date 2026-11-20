# 00 — Product Charter

**Status:** Normative product intent · 2026-11-20

## Mission

ConWoeld Tools (working application name **Living History Atlas**) is a local-first system for designing, exploring, explaining and optionally simulating interconnected fictional worlds. It begins with fictional genealogy but is fundamentally a **temporal, evidence-aware world knowledge graph**. Its differentiator is not the number of diagram types: it is the ability to answer *what was connected, when, from whose viewpoint, what happened, and why it changed*.

A world author should be able to enter a single person, an incomplete event or an organization; gradually connect them; discover possible related entries; compose an editorial page; and publish an appropriate, searchable view without copying the same information into three incompatible stores.

## Product pillars

1. **Historical relationships:** people, households, organizations, families, places, assets, beliefs, events, actions and conflicts are all referenceable objects. Any significant relationship or affiliation can be tagged, dated, sourced, revised and investigated.
2. **Genealogical depth:** legal, social and biological kinship, dynasties, household history, inheritance and synthetic DNA-style ancestry are distinguished rather than collapsed into family-tree edges.
3. **Causality and agency:** actors perform actions within events, groups have event-specific affiliations, conflicts have dynamic positions, and explicit effects explain relationship and institutional change.
4. **Three faces of one entry:** Node Studio exposes semantic connections; Canvas Studio composes responsive pages from layout primitives and data-bound components; Display Studio presents a scoped, read-only publication.
5. **Universal discovery and queries:** search and typed graph/temporal queries work in every studio; backlinks, aliases, text mentions and image hotspots support progressive worldbuilding without automatically treating every mention as a fact.
6. **Perspective and privacy:** canon, source testimony, rumor, uncertainty and character knowledge are different. Hidden author knowledge must not leak through queries, public backlinks or published components.
7. **Local ownership:** offline desktop authoring, portable project data, deterministic generated histories, no required AI or cloud service.

## Primary users and workflows

**Worldbuilder:** starts with named nodes, authors family/organization/event facts, follows suggestions, builds interconnected chronologies and historical explanations.

**Writer/researcher:** compares conflicting records, reads an event biography, discovers cross-generational causes, queries the world as a particular character or at a particular date.

**Game/NPC designer:** exports typed canonical snapshots and knowledge-scoped queries for NPC memory or external simulations. NPCs do not need to be active agents for the authoring application to work.

**Reader/audience:** navigates carefully published pages, images, relationships and search without gaining access to withheld material.

## Exemplary workflow: an estrangement

Create Alice and Beatrice in 1898; establish a friendship and a pair of asymmetric trust states; compose a character entry. In 1908, record discovery of a disclosed letter, a Beatrice action, sources/claims, and explicit changes to tags and trust. In 1914, record a political alliance despite distrust. Slide the world date back: the Node graph, relationship biography, Canvas widgets, Display view and permitted search results must agree. Character and public perspectives may legitimately differ. No automatic AI prose may invent missing reasons.

## Scope

**Included:** stable identities, typed temporal links, event participation, actions, organizations, memberships, factions, conflicts, sources/claims, genealogical trees, structured search, evidence-aware timelines, bounded graph rendering, composable entry authoring and secure publication.

**Later, not prerequisites for v1:** synthetic chromosome recombination and reference populations, population-scale simulations, procedural historical generation, GIS-heavy conflict territory, multiplayer collaboration, hosted publishing, external NPC write-back, arbitrary third-party component execution and fully featured freeform art boards. Preserve extensible boundaries now without pretending these features are built.

## Launch assumptions

- Windows desktop first; single editor per open world; offline-first.
- Rust 2024 native authoritative engine; HTML/CSS/TypeScript UI in Tauri 2; optional React view layer; wgpu rendering behind a tested canvas boundary, with accessible fallback.
- SQLite per project or equivalent locally managed database; no server or dedicated graph database required for initial scale.
- Multiple isolated worlds; configurable fictional calendar; partial dates and alternate-history branches.
- English initial UI; localization, shared editing and mobile readers are future decisions.

## Success criteria

The first end-to-end proof must show create/edit/save/reopen; historical and author-revision queries; at least one event-driven relationship change with cited cause; perspective-sensitive nonleaking publication; the three entry surfaces; and a query-bound component. Follow with genealogical and organizational integrated fixtures, export/import verification, and measured 1k/10k/50k focused-graph tests.

**Non-goal:** to make relationships, personalities, ethnic identity or narrative causality entirely numeric. User-authored qualitative content and uncertain claims are first-class.
