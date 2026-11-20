CREATE TABLE objects_v5 (
    world_id BLOB NOT NULL REFERENCES worlds(id),
    id BLOB NOT NULL CHECK(length(id) = 16),
    kind TEXT NOT NULL CHECK(kind IN ('person', 'historical_event', 'source', 'claim')),
    PRIMARY KEY (world_id, id),
    UNIQUE (world_id, id, kind)
) STRICT;

INSERT INTO objects_v5 SELECT world_id, id, kind FROM objects;
DROP TABLE objects;
ALTER TABLE objects_v5 RENAME TO objects;

CREATE TABLE claim_versions (
    world_id BLOB NOT NULL,
    claim_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    state_revision INTEGER NOT NULL,
    revision INTEGER NOT NULL,
    subject_id BLOB NOT NULL,
    predicate TEXT NOT NULL CHECK(length(trim(predicate)) > 0),
    statement TEXT NOT NULL CHECK(length(trim(statement)) > 0),
    PRIMARY KEY (world_id, claim_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, claim_id, branch_id, state_revision, revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, subject_id) REFERENCES objects(world_id, id),
    CHECK (claim_id <> subject_id)
) STRICT;

CREATE TABLE historical_effects (
    world_id BLOB NOT NULL,
    target_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    after_state_revision INTEGER GENERATED ALWAYS AS (revision) VIRTUAL,
    event_id BLOB NOT NULL,
    event_branch BLOB NOT NULL,
    event_state_revision INTEGER NOT NULL,
    event_revision INTEGER NOT NULL,
    before_branch BLOB NOT NULL,
    before_state_revision INTEGER NOT NULL,
    before_revision INTEGER NOT NULL,
    day INTEGER NOT NULL,
    PRIMARY KEY (world_id, target_id, branch_id, revision),
    FOREIGN KEY (world_id, target_id, branch_id, after_state_revision, revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, target_id, before_branch, before_state_revision, before_revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, event_id, event_branch, event_state_revision, event_revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision),
    CHECK (event_id <> target_id AND before_revision < revision AND event_revision <= revision)
) STRICT;

CREATE TABLE fork_anchors (
    world_id BLOB NOT NULL,
    child_branch BLOB NOT NULL,
    event_id BLOB NOT NULL,
    event_branch BLOB NOT NULL,
    event_state_revision INTEGER NOT NULL,
    event_revision INTEGER NOT NULL,
    PRIMARY KEY (world_id, child_branch),
    FOREIGN KEY (world_id, child_branch) REFERENCES branch_forks(world_id, branch_id),
    FOREIGN KEY (world_id, event_id, event_branch, event_state_revision, event_revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision)
) STRICT;

CREATE TABLE event_calendar_dates (
    world_id BLOB NOT NULL,
    event_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    state_revision INTEGER NOT NULL,
    revision INTEGER NOT NULL,
    calendar_id BLOB NOT NULL,
    calendar_revision INTEGER NOT NULL,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL CHECK(month BETWEEN 1 AND 65535),
    day INTEGER NOT NULL CHECK(day BETWEEN 1 AND 65535),
    original_label TEXT NOT NULL CHECK(length(trim(original_label)) > 0),
    PRIMARY KEY (world_id, event_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, event_id, branch_id, state_revision, revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, calendar_id, calendar_revision)
        REFERENCES calendar_rules(world_id, calendar_id, revision),
    CHECK(calendar_revision < revision)
) STRICT;

CREATE TABLE calendar_eras (
    world_id BLOB NOT NULL,
    calendar_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    position INTEGER NOT NULL CHECK(position > 0),
    year_one INTEGER NOT NULL,
    descending INTEGER NOT NULL CHECK(descending IN (0, 1)),
    PRIMARY KEY (world_id, calendar_id, revision, name),
    UNIQUE (world_id, calendar_id, revision, position),
    FOREIGN KEY (world_id, calendar_id, revision) REFERENCES calendar_rules(world_id, calendar_id, revision)
) STRICT;
