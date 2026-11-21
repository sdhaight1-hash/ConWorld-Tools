CREATE TABLE IF NOT EXISTS entry_link_index (
    world_id BLOB NOT NULL,
    entry_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    element_id BLOB NOT NULL CHECK(length(element_id)=16),
    target_id BLOB NOT NULL CHECK(length(target_id)=16),
    PRIMARY KEY(world_id,entry_id,revision,element_id),
    FOREIGN KEY(world_id,entry_id,revision)
        REFERENCES entry_layout_versions(world_id,entry_id,revision) ON DELETE CASCADE
) STRICT;
CREATE INDEX IF NOT EXISTS entry_link_target
    ON entry_link_index(world_id,target_id,entry_id,revision);
