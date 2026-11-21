-- Rebuild the closed typed registry without changing any existing identities.
CREATE TABLE objects_v7 (
    world_id BLOB NOT NULL REFERENCES worlds(id),
    id BLOB NOT NULL CHECK(length(id) = 16),
    kind TEXT NOT NULL CHECK(kind IN (
        'person', 'historical_event', 'source', 'claim',
        'relationship', 'event_involvement', 'actor_action', 'definition'
    )),
    PRIMARY KEY (world_id, id),
    UNIQUE (world_id, id, kind)
) STRICT;
INSERT INTO objects_v7 SELECT world_id, id, kind FROM objects;
DROP TABLE objects;
ALTER TABLE objects_v7 RENAME TO objects;

CREATE TABLE definition_versions (
    world_id BLOB NOT NULL, definition_id BLOB NOT NULL, branch_id BLOB NOT NULL,
    state_revision INTEGER NOT NULL, revision INTEGER NOT NULL,
    category TEXT NOT NULL CHECK(category IN ('relationship', 'tag', 'role')),
    description TEXT NOT NULL,
    reverse_label TEXT,
    symmetric INTEGER NOT NULL CHECK(symmetric IN (0, 1)),
    PRIMARY KEY (world_id, definition_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, definition_id, branch_id, state_revision, revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision)
) STRICT;
ALTER TABLE event_involvement_versions ADD COLUMN role_definition_id BLOB;
ALTER TABLE claim_versions ADD COLUMN audience TEXT NOT NULL DEFAULT 'author'
    CHECK(audience IN ('author', 'public', 'character'));
ALTER TABLE claim_versions ADD COLUMN reader_id BLOB;
CREATE INDEX claim_by_subject ON claim_versions(world_id, subject_id, claim_id);
