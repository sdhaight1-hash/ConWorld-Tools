CREATE TABLE IF NOT EXISTS custom_canvas_templates (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    revision INTEGER NOT NULL,
    name TEXT NOT NULL CHECK(length(trim(name))>0),
    source_id BLOB NOT NULL,
    document TEXT NOT NULL CHECK(length(document)<=1048576),
    PRIMARY KEY(world_id,id,revision),
    FOREIGN KEY(world_id,source_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,revision) REFERENCES author_revisions(world_id,revision)
) STRICT;
CREATE INDEX IF NOT EXISTS custom_template_latest ON custom_canvas_templates(world_id,id,revision DESC);
