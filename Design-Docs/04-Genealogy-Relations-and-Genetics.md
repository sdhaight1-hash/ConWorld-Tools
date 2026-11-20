# 04 — Genealogy, Relationships, Organizations and Synthetic Genetics

**Status:** Normative model; genetic simulation is a later implementation phase.

## 1. Relationships have their own lives

Each consequential relationship is an addressable, versioned object: subject, object, typed relationship definition, branch, valid interval, source/provenance, status, tags, optional directional attributes and event-linked effects. Person A may simultaneously be Person B's cousin, business partner, political opponent and legal creditor. Ending one relation neither deletes nor ends the others.

**Relationship type definition** describes eligible endpoint kinds, symmetric/asymmetric behavior, inverse label (parent-of / child-of), interval-overlap rules, allowed role qualifiers and domain validation. **Relationship tag definition** describes category, allowed attachment kinds, mutual exclusions if any, evidence requirements, author/public visibility and suggestion source. Tags have their own effective and author-revision history.

Examples include familial (sibling, adoptive family, estranged), social (friend, mentor, acquaintance), romantic (partner, ex-partner, married), emotional (trusts, envies, resents), economic (owes, employs, owns, finances), power (commands, charters, governs), organizational (member, infiltrator, officer), conflict (opposes, mediates) and secrecy (publicly denied, rumored). Labels do not imply symmetric emotional states.

## 2. Relationship evolution and biographies

The relationship inspector MUST expose:

- Dated intervals/tags, starting and ending state, breakpoints and linked events.
- Directional perspectives (Alice -> Beatrice vs Beatrice -> Alice), optionally typed emotional dimensions with qualitative notes. Never require scores.
- Changes connected to action/event IDs, with before/after versions, alleged causes vs established causes, citations, actor interpretations and uncertainty.
- A graph of contributing evidence/events and the ability to open adjacent conflicts, memberships or historical records.
- A dynamic biography assembled from verified, accessible history; gaps remain explicitly unresolved rather than filled with generated fiction.

In the demonstration fixture: friends in 1898, business pressure in 1903, an attributed betrayal in 1908 and pragmatic alliance in 1914. A 1902 snapshot must not show 1908 estrangement; a 1914 snapshot can simultaneously show political cooperation and low trust. A character may hold an incorrect explanation for the betrayal without altering canonical events.

## 3. Genealogical relationships

Model **biological parentage**, **legal parentage**, **adoption**, **step/foster relationship**, **guardianship**, **household membership**, **marriage/partnership**, **family/house identity**, **surname use**, **inheritance/title succession** independently.

A child may have biological parents A and B, an adoptive parent C, a guardian D, live in Household H, claim membership in Dynasty X and appear in a false public record naming E. This is not a contradiction in the schema: the record sources and semantics differ.

Do not assume exactly one mother/father field. Parentage cardinality for biological human worlds is constrained by species/ruleset, evidence may be uncertain, and fantasy reproduction can be provided through explicit alternate rules. Enforce no ancestry cycle on *accepted* parentage and configurable age/chronology rules. Derive sibling, cousin, ancestor, descendant, relationship degree, common ancestor and genealogical kinship paths from parent links.

Families/houses are institutions with charter, founding, cadet branches, heraldry, historical succession and affiliations. A family house is not the same as the set of genetic descendants or a co-resident household. Dynastic rules include elective succession, primogeniture, absolute primogeniture, matrilineal, designated heir, adoption and custom legal exclusions, each attached to a jurisdiction/time window with conflict resolution.

Household membership and residences are intervals; inheritances are transfer events and ownership changes with possible disputes. Do not turn an inheritance into a static "owns" flag without the transfer history.

## 4. Organization and faction affiliations

Organizations include churches, guilds, companies, schools, clubs, political parties, armies, governments, secret societies and social movements. Their own histories track founding, governance, membership qualifications, offices, charter, names, jurisdictions, mergers, schisms, annexations and dissolution.

An **Affiliation/Membership** object has actor/member, target institution, role(s), legal and perceived status, admission/exit dates, offices, authority, initiator/approver, public/private status and source. Institutional membership is not automatic allegiance; supporting an institution in a strike does not imply joining it. An individual can oppose their own organization's official event position.

Organizations may be members or constituent bodies of other organizations, or linked through alliances, rivalries, charters, ownership, certification, control or succession. Reify historically consequential associations. Event-specific organizational participation is a different object from persistent membership.

Query examples: "which families controlled council positions between 1880 and 1910?", "members of the preservation society at a date", "who defected during the Northbridge succession crisis?", "which institutions descend from the dissolved guild?".

## 5. Population ancestry versus social identity

A population is a time/region-aware demographic and genetic sampling construct. A character's biological lineage may include many simulated ancestral populations; social/ethnic/cultural identification, citizenship, language, religion, faction membership and personality must be authored or generated through separate historical/social systems, **never derived as deterministic DNA facts**.

Migration connects people, households and populations to places with dates and source history. Maps/region assignments evolve over time; modern borders cannot be imposed retroactively.

## 6. Synthetic DNA engine: two capability levels

**Level A (early):** pedigree, expected relatedness, relationship paths and illustrative ancestry proportions derived from founder-population membership, explicitly labeled "genealogical inference", not simulated DNA results.

**Level B (later):** deterministic synthetic haplotype/chromosome segment inheritance:
1. Seed founder haplotypes/marker panels and versioned reference populations.
2. Perform reproducible parental recombination using seed, algorithm version and reproductive event.
3. Track segment provenance, chromosome boundaries and segment inheritance through generations.
4. Compute shared segment matches, autosomal relatedness, possible relationship ranges and DNA-relative networks.
5. Where biologically applicable, model mitochondrial lineage and Y-DNA transmission.
6. Produce a chromosome painter, ancestry estimates with uncertainty, reports of inconsistent recorded parentage and historical population effects (bottleneck, founder effect, migration, drift).

Do not use invented whole-genome accuracy claims; marker density, recombination method, reference-population limits and uncertainty must be shown. Distant relatives may share no detectable segments despite documentary ancestry. Re-running the same versioned inputs reproduces output. Editing canonical biological parentage invalidates affected descendant segment simulations and DNA-derived match indexes; it must not silently rewrite historical legal records.

## 7. Queries and visualizations

Pedigree, descendant, fan chart, timeline, household roster, family alliances, ancestor migration map, office succession, relationship evolution chart, genetic-match explorer and event impact graph are alternative projections of the same canonical objects.

Queries must distinguish **biological ancestor**, **legal ancestor**, **family-house member**, **organization member**, **living in household**, **co-actor of event**, and **shares DNA markers**; none is an automatic synonym. Results are scoped by branch, date, revision and audience; historical portraits and notes may have more restrictive publication rules.

## 8. Validation gates

- Biological vs adoptive/legal parent and conflicting rumor can coexist without leaking.
- Multiple tags and roles with independent histories reconstruct accurately.
- A trusts B 90; B trusts A 15; editing one does not mutate the other.
- An organization schism preserves the predecessor's historic identity.
- A guild officer is expelled yet secretly funds the guild; membership and sponsorship histories remain distinct.
- Genealogy-derived cousin path computes on demand and respects parentage type.
- Synthetic DNA re-run is deterministic; inappropriate cultural inferences are absent.
- Editing a parent or population rule invalidates only affected derived genetic products, never canon.
