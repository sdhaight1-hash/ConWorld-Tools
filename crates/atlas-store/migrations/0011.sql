-- Phase 3 indexed author-only historical *claim statement* search. The
-- canonical text stays in claim_versions; this derived index is disposable.
-- Explicit INTEGER PRIMARY KEY doc_id survives VACUUM INTO backup.
CREATE TABLE claim_search_documents (
    doc_id INTEGER PRIMARY KEY,
    world_id BLOB NOT NULL,
    claim_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    state_revision INTEGER NOT NULL,
    revision INTEGER NOT NULL,
    statement TEXT NOT NULL,
    UNIQUE (world_id, claim_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, claim_id, branch_id, state_revision, revision)
        REFERENCES claim_versions(world_id, claim_id, branch_id, state_revision, revision)
        ON DELETE CASCADE
) STRICT;

CREATE VIRTUAL TABLE claim_search_fts USING fts5(
    statement,
    content='claim_search_documents',
    content_rowid='doc_id',
    tokenize='unicode61'
);

CREATE TRIGGER claim_search_documents_ai AFTER INSERT ON claim_search_documents BEGIN
    INSERT INTO claim_search_fts(rowid, statement) VALUES (new.doc_id, new.statement);
END;
CREATE TRIGGER claim_search_documents_ad AFTER DELETE ON claim_search_documents BEGIN
    INSERT INTO claim_search_fts(claim_search_fts, rowid, statement)
    VALUES ('delete', old.doc_id, old.statement);
END;
CREATE TRIGGER claim_search_documents_au AFTER UPDATE OF statement ON claim_search_documents BEGIN
    INSERT INTO claim_search_fts(claim_search_fts, rowid, statement)
    VALUES ('delete', old.doc_id, old.statement);
    INSERT INTO claim_search_fts(rowid, statement) VALUES (new.doc_id, new.statement);
END;

CREATE TRIGGER claim_versions_search_ai AFTER INSERT ON claim_versions BEGIN
    INSERT INTO claim_search_documents
        (world_id, claim_id, branch_id, state_revision, revision, statement)
    VALUES (new.world_id, new.claim_id, new.branch_id, new.state_revision,
            new.revision, new.statement);
END;
CREATE TRIGGER claim_versions_search_au AFTER UPDATE OF statement ON claim_versions BEGIN
    UPDATE claim_search_documents SET statement = new.statement
    WHERE world_id = new.world_id AND claim_id = new.claim_id
      AND branch_id = new.branch_id AND state_revision = new.state_revision
      AND revision = new.revision;
END;

INSERT INTO claim_search_documents
    (world_id, claim_id, branch_id, state_revision, revision, statement)
SELECT world_id, claim_id, branch_id, state_revision, revision, statement
FROM claim_versions ORDER BY world_id, claim_id, branch_id, state_revision, revision;

-- Extend the saved-query allowlist without changing its canonical identity,
-- revision semantics, or result caching (there is none).
CREATE TABLE saved_query_versions_v11 (
    world_id BLOB NOT NULL,
    query_id BLOB NOT NULL,
    revision INTEGER NOT NULL CHECK(revision > 0),
    name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 200),
    schema_version INTEGER NOT NULL CHECK(schema_version = 1),
    filter_kind TEXT NOT NULL CHECK(filter_kind IN ('contains', 'exact', 'fulltext', 'claimtext')),
    filter_text TEXT NOT NULL CHECK(length(trim(filter_text)) > 0 AND length(filter_text) <= 800),
    result_limit INTEGER NOT NULL CHECK(result_limit BETWEEN 1 AND 500),
    PRIMARY KEY (world_id, query_id, revision),
    FOREIGN KEY (world_id, query_id) REFERENCES saved_queries(world_id, id),
    FOREIGN KEY (world_id, revision) REFERENCES author_revisions(world_id, revision)
) STRICT;
INSERT INTO saved_query_versions_v11
    (world_id, query_id, revision, name, schema_version, filter_kind, filter_text, result_limit)
SELECT world_id, query_id, revision, name, schema_version, filter_kind, filter_text, result_limit
FROM saved_query_versions;
DROP TABLE saved_query_versions;
ALTER TABLE saved_query_versions_v11 RENAME TO saved_query_versions;
