-- Rebuild the closed typed registry without changing any existing identities.
CREATE TABLE objects_v6 (
    world_id BLOB NOT NULL REFERENCES worlds(id),
    id BLOB NOT NULL CHECK(length(id) = 16),
    kind TEXT NOT NULL CHECK(kind IN (
        'person', 'historical_event', 'source', 'claim',
        'relationship', 'event_involvement', 'actor_action'
    )),
    PRIMARY KEY (world_id, id),
    UNIQUE (world_id, id, kind)
) STRICT;
INSERT INTO objects_v6 SELECT world_id, id, kind FROM objects;
DROP TABLE objects;
ALTER TABLE objects_v6 RENAME TO objects;

CREATE TABLE relationship_versions (
    world_id BLOB NOT NULL, relationship_id BLOB NOT NULL, branch_id BLOB NOT NULL,
    state_revision INTEGER NOT NULL, revision INTEGER NOT NULL,
    subject_id BLOB NOT NULL, target_id BLOB NOT NULL,
    relation_kind TEXT NOT NULL CHECK(length(trim(relation_kind)) > 0),
    trust INTEGER CHECK(trust BETWEEN 0 AND 100),
    PRIMARY KEY (world_id, relationship_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, relationship_id, branch_id, state_revision, revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, subject_id) REFERENCES objects(world_id, id),
    FOREIGN KEY (world_id, target_id) REFERENCES objects(world_id, id),
    CHECK(subject_id <> target_id)
) STRICT;
CREATE INDEX relationship_subject_idx ON relationship_versions(world_id, subject_id, target_id);

CREATE TABLE relationship_tags (
    world_id BLOB NOT NULL, relationship_id BLOB NOT NULL, branch_id BLOB NOT NULL,
    state_revision INTEGER NOT NULL, revision INTEGER NOT NULL,
    tag TEXT NOT NULL CHECK(length(trim(tag)) > 0),
    PRIMARY KEY (world_id, relationship_id, branch_id, state_revision, revision, tag),
    FOREIGN KEY (world_id, relationship_id, branch_id, state_revision, revision)
        REFERENCES relationship_versions(world_id, relationship_id, branch_id, state_revision, revision)
) STRICT;

CREATE TABLE event_involvement_versions (
    world_id BLOB NOT NULL, involvement_id BLOB NOT NULL, branch_id BLOB NOT NULL,
    state_revision INTEGER NOT NULL, revision INTEGER NOT NULL,
    event_id BLOB NOT NULL, participant_id BLOB NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('actor','witness','victim','stakeholder','sponsor')),
    PRIMARY KEY (world_id, involvement_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, involvement_id, branch_id, state_revision, revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, event_id) REFERENCES objects(world_id, id),
    FOREIGN KEY (world_id, participant_id) REFERENCES objects(world_id, id)
) STRICT;
CREATE INDEX involvement_event_idx ON event_involvement_versions(world_id, event_id);

CREATE TABLE actor_action_versions (
    world_id BLOB NOT NULL, action_id BLOB NOT NULL, branch_id BLOB NOT NULL,
    state_revision INTEGER NOT NULL, revision INTEGER NOT NULL,
    event_id BLOB NOT NULL, actor_id BLOB NOT NULL, target_id BLOB,
    action_kind TEXT NOT NULL CHECK(length(trim(action_kind)) > 0),
    intent TEXT NOT NULL CHECK(length(trim(intent)) > 0),
    outcome TEXT NOT NULL CHECK(outcome IN ('attempted','succeeded','failed')),
    PRIMARY KEY (world_id, action_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, action_id, branch_id, state_revision, revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, event_id) REFERENCES objects(world_id, id),
    FOREIGN KEY (world_id, actor_id) REFERENCES objects(world_id, id),
    FOREIGN KEY (world_id, target_id) REFERENCES objects(world_id, id)
) STRICT;
CREATE INDEX action_event_idx ON actor_action_versions(world_id, event_id);

-- Explicitly link a particular actor action to an existing event-caused change.
CREATE TABLE action_effect_links (
    world_id BLOB NOT NULL, target_id BLOB NOT NULL, branch_id BLOB NOT NULL,
    effect_revision INTEGER NOT NULL, action_id BLOB NOT NULL, action_branch BLOB NOT NULL,
    action_state_revision INTEGER NOT NULL, action_revision INTEGER NOT NULL,
    PRIMARY KEY (world_id, target_id, branch_id, effect_revision),
    FOREIGN KEY (world_id, target_id, branch_id, effect_revision)
        REFERENCES historical_effects(world_id, target_id, branch_id, revision),
    FOREIGN KEY (world_id, action_id, action_branch, action_state_revision, action_revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision)
) STRICT;
