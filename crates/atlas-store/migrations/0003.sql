CREATE TABLE branch_forks (
    world_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    parent_id BLOB NOT NULL,
    fork_day INTEGER NOT NULL,
    inherited_revision INTEGER NOT NULL CHECK(inherited_revision >= 0),
    created_revision INTEGER NOT NULL CHECK(created_revision = inherited_revision + 1),
    PRIMARY KEY (world_id, branch_id),
    FOREIGN KEY (world_id, branch_id) REFERENCES branches(world_id, id),
    FOREIGN KEY (world_id, parent_id) REFERENCES branches(world_id, id),
    FOREIGN KEY (world_id, created_revision, parent_id) REFERENCES author_revisions(world_id, revision, branch_id),
    CHECK(branch_id <> parent_id)
) STRICT;

CREATE TABLE branch_overrides (
    world_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    object_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    starts_at INTEGER NOT NULL,
    PRIMARY KEY (world_id, branch_id, object_id),
    FOREIGN KEY (world_id, branch_id) REFERENCES branch_forks(world_id, branch_id),
    FOREIGN KEY (world_id, object_id) REFERENCES objects(world_id, id),
    FOREIGN KEY (world_id, revision, branch_id) REFERENCES author_revisions(world_id, revision, branch_id)
) STRICT;
