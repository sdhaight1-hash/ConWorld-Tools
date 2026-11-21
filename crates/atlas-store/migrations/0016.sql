-- Reverse endpoint lookup complements relationship_subject_idx from schema 6.
-- Canonical relationship versions are still resolved at the requested view.
CREATE INDEX IF NOT EXISTS relationship_target_idx
    ON relationship_versions(world_id, target_id, subject_id);
