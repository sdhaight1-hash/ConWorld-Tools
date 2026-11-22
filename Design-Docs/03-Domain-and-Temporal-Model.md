# 03 — Domain, Identity, Time and Knowledge Model

**Status:** Normative semantics; Rust examples are design contracts, not compiled production types.

## 1. Universal object registry, typed payloads

Addressable kinds include World, Person, FamilyHouse, Household, Organization, Faction, Place, Asset, Concept, HistoricalEvent, Conflict, Population, Relationship, Membership, EventInvolvement, ActorAction, Claim, Source, TagAssignment and optional genetic profile. Every object uses world-scoped stable identity. A relationship, involvement or membership is a reified object precisely because it needs its own tags, evidence, actions, versions and backlinks.

**Do not use one generic EAV table as the entire domain model.** Store identity/kind in the registry; type-specific validated payloads in typed tables. Flexible definitions (tags, custom relationship names, event role vocabularies) are governed metadata over typed endpoints.

~~~rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct WorldId(Uuid);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct ObjectId(Uuid);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct BranchId(Uuid);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorldDay(i64);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct AuthorRevision(u64);

enum ObjectKind {
    Person, FamilyHouse, Household, Organization, Faction, Place, Asset,
    Concept, HistoricalEvent, Conflict, Population, Relationship,
    Membership, EventInvolvement, ActorAction, Claim, Source, TagAssignment
}
struct ObjectRef { world: WorldId, id: ObjectId, kind: ObjectKind }
~~~

Definitions can be user-defined, but new kinds require explicit schema/validation or a documented generic Entry extension. Avoid hard-coded magic type strings in every subsystem.

## 2. Time coordinates and uncertainty

There are **three independent coordinates**:

1. **Valid/world time:** when something is true or an event occurs in fiction.
2. **Recorded/author time:** when the author committed, corrected or retracted a statement.
3. **Branch:** which alternative-history line applies.

For exact intervals, use half-open [start, end) endpoints in normalized world-day coordinates. Unknown and approximate dates require a distinct structured value rather than an invented exact endpoint.

~~~rust
enum DatePrecision { Day, Month, Year, Century, Range, Unknown }
enum TemporalValue {
    Exact { calendar: CalendarId, day: WorldDay, original_label: String },
    Uncertain { calendar: CalendarId, earliest: Option<WorldDay>,
                latest: Option<WorldDay>, precision: DatePrecision,
                original_label: String },
    Unknown
}
struct ValidInterval { start: Option<WorldDay>, end_exclusive: Option<WorldDay> }
struct FactVersion {
    object: ObjectId, version: VersionId, branch: BranchId,
    valid: Option<ValidInterval>, revised_from: AuthorRevision,
    revised_until: Option<AuthorRevision>, provenance: Vec<ObjectId>
}
~~~

An exact snapshot predicate returns true/false for exact intervals. Uncertain assertions require a three-valued outcome **definitely true / possibly true / not established** for a query instant. Authors can explicitly override uncertain interpretations with new evidence. A death recorded as "spring 1908" must not become March 1 automatically.

World calendars are versioned and map representable instants to monotonically ordered world-day numbers. Before arbitrary calendars are supported, an ADR must settle BCE/negative years, leap rules, discontinuities and calendars without exact cross-conversion. Never compare unresolvable clocks as if they share a calendar.

## 3. Revision/correction vs fictional event

A fictional birth, assassination or treaty is a **HistoricalEvent**. A software command creating that record is an **AuthorRevision/AuditOperation**. Correcting the reported fictional day of the assassination creates a new recorded revision of the *same event*, not a second fictional assassination.

Operations are atomic and carry author, timestamp, command ID, precondition revision, affected IDs and rationale. Canonical history can be corrected while audit history remains intact.

## 4. Branch overlays

A branch records parent branch, fork world-day (or exact event boundary), inherited author revision, and optional fork-point object. A child branch reads the parent's history **as frozen at the fork**, overlaying child versions after the fork. Parent revisions made later do not silently change an existing child unless a deliberate rebase/merge occurs.

