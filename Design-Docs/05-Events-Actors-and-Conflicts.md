# 05 — Historical Events, Actors, Agency, Affiliations and Conflict Atlas

**Status:** Normative domain specification.

## 1. Event identity and structure

A **HistoricalEvent** is an in-world occurrence or extended process, not an application audit operation. Fields: stable event ID, type, title, tags, possible start/end interval, calendar uncertainty, locations, parent episode, subevents, claims/sources, public/secret aspects and involved objects. A war can contain battles, negotiations, propaganda events and aftermath. A wedding can include engagements, financing and failed objections.

Events can relate to other events via typed links: part-of, caused-by, contributed-to, triggered, enabled, retaliated-for, prevented, contemporaneous-with and allegedly-related. Causal links require explicit evidence/attribution and status (canonical, claimed, hypothetical); ordering alone proves nothing. Event->event graph traversals are bounded and cycle-checked for relations that require a DAG.

## 2. Event involvement is n-ary, typed and time-bounded

One event can have many actor/participant/affiliation links with different roles and intervals. This is not a bag of keywords or a forced set of pairwise interpersonal edges.

Each **EventInvolvement** is independently referenceable and may include:
- participant entity (person, group, institution or faction);
- class: actor, witness, victim, beneficiary, stakeholder, bystander, sponsor, mediator, signatory, organizer, opponent or other world-defined role;
- official role, concealed role, public narrative, uncertainty and evidence;
- affiliation/institution "on whose behalf" the entity acted (if any);
- effective interval, status changes, changes of side, coerced vs voluntary participation;
- optional goals, perceived opportunities, personal knowledge and accountability.

**Actor** means agency exercised through some actual/attempted action; beneficiary/victim/witness/stakeholder need not be agents. An event affiliated with St. Agnes Parish for relief can also be affiliated with Northbridge Mining Company as opposition. Roles can conflict with a participant's enduring organizational membership.

## 3. Actions and intent

An **ActorAction** is an addressable record under a historical event:
- actor and optional appointing/financing organization;
- timestamp/uncertainty, target(s), action kind, role;
- desired objective and motivations *as known, asserted, or uncertain*;
- resources/constraints, decision authorization and knowledge at that time;
- attempt/outcome/observations, success state and evidence;
- affected relationships, organizations or assets through explicit HistoricalEffects.

An attempted sabotage is not successful sabotage. False rumor of sabotage is a Claim and possible information-propagation effect. Public blame can attach to an actor without canonically proving guilt. Actors can change allegiance over the event timeline; preserve phase-specific involvement.

## 4. Historical effects and action consequences

**HistoricalEffect** links event/action -> target version/change with direction, delta or before/after version, effect category and causal rationale. Examples:
- end friendship tag, begin estrangement tag;
- change subjective trust in one direction;
- admit or expel member; transfer official authority;
- transfer ownership/inheritance; create debt;
- form faction; alter issue-specific stance;
- create belief or publicity; record death, displacement or destruction.

All connected changes commit atomically and retain the originating event and source. Importantly, **an authored correction** of an event date is a software revision, not another historical effect or duplicate fictional event.

## 5. Conflict as first-class process

A **Conflict** is a time-bounded, hierarchical, contested process with issue/stakes, grievances, resources, actors, factions, phases, linked events and settlements. It is not equivalent to a single battle or a two-sided edge. Conflict kinds include interpersonal, familial, legal, labor, economic, political, religious, organizational, covert and military.

A **ConflictPosition** belongs to a participant and *specific issue*, with stance, role, effective interval, public/secret alignment, evidence and optional strategic objective. One faction can support workers' wages, oppose certain tactics, and act as mediator on safety regulation simultaneously. Neutrality, mediation, opportunism, competing groups, defections and secret sponsorship all exist independently.

Conflict phases may include grievance, mobilization, escalation, confrontation, negotiation, settlement and aftermath; descriptive intensity scores are optional. Resolutions can leave open issues. A conflict may merge, split, recur or be linked to another conflict.

## 6. Conflict and event affiliation maps

Event affiliations express organizer, finance, oppose, relief, enforcement, endorsement, jurisdiction, beneficiary or other contextual involvement. Organizational genealogy tells who succeeded whom; event-specific alliances and opposition tell who cooperated in this incident. Do not replace membership with affiliation.

Visualization lenses:
- Actor-action chain: who authorized, who executed, who funded, who witnessed.
- Multipolar faction network: alliances/opposition qualified by issue and date.
- Event causality graph: documented causal and alleged connections styled distinctly.
- Geographic layer: event sites, evolving control/jurisdiction when historical map data exists.
- Timeline and relationship impact: choose date to inspect who changed sides, why trust shifted and what settlements occurred.

Event/Conflict Biography must use source-linked known facts, explicitly mark contested interpretations and retain unexplained gaps.

## 7. Example: Northbridge mining strike

In 1892 safety grievances arise. In May 1893 workers strike; Miners' Mutual organizes, mining company opposes, parish provides food, constabulary enforces public-order measures, merchant family funds negotiations. Individual guild officers may privately support the strike despite institutional opposition. In July violence escalates; by September negotiation achieves partial settlement; in 1895 political disputes linger.

The event model holds *who took which actions*; the conflict model holds *issue-level positions and phases*; the organization model holds persistent memberships; relationship change records show how trust, alliances, property and offices changed. Their temporal and causal links provide a traceable 1892/1893/1895 reconstruction.

## 8. Why/impact query contract

Given event E, return recorded direct effects on typed objects, then optionally a bounded causal expansion via supported event/action/effect links. Distinguish confirmed, disputed, alleged and inferred routes. Given a later relationship R, return supporting change events and actor actions; if there is no recorded cause, say **unknown**, not an invented explanation.

## 9. Acceptance gates

- Four-party treaty with mediator, opponent and signatories is represented as one event.
- A person has multiple involvement roles and an event-specific affiliation.
- Failed action plus damaging rumor does not turn into canonical successful action.
- Conflict has 3+ parties and a neutral mediator; one party changes position between dates.
- A dispute causes explicit change in one directional relationship, with before/after, source, tags and "why".
- A related but merely earlier incident is not treated as causal.
- Events have organizations as participants and can be attributed to secret or public sponsors without revealing that fact to unauthorized viewers.
- Multi-hop impact traversal obeys depth, audience and truncation budgets.

## 10. Spatial event and conflict effects

[13](13-Spatial-and-Nested-Location-Model.md) supplies typed event site/affected region associations and dated jurisdiction, control and claim facts. Conquest, annexation, destruction and boundary change are explicit sourced effects, not map recoloring or inferred cause. Multiple sites, uncertain site claims and competing control must remain distinct. Historical MapView uses same branch/time/revision/audience as causal narrative; no interpolation asserted as historical fact.
