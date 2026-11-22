# 08 — Persistence and API Contracts

**Status:** Design contract. See the [implementation foundation](Phase-Foundation.md) for the executed migration and test results.

## Storage invariants

- SQLite project/world database. Foreign keys enabled per connection, migrations transactional, local-first. One world registry with versioned typed payloads, not a universal EAV table for all domain semantics.
- All object cross-references must be world-scoped. A valid ID of the wrong kind is rejected by domain validation, even when the foreign key exists.
- Distinguish canonical historical facts, in-world claims, revision/audit operations, proposals, knowledge and publicly compiled projections. These must not overwrite each other.
- Valid time, recorded author revision and alternative-history branch are distinct axes. Historical correction is a new recorded version, not a new in-world event.
- Derived adjacency, FTS5, backlink, alias/mention, Canvas dependency and publication indexes are rebuildable caches. They are not world authority.

## Logical schema families

| Subsystem | Illustrative tables | Essential constraints/indexes |
|---|---|---|
| Projects | worlds, calendar_rules, branches, author_revisions, command_log | Fork revision pinned; world-scoped command ID idempotency |
| Registry | objects, object_versions, names, aliases | (world_id, object_id) PK/FK, kind checks |
| Relationships | relationship_definitions, relationships, relation_state, tag_definitions, tag_assignments | Endpoint kinds, direction, valid intervals |
| Families | parentage, partnerships, households, house_memberships, succession_rules, titles, inheritances | Accepted ancestry acyclic; parentage kinds distinct |
| Institutions | organizations, affiliations, offices, institutional_links | Membership role histories, organization succession |
| Events | historical_events, event_involvements, actor_actions, event_links, historical_effects | Event/action/effect references, explicit causality |
| Conflicts | conflicts, conflict_issues, conflict_positions, conflict_event_links | Issue-specific multipolar alignment |
| Evidence | sources, claims, claim_evidence, character_knowledge, visibility_policies | Claim/canon separation |
| Entries | layout_documents, layout_versions, template_versions, layout_elements, bound_components | Template pins, stable element IDs |
| Links | explicit_links, image_hotspots, mentions, discovery_proposals, backlink_index | Source/target/visibility history |
| Queries | saved_queries, query_versions, component_dependencies, index_jobs | AST schema, binding parameter types |
| Assets | media, asset_versions, captions, thumbnails, asset_policies | Content hashes, authorization |
| Later genetics | founder_populations, genetic_markers, haplotypes, recombination_runs, dna_matches | Seed and algorithm version |

**SQLite example (illustrative only):**

~~~sql
PRAGMA foreign_keys = ON;
CREATE TABLE worlds (
    world_id TEXT PRIMARY KEY,
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    active_revision INTEGER NOT NULL DEFAULT 0 CHECK(active_revision >= 0)
) STRICT;
CREATE TABLE objects (
    world_id TEXT NOT NULL,
    object_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    current_label TEXT NOT NULL DEFAULT '',
    PRIMARY KEY(world_id, object_id),
    FOREIGN KEY(world_id) REFERENCES worlds(world_id)
) STRICT;
CREATE INDEX objects_by_kind ON objects(world_id, kind);
~~~

The Phase One migrations implement branch overlays, world-scoped typed evidence, interval checks, author history and exact version references for event effects. Later phases add domain-specific endpoints, index consumers and portable archives. Earlier foundation SQL in the v0.2 package is a reference prototype, **not accepted as the project production schema**.

## Transaction contract

Every mutation supplies world, branch, an expected author revision, unique command ID, capability, explicit payload version and rationale. Rust validates types, historic dates, typed endpoints, permissions and effects. In one SQLite transaction it creates the author audit record, revisions facts/effects, invalidates derived indexes and commits. On concurrency conflict return a structured diff/error; never overwrite silently.

One historical event may drive multiple effects and those effects must commit/rollback as one unit. A correction to an event's date creates a new *author revision* of that historical event.

