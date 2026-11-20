CREATE TABLE calendar_rules (
    world_id BLOB NOT NULL,
    calendar_id BLOB NOT NULL CHECK(length(calendar_id) = 16),
    revision INTEGER NOT NULL,
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    epoch_day INTEGER NOT NULL,
    leap_every INTEGER,
    leap_month INTEGER,
    PRIMARY KEY (world_id, calendar_id, revision),
    FOREIGN KEY (world_id, revision) REFERENCES author_revisions(world_id, revision),
    CHECK(
        (leap_every IS NULL AND leap_month IS NULL)
        OR (
            leap_every IS NOT NULL AND leap_every BETWEEN 1 AND 65535
            AND leap_month IS NOT NULL AND leap_month BETWEEN 1 AND 65535
        )
    )
) STRICT;

CREATE TABLE calendar_months (
    world_id BLOB NOT NULL,
    calendar_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    month INTEGER NOT NULL CHECK(month BETWEEN 1 AND 65535),
    length INTEGER NOT NULL CHECK(length BETWEEN 1 AND 65535),
    PRIMARY KEY (world_id, calendar_id, revision, month),
    FOREIGN KEY (world_id, calendar_id, revision) REFERENCES calendar_rules(world_id, calendar_id, revision)
) STRICT;
