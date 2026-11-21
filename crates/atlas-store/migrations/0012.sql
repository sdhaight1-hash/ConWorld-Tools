-- Phase 3: persist validated typed composite predicate trees with the
-- existing versioned saved-query identity; never cache query results.
CREATE TABLE saved_query_versions_v12 (
    world_id BLOB NOT NULL,
    query_id BLOB NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 200),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    filter_kind TEXT NOT NULL CHECK(filter_kind IN ('contains', 'exact', 'fulltext', 'claimtext', 'composite')),
    filter_text TEXT NOT NULL CHECK(
        length(trim(filter_text)) > 0
        AND length(filter_text) <= CASE WHEN filter_kind = 'composite' THEN 4096 ELSE 800 END
    ),
    result_limit INTEGER NOT NULL CHECK(result_limit BETWEEN 1 AND 500),
    PRIMARY KEY (world_id, query_id, revision),
    FOREIGN KEY (world_id, query_id) REFERENCES saved_queries(world_id, id),
    FOREIGN KEY (world_id, revision) REFERENCES author_revisions(world_id, revision)
) STRICT;

INSERT INTO saved_query_versions_v12
    (world_id, query_id, revision, name, schema_version, filter_kind, filter_text, result_limit)
SELECT world_id, query_id, revision, name, schema_version, filter_kind, filter_text, result_limit
FROM saved_query_versions;
DROP TABLE saved_query_versions;
ALTER TABLE saved_query_versions_v12 RENAME TO saved_query_versions;
