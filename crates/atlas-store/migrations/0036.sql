CREATE TABLE media_assets (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    name TEXT NOT NULL CHECK(length(trim(name))>0 AND length(name)<=200),
    width INTEGER NOT NULL CHECK(width BETWEEN 1 AND 2048),
    height INTEGER NOT NULL CHECK(height BETWEEN 1 AND 2048),
    sha256 BLOB NOT NULL CHECK(length(sha256)=32),
    bytes BLOB NOT NULL CHECK(length(bytes) BETWEEN 1 AND 2097152),
    revision INTEGER NOT NULL,
    PRIMARY KEY(world_id,id),
    FOREIGN KEY(world_id,revision) REFERENCES author_revisions(world_id,revision)
) STRICT;
CREATE INDEX media_assets_revision ON media_assets(world_id,revision);
