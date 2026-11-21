CREATE TABLE worlds (
    id BLOB PRIMARY KEY CHECK(length(id) = 16),
    name TEXT NOT NULL CHECK(length(trim(name)) > 0)
) STRICT;

CREATE TABLE branches (
    world_id BLOB NOT NULL REFERENCES worlds(id),
    id BLOB NOT NULL CHECK(length(id) = 16),
    is_main INTEGER NOT NULL CHECK(is_main IN (0, 1)),
    PRIMARY KEY (world_id, id)
) STRICT;

CREATE UNIQUE INDEX one_main_branch ON branches(world_id) WHERE is_main = 1;

CREATE TABLE author_revisions (
    world_id BLOB NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    branch_id BLOB NOT NULL,
    command_id BLOB NOT NULL CHECK(length(command_id) = 16),
    author TEXT NOT NULL CHECK(length(trim(author)) > 0),
    rationale TEXT NOT NULL CHECK(length(trim(rationale)) > 0),
    recorded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (world_id, revision),
    UNIQUE (world_id, command_id),
    UNIQUE (world_id, revision, branch_id),
    FOREIGN KEY (world_id, branch_id) REFERENCES branches(world_id, id)
) STRICT;

CREATE TABLE objects (
    world_id BLOB NOT NULL REFERENCES worlds(id),
    id BLOB NOT NULL CHECK(length(id) = 16),
    kind TEXT NOT NULL CHECK(kind IN ('person', 'historical_event', 'source')),
    PRIMARY KEY (world_id, id),
    UNIQUE (world_id, id, kind)
) STRICT;

CREATE TABLE object_versions (
    world_id BLOB NOT NULL,
    object_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    kind TEXT NOT NULL,
    label TEXT NOT NULL CHECK(length(trim(label)) > 0),
    valid_start INTEGER,
    valid_end INTEGER,
    event_day INTEGER,
    earliest_day INTEGER,
    latest_day INTEGER,
    date_label TEXT,
    citation TEXT,
    source_id BLOB,
    source_kind TEXT GENERATED ALWAYS AS ('source') VIRTUAL,
    PRIMARY KEY (world_id, object_id, branch_id, revision),
    FOREIGN KEY (world_id, object_id, kind) REFERENCES objects(world_id, id, kind),
    FOREIGN KEY (world_id, revision, branch_id) REFERENCES author_revisions(world_id, revision, branch_id),
    FOREIGN KEY (world_id, source_id, source_kind) REFERENCES objects(world_id, id, kind),
    CHECK(valid_start IS NULL OR valid_end IS NULL OR valid_start < valid_end),
    CHECK(
        (kind = 'source' AND citation IS NOT NULL AND length(trim(citation)) > 0)
        OR (kind <> 'source' AND citation IS NULL)
    ),
    CHECK(
        (event_day IS NULL AND earliest_day IS NULL AND latest_day IS NULL AND date_label IS NULL)
        OR (kind = 'historical_event' AND (
            (event_day IS NOT NULL AND earliest_day IS NULL AND latest_day IS NULL AND date_label IS NULL)
            OR (event_day IS NULL AND date_label IS NOT NULL AND length(trim(date_label)) > 0
                AND (earliest_day IS NOT NULL OR latest_day IS NOT NULL)
                AND (earliest_day IS NULL OR latest_day IS NULL OR earliest_day <= latest_day))
        ))
    )
) STRICT;

CREATE TABLE invalidations (
    world_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    object_id BLOB NOT NULL,
    PRIMARY KEY (world_id, revision, object_id),
    FOREIGN KEY (world_id, revision) REFERENCES author_revisions(world_id, revision),
    FOREIGN KEY (world_id, object_id) REFERENCES objects(world_id, id)
) STRICT;
