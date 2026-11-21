-- Current public text only, never a second authoritative world database.
-- Stable integer document IDs survive VACUUM INTO and backup/restore.
CREATE TABLE IF NOT EXISTS public_page_search_documents (
    doc_id INTEGER PRIMARY KEY,
    world_id BLOB NOT NULL,
    entry_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    search_text TEXT NOT NULL,
    UNIQUE(world_id,entry_id),
    FOREIGN KEY(world_id,entry_id,revision)
        REFERENCES entry_layout_versions(world_id,entry_id,revision) ON DELETE CASCADE
) STRICT;

CREATE VIRTUAL TABLE IF NOT EXISTS public_page_search_fts USING fts5(
    search_text,
    content='public_page_search_documents',
    content_rowid='doc_id',
    tokenize='trigram case_sensitive 1'
);

CREATE TRIGGER IF NOT EXISTS public_page_search_ai
AFTER INSERT ON public_page_search_documents BEGIN
    INSERT INTO public_page_search_fts(rowid,search_text)
    VALUES(new.doc_id,new.search_text);
END;
CREATE TRIGGER IF NOT EXISTS public_page_search_ad
AFTER DELETE ON public_page_search_documents BEGIN
    INSERT INTO public_page_search_fts(public_page_search_fts,rowid,search_text)
    VALUES('delete',old.doc_id,old.search_text);
END;
