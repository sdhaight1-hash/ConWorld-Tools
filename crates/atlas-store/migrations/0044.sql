-- Independent AE-05 seek experiment; no change to canonical facts or payload.
CREATE INDEX IF NOT EXISTS semantic_label_seek
    ON object_versions(world_id,label,object_id);
CREATE INDEX IF NOT EXISTS classification_subject_profile
    ON concept_classifications(world_id,subject_id,profile,starts_at,recorded_revision);
