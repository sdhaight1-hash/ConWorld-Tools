-- Internal link-caption candidates; never return these rows to readers.
-- A destination's current release and historical visibility are checked on read.
CREATE TABLE IF NOT EXISTS entry_caption_candidates (
    doc_id INTEGER PRIMARY KEY,
    world_id BLOB NOT NULL,
    entry_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    element_id BLOB NOT NULL CHECK(length(element_id)=16),
    target_id BLOB NOT NULL CHECK(length(target_id)=16),
    caption TEXT NOT NULL,
    UNIQUE(world_id,entry_id,element_id),
    FOREIGN KEY(world_id,entry_id,revision)
        REFERENCES entry_layout_versions(world_id,entry_id,revision) ON DELETE CASCADE
) STRICT;

CREATE VIRTUAL TABLE IF NOT EXISTS entry_caption_fts USING fts5(
    caption,
    content='entry_caption_candidates',
    content_rowid='doc_id',
    tokenize='trigram case_sensitive 1'
);

CREATE TRIGGER IF NOT EXISTS entry_caption_ai
AFTER INSERT ON entry_caption_candidates BEGIN
    INSERT INTO entry_caption_fts(rowid,caption) VALUES(new.doc_id,new.caption);
END;
CREATE TRIGGER IF NOT EXISTS entry_caption_ad
AFTER DELETE ON entry_caption_candidates BEGIN
    INSERT INTO entry_caption_fts(entry_caption_fts,rowid,caption)
    VALUES('delete',old.doc_id,old.caption);
END;
