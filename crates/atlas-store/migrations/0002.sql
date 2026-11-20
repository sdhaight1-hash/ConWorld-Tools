ALTER TABLE object_versions RENAME TO object_versions_v1;

CREATE TABLE object_versions (
    world_id BLOB NOT NULL,
    object_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    state_revision INTEGER NOT NULL CHECK(state_revision > 0 AND state_revision <= revision),
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
    PRIMARY KEY (world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, object_id, kind) REFERENCES objects(world_id, id, kind),
    FOREIGN KEY (world_id, revision, branch_id) REFERENCES author_revisions(world_id, revision, branch_id),
    FOREIGN KEY (world_id, state_revision, branch_id) REFERENCES author_revisions(world_id, revision, branch_id),
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

INSERT INTO object_versions (
    world_id, object_id, branch_id, revision, kind, label, valid_start,
    valid_end, event_day, earliest_day, latest_day, date_label, citation, source_id, state_revision
)
SELECT world_id, object_id, branch_id, revision, kind, label, valid_start,
    valid_end, event_day, earliest_day, latest_day, date_label, citation, source_id,
    MIN(revision) OVER (PARTITION BY world_id, object_id, branch_id)
FROM object_versions_v1;

DROP TABLE object_versions_v1;
