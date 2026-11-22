# Universal Worldbuilding Ontology and Governed Semantic Extensibility

**Status:** Accepted Design v1.0 — implementation readiness and production verification pending.
**Authority:** [Architectural Evolution #72](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72), [explicit owner pre-approval](https://github.com/sdhaight1-hash/ConWorld-Tools/issues/72#issuecomment-7074159822), 2026-11-21 local / 2026-11-22 UTC.
**Owner:** repository owner. **Author:** Lola.
**Baseline:** [1eef250](https://github.com/sdhaight1-hash/ConWorld-Tools/tree/1eef25048d9a7602129e74c7db594eec22b5cd37), schema 36. [Research #71](../Research-Studies/universal-worldbuilding-ontology.md), [entry review](../Entry-Ontology-Review-2026-11-22.md), [ADR-007](../ADRs/adr-007-governed-semantic-extensibility.md).

## 1. Purpose and evidence

The existing historical graph serves people, institutions and history well but lacks consistent semantic coverage for cultures, languages, faiths, technology, materials, species, works and other authored concepts. Complete original Concept, compatible relationship and independently addressable fact obligations first. Main has 20 object kinds, 13 identity creation kinds, 18 network fact kinds and nine presentation templates. Place is independently governed by accepted #68; #67 remains held. The issue owns the shared architecture delta, not extra feature scope or ordinary phase work.

Research #71 compared formal ontology/reference systems and five principal game/TTRPG case studies plus two supplemental references, documented 14 combined competency questions, five lifecycle traces and a 12-task value matrix. Executed isolated tests: 126 equality assertions, six scale parity checks and explicit rejected inputs; indexed rows vs JSON dense 1k/10k/50k medians 0.703/8.024/45.781 vs 2.038/21.485/115.771 ms, bounded output not total work. Follow-up 20 equality assertions, nine invalid cases, novel metadata category and historical unit pin demonstrated. These are synthetic feasibility results, not native usability, full migration, security or performance evidence. See [study §§7,11](../Research-Studies/universal-worldbuilding-ontology.md), exact fixtures and results.

Alternatives: A complete original Concept and governed facts (required baseline); B small curated descriptors on same domain services (preferred starting point); C constrained metadata extensions within accepted controlled extensibility only if demonstrable additional value. Stop optional expansion if A/B suffice. Reject per-category enum proliferation, universal EAV, independent ontology engine, arbitrary scripts and automatic inference.

## 2. Accepted architecture delta

| Existing issue | Accepted direction |
|---|---|
| Core kind and descriptive category conflated | Stable core identity, separately governed compatible classification and semantic profile |
| Missing Concept | Implement original referenceable Concept, not new kind per subject |
| Primitive Definition and string metadata | Shared Rust-governed world-scoped stable descriptor IDs/version pins over existing definitions where compatible |
| Separate stable network/parentage IDs | Evaluate discriminated authorized object/fact references over existing storage; no duplicate fact |
| Person-only endpoints | Complete already specified compatible institution/actor/participant/relationship roles under owning phases; retain specialized validators |
| Layout and query diversity | Reuse single canonical fact, shared typed query and authorized projections across all studios |

Preserve D-001–D-020 and INV-01–16; clarify ADR-007 and D-004/005 implementation direction without weakening reification or stable identity. Definitions version in recorded author time. Assignments and values/facts have valid time, recorded revision, branch, provenance and visibility. Classifications do not assert culture membership, belief, ancestry, language proficiency, ownership or causality. Explicit missing/unrecorded, unknown, zero and contrary claims are different. No automatic subtype inference or privilege inheritance. Core ancestry, succession, temporal/causal and spatial validators cannot be overridden.

Logical contracts (not compiled Rust or reserved tables):
~~~text
WorldRef = ObjectRef(world,id) | FactRef(world,family,id)
DescriptorPin = (world,descriptor_id,version)
SemanticDescriptor = pin, compatible_base_kind, scheme, labels, constraints, capabilities
ClassificationFact = stable_id, subject_ref, descriptor_pin, effective_context, source, scope
FieldDescriptor = pin, approved_datatype, compatible_kind, units/enum/ref constraints,
                  cardinality, unknown_policy
ValueFact = original_fact_ref, field_pin, value_or_explicit_unknown, context, source, scope
~~~

Prefer indexed temporal assignment facts for frequent filters; compact validated definition metadata may use JSON. Reuse existing Definition, record/version, typed validation and commands rather than creating a universal data store. Constrained typed descriptors support approved semantic data only; if a field represents an existing relation, edit the original canonical fact via original command. No parallel stores. Evaluate WorldRef/FactRef adapter at AE-02 preserving world/family/object IDs, branch/revision/time, visibility, evidence links, stable annotations and historical withdrawal/replay. If insufficient, justify ID-preserving reification with a migration proposal. Capability flags must distinguish create/edit/resolve/link/annotate/search/discover/publish/bind/transform; existence of enum alone proves none.

## 3. Classification and authoring boundaries

Concept for named culture, language/script, tradition, belief, material, technology, species, work, commodity, rule/lore; Organization for institution; Person, Household, FamilyHouse, Faction remain distinct; Place for site/territory under #68; Asset for tracked physical instance; Event/Conflict, Source/Claim maintain occurrence and epistemic roles. Initial candidate curated Culture/Language and Material/Technology, compatible Place categories when ready. AE-01 selects minimum useful subset, not all domains mandatory. Use current authoring controls and governed user-defined categories/labels/roles under INV-11. Metadata-only setting-specific category may guide optional fields and compatible pickers without new core type. Unknown lore remains valid; no compulsory fields beyond actual invariants.

Only necessary typed fields for approved data: allowlisted value/ref, dates/intervals, enum, qualitative or quantity/unit constraints as verified. No automatic conversion, forced exclusivity, silent reclassification, executable physics/biology/magic, generic property runtime or arbitrary field/predicate editor. Independent agreement obligations, economic cohorts, production simulators, speculative reproduction and standalone visual ontology editor are not approved by #72. Separate New Feature approval if new product scope needed. Species classification must not silently invoke human genetic models. Profile is semantic metadata; template controls presentation and neither creates facts by selection. Do not infer missing facts or require full lore to save.

## 4. Shared integration and policy

Rust owns validation, authorization, persistence and query; TypeScript renders and submits typed commands, no independent truth. Apply trusted ViewContext, compatible world/kind, definition pin, branch/revision, audience/source policy before traversal, aggregation, facets/counts, labels, suggested entries, render/cache/export. Fail closed for unsupported readers, no hidden profile/parent/source/alias/field metadata leakage. Reauthorize replay, distinguish incomplete/truncated/cancelled from absence. Preserve existing QuerySpec/QueryControl and review flow. No new GPU, ontology library, cloud, AI or third party dependency required.

| Owner | Integration/ownership |
|---|---|
| #4 | Concept, compatible refs, typed classification/approved field queries, saved pins and discovery, index invalidation |
| #5 | Existing Node category/reference controls; Canvas binding original authorized facts; Display scoped render and keyboard/list navigation; profile ≠ template |
| #6 | Family/institution typed endpoints and fact identities; distinct legal/genetic/social semantics |
| #7 | Event/action/causal facts, roles and group agency within original contract |
| #8 | Evidence, knowledge/claims, author correction vs world event, branch/scope privacy from first slice |
| #9 | Existing optional synthetic genetics unaffected; no new algorithm |
| #10 | Archive/migration, compatibility, performance, native regression/offline |
| #68 | Typed Place, containment/role/lens, frames, geometry authoritative. Compatible classification only. SP gates and #67 hold unchanged. |
| #70 | Verified domain operation availability and typed query pins only; WT-01 readiness and recipe scope unchanged. |

#72 owns delta and AE gates, existing work stays with respective phases/features. No new phase, reopening or renamed phase exit. Do not block independent original work unrelated to dependency.

## 5. Migration and implementation sequencing

A: exact-head baseline CI, investigate caption-search deadline failure, review spatial draft and overlap, compare A/B/C three actual author tasks (culture/language, place/material values, novel setting category); record errors, steps, completeness, explanation, maintenance surfaces/history and actual native evidence; establish budgets and defer unnecessary extension. B: complete prerequisite original obligations, minimal typed references and shared descriptor validation in Rust; begin scope policy and tests immediately. C: minimal compatible versioned classification and only justified approved values; protect typed domain rules. D: query/discovery and existing author controls/projection integration. E: verified migration, archive, feature boundary compatibility and native regression. No implementation before applicable dependencies; test-first, small slices, review exact-head CI and real captures when UI exercised.

Migration additive if justified, no schema number reserved. Preserve all existing IDs, kinds, fact/source provenance, valid-time/revision/branch history, aliases, layout/query pins and permissions. Unclassified objects valid; no guessed classification from names/tags. Explicit review of legacy mapping and version compatibility; old pinned interpretation preserved, unsupported pins error. Backup and dry-run schema 36 and then-current fixture, transaction, FKs, semantic parity, reopen/fork, restore on new destination, old binaries reject unsupported schema. Rollback via reviewed revert and verified pre-upgrade backup, not destructive downgrade. Coordinate #67/#68 before competing migration. No raw private schema export or duplicate canonical graph.

## 6. Architecture verification (all unpassed)

- AE-01 baseline and comparator: healthy exact-head CI, resolved/reproduced regression, actual three-task comparison and measured benefit/limits; stop/defer optional extension where simpler enough.
- AE-02 stable object/fact refs: wrong-world/kind/family, rename, branch/date/revision/withdrawal, original identity and evidence/annotation/backlink references; no duplicate facts, atomicity and existing validators.
- AE-03 descriptors: stable compatible pins, dated multi-classification as needed, wrong-kind/world/version, deprecated/unknown, no inferred fact.
- AE-04 approved typed values and refs: missing/unknown/zero, incompatible units/ref/date/cardinality, correction versus change, version pin, canonical command dispatch; defer out-of-scope fields.
- AE-05 shared bounded search/query/discovery: parity, historical/fork/sibling, source/endpoint/privacy before facets/counts/output, cancel/partial, saved pin/invalidations, no candidate auto-promotion.
- AE-06 Node/Canvas/Display existing controls, authorized projection, accessibility, no layout-driven world writes; #68/#70 independent readiness and compatibility tests; no new standalone editor.
- AE-07 migration/archive: dry run, stable IDs/history/pins, FK/semantic checks, backup, rollback and restore, old-version error, safe offline/archive/export.
- AE-08 exact-head CI, native tasks, sparse/dense 1k/10k/50k measured work/latency/memory, cancellation/truncation, offline/no-GPU as applicable, real dated capture. No fabricated SLA or research benchmark substitution.
- AE-09 verified reconciled docs/ADRs, code/tests/PRs, original AT/SP/WT independently tracked, review before closure.

Risks: unnecessary extensibility (A/B comparison and stop), duplicate data (canonical command), hidden scope (preprojection checks), semantic drift (version pins and domain validators), migration regression (additive/backup/reopen/CI). No production code, native capture or production gate passed in this design pass.

## 7. Canonical reconciliation record

01 INV-11 unchanged, elaborated shared descriptors; 03 §§1/6/8 identity/ref metadata distinction; 06 §§1/4 profile vs layout; 07 typed query pins/source/scope; 08 storage/reference/migration additive constraints; 09 AE gates separate; 10 dependencies unchanged; 12 ADR-007 refinement and D-004/005 preservation. [ADR-007](../ADRs/adr-007-governed-semantic-extensibility.md) records exact direction and unresolved physical details. The research disposition is acceptance of bounded architectural evolution, not new product scope or implemented behavior.