Branch-specific effects never leak to main; edits that contradict the fork cannot be applied silently. Full multi-branch merge is deferred to an explicit conflict resolution design. Source branch and fork lineage are exposed in provenance.

## 5. Fact vs claim vs knowledge

- **Canonical fact:** author-established assertion about the world, versioned and conflict-validated.
- **Claim:** statement made by a person, document, institution or narrator, possibly wrong. References predicates/objects with the asserting source and dates.
- **Evidence/source:** document, recollection, image, witness report, decree, DNA-style simulated report; source can have its own chain of custody and contested reliability.
- **Belief/knowledge:** what a particular entity knew, suspected, misunderstood or concealed at a given date. It is linked to a claim/evidence and acquisition event, not a copy of global canon.
- **Proposal:** unapproved discovery or AI hypothesis. Not visible as canonical truth.

Truth, belief, publication visibility and certainty are independent axes. A person may believe a false parentage record; a journalist may report it; an author may know the biological fact; a public reader may see neither. Do not use numerical confidence as a substitute for a supported source and explicit epistemic state.

## 6. Relationship as historical object

Typed relationship definitions specify allowed endpoint kinds, direction, inverse label, optional symmetry, overlap/cardinality, and validation. An instance links two IDs and has effective period(s), tags, optional changing asymmetric state, claims and explicit causes. Multiple relationships between same objects are allowed. Derive sibling/cousin/ancestor from biological/legal parentage paths; do not permanently store all inferred degrees of kinship.

Membership, title holding, legal office and event involvement may use their own types, not just arbitrary generic edges, because they contain unique invariants and role histories.

## 7. Change effects and explanations

A historical effect links **the originating event or actor action**, target object, before/after version or state delta, what changed, evidence/attribution and causal status. Can affect an edge, membership, ownership, belief, conflict stance or title. An impact chain is traversal over *recorded links*, not accidental temporal adjacency. "Cause unknown" is an acceptable state.

## 8. Query context is mandatory

~~~rust
struct ViewContext {
    world: WorldId,
    branch: BranchId,
    at_world_time: Option<TemporalValue>,
    as_of_author_revision: AuthorRevision,
    audience: AudienceId,
    perspective: Perspective,
    capability: AccessCapability
}
~~~

All read APIs, FTS, graph scenes, nested Canvas components, asset access and exports take this context or a derived no-more-powerful form. An absent context is a type/validation error, not implicit author mode. Exposed source revision allows cache coherence.

## 9. Integrity checks

World-scoped composite foreign keys; no orphan tags or edges; valid typed endpoints; no biological ancestry cycles; chronology policy catches impossible exact dates while permitting explicit speculative settings; invalid conflicting canonical state rejected or flagged for author arbitration; event/action effect transactions atomic; unknown dates remain uncertain; stable ID survives rename. Destructive merges require a link/effect/asset impact review and never silently erase history.

## 10. Spatial identity and history

[13](13-Spatial-and-Nested-Location-Model.md) is normative for Place, versioned frames/geometry, strict typed containment, location roles and their branch/interval semantics. Place is already a registry kind in this design; actual implementation remains pending. Validate strict cycles over co-effective intersections in author mutation graph; do not mix physical/admin lenses. Geometry is optional, reparenting and ownership independent, moving interior identity stable. Claims and draft maps never mutate canonical geography without explicit acceptance.

## Accepted ontology evolution #72 — contract reconciliation

[Accepted Architectural Evolution #72](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72) ([design](Architectural-Evolutions/universal-worldbuilding-ontology-and-governed-semantic-extensibility.md), [ADR-007](ADRs/adr-007-governed-semantic-extensibility.md), [approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72#issuecomment-7074159822)). AE-01–09 production gates remain unpassed; original phase/feature gates unchanged.

Complete original Concept; separate stable core identities, descriptor metadata/pins, dated classifications, approved values, reified facts, evidence and presentation. Evaluate authorized discriminated object/fact resolver preserving existing registry/network/parentage IDs. Definitions version in recorded time; assignments/facts preserve valid time/revision/branch/source/scope. No duplicate facts or automatic classification inference. Physical schema open.
