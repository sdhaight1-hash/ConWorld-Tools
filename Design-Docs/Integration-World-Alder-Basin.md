# Alder Basin — persistent cross-domain integration fixture

Status: candidate, pending native Rust tests. Based on verified schema-44 head 27daf6c. Does not change production code, schema, acceptance gates or the frozen ontology consolidation branch. Tracks #72 with #4–#10 and #68 integration dependencies; not a new feature.

## Purpose

A coherent fictional basin with three centuries of history, twelve settlements in three districts, twelve houses and mills, four generations per house, a guild, opposing factions, three documented events, and competing treaty accounts. Cultural, linguistic, material and technological profiles use the existing governed descriptors. Faith, currency and commodity are named Concepts only; the fixture does not invent unsupported semantic fields or economic simulation. Geography uses sourced containment, not invented exact geometry. No actual-world personal data.

## Run and retain

Default test uses a temporary on-disk database and a separate restored copy. From the candidate branch:

```sh
cargo +stable test -p atlas-store --locked --test integration_world -- --nocapture
```

To retain the generated world for a compatible schema-44 desktop build, set ATLAS_INTEGRATION_WORLD to a fresh database filename before running the same command. Existing files are refused via create_new; never point at a real author database. Example PowerShell:

```powershell
$env:ATLAS_INTEGRATION_WORLD = Join-Path $PWD 'alder-basin.sqlite'
cargo +stable test -p atlas-store --locked --test integration_world -- --nocapture
Remove-Item Env:ATLAS_INTEGRATION_WORLD
```

The fixture is generated via Store commands, not direct SQL. Local object UUIDs are reproducible within the generated world; world/branch/fact/command IDs are newly generated and preserved across reopen and restore. The retained file is not a committed binary and is not a substitute for a genuine predecessor migration fixture. Use only after the test passes. No file has been generated or inspected by this commit alone.

## Assertions

- Historical physical containment before and after settlement establishment, before author recording, with cycle rejection and unchanged author revision.
- Historical cohort counts and four governed semantic profiles; source-backed culture/language and technology/material relations.
- Explicit unknown versus known zero area; missing basin/district values remain absent.
- Distinct canonical network facts for family, residence, institutions, ownership, event site and confirmed causality; accepted parentage is recorded by native API.
- Author, public and character-designated claims are distinct; public evidence exposes only the released statement and no source; raw semantic references deny public reads.
- Alternative branch records a new allegiance without changing main; inherited fact ID and provenance preserved.
- Actual on-disk reopen and standalone restored archive replay exact IDs and branch results.

Next slices: assert explicit parentage paths, dated succession, conflict impact, saved query and Canvas/Display bindings, author workflow measurements, then native screenshots of the retained fixture. Do not mark complete without execution; test failures identify product or fixture gaps. No production semantic expansion, extra migration, UI mock or screenshot claimed.
