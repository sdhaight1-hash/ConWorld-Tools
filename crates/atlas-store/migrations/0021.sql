-- SQLite maintains these materialized expression indexes transactionally
-- with canonical version writes. Exact Rust span matching remains authoritative.
CREATE INDEX IF NOT EXISTS discovery_person_prefix_idx
    ON object_versions(world_id, lower(substr(label, 1, 3)), branch_id, revision, object_id)
    WHERE kind = 'person';
CREATE INDEX IF NOT EXISTS discovery_alias_prefix_idx
    ON person_alias_versions(world_id, lower(substr(label, 1, 3)), revision, alias_id);
