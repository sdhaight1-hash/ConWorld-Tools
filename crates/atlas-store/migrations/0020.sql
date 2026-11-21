-- Bound discovery's historical typed candidate universe without scanning
-- unrelated source/event/relationship records.
CREATE INDEX IF NOT EXISTS discovery_kind_lineage_idx
    ON object_versions(world_id, kind, branch_id, revision, object_id);
