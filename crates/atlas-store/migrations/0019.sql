-- Parameter declarations belong to one immutable saved-query version.
CREATE TABLE IF NOT EXISTS saved_query_parameters (
    world_id BLOB NOT NULL,
    query_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    declarations TEXT NOT NULL CHECK(length(declarations) BETWEEN 2 AND 16384),
    PRIMARY KEY (world_id, query_id, revision),
    FOREIGN KEY (world_id, query_id, revision)
        REFERENCES saved_query_versions(world_id, query_id, revision)
) STRICT;
