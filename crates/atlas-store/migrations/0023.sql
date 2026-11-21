CREATE TABLE IF NOT EXISTS accepted_parentage (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    parent_id BLOB NOT NULL,
    child_id BLOB NOT NULL CHECK(child_id <> parent_id),
    kind TEXT NOT NULL CHECK(kind IN ('biological','legal','adoptive')),
    starts_at INTEGER NOT NULL,
    ends_before INTEGER CHECK(ends_before IS NULL OR starts_at < ends_before),
    recorded_revision INTEGER NOT NULL,
    withdrawn_revision INTEGER CHECK(withdrawn_revision IS NULL OR recorded_revision < withdrawn_revision),
    PRIMARY KEY (world_id, id),
    FOREIGN KEY(world_id,parent_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,child_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,recorded_revision) REFERENCES author_revisions(world_id,revision),
    FOREIGN KEY(world_id,withdrawn_revision) REFERENCES author_revisions(world_id,revision)
) STRICT;
CREATE INDEX IF NOT EXISTS parentage_parent ON accepted_parentage(world_id,parent_id,starts_at);
CREATE INDEX IF NOT EXISTS parentage_child ON accepted_parentage(world_id,child_id,starts_at);
