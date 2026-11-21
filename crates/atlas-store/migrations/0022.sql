CREATE TABLE IF NOT EXISTS entry_layout_versions (
    world_id BLOB NOT NULL,
    entry_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    document TEXT NOT NULL CHECK(json_valid(document)),
    published INTEGER NOT NULL CHECK(published IN (0, 1)),
    PRIMARY KEY (world_id, entry_id, revision),
    FOREIGN KEY (world_id, entry_id) REFERENCES objects(world_id, id),
    FOREIGN KEY (world_id, revision) REFERENCES author_revisions(world_id, revision)
) STRICT;
