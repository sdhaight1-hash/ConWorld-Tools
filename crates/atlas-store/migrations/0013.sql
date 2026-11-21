-- Phase 3: efficient candidate discovery for person-to-event role queries.
-- Event lookup already has involvement_event_idx from schema 0006.
-- All matches are revalidated against the authoritative historical view.
CREATE INDEX IF NOT EXISTS involvement_participant_idx
    ON event_involvement_versions(world_id, participant_id, involvement_id);
