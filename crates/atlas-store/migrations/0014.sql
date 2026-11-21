-- Phase 3: versioned author-only alternative names for stable person IDs.
-- Main-branch metadata until branch-scoped alias lineage has its own contract.
CREATE TABLE IF NOT EXISTS person_aliases (
    world_id BLOB NOT NULL REFERENCES worlds(id),
    id BLOB NOT NULL CHECK(length(id) = 16),
    person_id BLOB NOT NULL,
    person_kind TEXT GENERATED ALWAYS AS ('person') VIRTUAL,
    PRIMARY KEY (world_id, id),
    FOREIGN KEY (world_id, person_id, person_kind) REFERENCES objects(world_id, id, kind)
) STRICT;

CREATE TABLE IF NOT EXISTS person_alias_versions (
    world_id BLOB NOT NULL,
    alias_id BLOB NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    label TEXT NOT NULL CHECK(length(trim(label)) > 0 AND length(label) <= 200),
    starts_at INTEGER,
    ends_before INTEGER,
    active INTEGER NOT NULL CHECK(active IN (0, 1)),
    PRIMARY KEY (world_id, alias_id, revision),
    FOREIGN KEY (world_id, alias_id) REFERENCES person_aliases(world_id, id),
    FOREIGN KEY (world_id, revision) REFERENCES author_revisions(world_id, revision),
    CHECK(starts_at IS NULL OR ends_before IS NULL OR starts_at < ends_before)
) STRICT;

CREATE INDEX IF NOT EXISTS person_aliases_person ON person_aliases(world_id, person_id);
