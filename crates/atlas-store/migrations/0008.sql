-- Phase 3 author-only label search: derived documents have stable INTEGER PRIMARY
-- KEY identities. Indexing object_versions.rowid directly would break after VACUUM INTO
-- because SQLite may renumber implicit rowids.
CREATE TABLE object_search_documents (
    doc_id INTEGER PRIMARY KEY,
    world_id BLOB NOT NULL,
    object_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    state_revision INTEGER NOT NULL,
    revision INTEGER NOT NULL,
    label TEXT NOT NULL,
    UNIQUE (world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, object_id, branch_id, state_revision, revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision)
        ON DELETE CASCADE
) STRICT;

CREATE VIRTUAL TABLE object_search_fts USING fts5(
    label,
    content='object_search_documents',
    content_rowid='doc_id',
    tokenize='unicode61'
);

CREATE TRIGGER object_search_documents_ai AFTER INSERT ON object_search_documents BEGIN
    INSERT INTO object_search_fts(rowid, label) VALUES (new.doc_id, new.label);
END;
CREATE TRIGGER object_search_documents_ad AFTER DELETE ON object_search_documents BEGIN
    INSERT INTO object_search_fts(object_search_fts, rowid, label)
    VALUES ('delete', old.doc_id, old.label);
END;
CREATE TRIGGER object_search_documents_au AFTER UPDATE OF label ON object_search_documents BEGIN
    INSERT INTO object_search_fts(object_search_fts, rowid, label)
    VALUES ('delete', old.doc_id, old.label);
    INSERT INTO object_search_fts(rowid, label) VALUES (new.doc_id, new.label);
END;

-- Every new canonical version receives a search document in the same transaction.
CREATE TRIGGER object_versions_search_ai AFTER INSERT ON object_versions BEGIN
    INSERT INTO object_search_documents
        (world_id, object_id, branch_id, state_revision, revision, label)
    VALUES (new.world_id, new.object_id, new.branch_id, new.state_revision, new.revision, new.label);
END;

CREATE TRIGGER object_versions_search_au AFTER UPDATE OF label ON object_versions BEGIN
    UPDATE object_search_documents SET label = new.label
    WHERE world_id = new.world_id AND object_id = new.object_id
      AND branch_id = new.branch_id AND state_revision = new.state_revision
      AND revision = new.revision;
END;

INSERT INTO object_search_documents
    (world_id, object_id, branch_id, state_revision, revision, label)
SELECT world_id, object_id, branch_id, state_revision, revision, label
FROM object_versions ORDER BY world_id, object_id, branch_id, state_revision, revision;
