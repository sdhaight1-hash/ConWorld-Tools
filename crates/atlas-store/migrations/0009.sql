-- Phase 3: saved label-query definitions are versioned author metadata,
-- not materialized result sets and not canonical world objects.
CREATE TABLE saved_queries (
    world_id BLOB NOT NULL REFERENCES worlds(id),
    id BLOB NOT NULL CHECK(length(id) = 16),
    PRIMARY KEY (world_id, id)
) STRICT;

CREATE TABLE saved_query_versions (
    world_id BLOB NOT NULL,
    query_id BLOB NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 200),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    filter_kind TEXT NOT NULL CHECK(filter_kind IN ('contains', 'exact', 'fulltext')),
    filter_text TEXT NOT NULL CHECK(length(trim(filter_text)) > 0 AND length(filter_text) <= 800),
    result_limit INTEGER NOT NULL CHECK(result_limit BETWEEN 1 AND 500),
    PRIMARY KEY (world_id, query_id, revision),
    FOREIGN KEY (world_id, query_id) REFERENCES saved_queries(world_id, id),
    FOREIGN KEY (world_id, revision) REFERENCES author_revisions(world_id, revision)
) STRICT;
