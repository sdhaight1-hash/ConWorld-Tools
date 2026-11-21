-- Reverse lookup indexes for the previously unindexed reference columns.
-- Existing relationship, claim-subject, event and mention indexes are reused.
CREATE INDEX IF NOT EXISTS object_source_backlink_idx
    ON object_versions(world_id, source_id, revision, object_id);
CREATE INDEX IF NOT EXISTS involvement_role_backlink_idx
    ON event_involvement_versions(world_id, role_definition_id, revision, involvement_id);
CREATE INDEX IF NOT EXISTS action_actor_backlink_idx
    ON actor_action_versions(world_id, actor_id, revision, action_id);
CREATE INDEX IF NOT EXISTS action_target_backlink_idx
    ON actor_action_versions(world_id, target_id, revision, action_id);
