-- Only dependencies of currently released, public Canvas components.
-- No private statement text or raw result cache stored in this index.
CREATE TABLE IF NOT EXISTS public_dynamic_dependencies (
    world_id BLOB NOT NULL,
    entry_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    element_id BLOB NOT NULL CHECK(length(element_id)=16),
    target_id BLOB NOT NULL CHECK(length(target_id)=16),
    kind TEXT NOT NULL CHECK(kind IN ('references','statements')),
    PRIMARY KEY(world_id,entry_id,element_id),
    FOREIGN KEY(world_id,entry_id,revision) REFERENCES entry_layout_versions(world_id,entry_id,revision) ON DELETE CASCADE
) STRICT;
CREATE INDEX IF NOT EXISTS public_dynamic_target ON public_dynamic_dependencies(world_id,target_id,entry_id);
