-- Derived exact-name lookup; Unicode lowercase keys are generated in Rust.
-- Versioned authored rows are canonical and can rebuild all keys.
CREATE TABLE IF NOT EXISTS person_alias_name_keys (
    world_id BLOB NOT NULL,
    alias_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    name_key TEXT NOT NULL CHECK(length(name_key) > 0),
    PRIMARY KEY (world_id, alias_id, revision),
    FOREIGN KEY (world_id, alias_id, revision)
        REFERENCES person_alias_versions(world_id, alias_id, revision)
) STRICT;

CREATE INDEX IF NOT EXISTS person_alias_name_lookup
    ON person_alias_name_keys(world_id, name_key, revision);