## Command interface

~~~text
create_world(name, calendar, seed)
create_object(kind, label, world, branch, expected_revision)
upsert_relationship(subject, object, kind, history, tags, sources, expected_revision)
record_event(event, involvements, actor_actions, effects, sources, expected_revision)
set_membership(member, organization, roles, interval, source, expected_revision)
change_conflict_position(conflict, issue, participant, position, interval, evidence)
edit_canvas(entry, template_pin, layout_operations, expected_revision)
promote_discovery(proposal, approved_kind, approved_target, expected_revision)
save_query(query_spec, parameters, audience_policy, expected_revision)
publish_entry(entry, policy, expected_revision)
~~~

Each command includes idempotency identity and context omitted from shorthand above. Failed validation must not produce partial objects.

## Query and projection interface

~~~text
get_object(object_id, ViewContext)
get_historical_snapshot(ViewContext, filters)
get_relationship_history(relation_id, ViewContext, period)
get_genealogy(root_person, kind, depth, ViewContext)
get_event_actors(event_id, ViewContext)
get_conflict_timeline(conflict_id, issue, ViewContext)
explain_change(object_id, period, ViewContext)
execute_query(QuerySpec, ViewContext, budgets)
compile_page(entry_id, ViewContext, target)
search_published(SearchSpec, ViewContext)
~~~

Views return typed DTOs, provenance when authorized, opaque continuation cursor, source revision, uncertainty state, and clear truncation/error indicators. All queries share mandatory context. Never send raw unfiltered DB records to Canvas, GPU or public search.

## Layout, link and publication persistence

The Canvas stores stable ElementIds and a versioned primitive/component AST; each component binding has saved query ID or typed inline query plus parameters. Explicit rich links reference object ID and source block/span or image region. A Display artifact is a compiled projection for one world/branch/time/revision/audience/perspective/policy signature, not editable independent truth.

## Archive contract

Versioned world archive contains SQLite database snapshot, semantic manifest, calendars, seed/ruleset versions, branch DAG, evidence/assets with hashes, layouts and template pins, saved queries, source-to-target links and optional genetic simulation metadata. Import must report broken assets, unsupported schema and ID conflicts. World restore should be tested for semantic roundtrip fidelity, not just successful ZIP extraction.

## Error and resilience contracts

At minimum: WrongWorld, WrongKind, RevisionConflict, UnresolvableCalendar, InvalidInterval, InvalidKinshipCycle, InvalidRole, QueryBudgetExceeded, UnknownTemplateVersion, AssetMissing, CapabilityDenied/NotFound and SchemaTooNew. Audit/diagnostic logs must not leak private authored content. Use parameterized SQL, bounded traversals and safe HTML rendering.

## Migration gate

Phase One verifies fresh migrations, upgrade and reopen, foreign-key checks, pinned branch and author-revision behavior, multi-effect event rollback, and SQLite backup restore. Delete and merge policies, derived-index rebuild and full export/import remain gates for the phases that introduce those features. No test in this specification is evidence that the application is implemented.

## Spatial schema/API extension

[13](13-Spatial-and-Nested-Location-Model.md) defines logical Place/PlaceEdge/LocatedAt/Frame/GeometryVersion plus versioned MapSpec/LayerSpec and read-only MapScene. Reuse registry, branch/revision, typed facts, provenance, visibility and audit; exact table names and migration number pending re-audit of #62/#63. Add typed place creation, dated edge/placement edits, optional geometry mutation and explicit candidate promotion through atomic expected-revision commands. Extend query rather than bypass it. Preserve existing records, IDs and branch freeze; test upgrade, wrong-kind/cross-world, rollback, restore and derived rebuild parity. Archive recipes, frames, styles/symbols and assets with pins/hashes; raw SVG/mesh/index is derivative, never canonical. Public release distinct from full author backup.
