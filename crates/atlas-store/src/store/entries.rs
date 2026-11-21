//! Main-branch authored layouts and explicit, revocable publication.
use super::*;
use crate::layout::{
    DisplayContent, DisplayElement, EntryBacklink, EntryBacklinks, EntryTextSearch, LayoutDocument,
    PublicPageMatch, PublicPageSearch,
};

impl Store {
    pub fn save_entry_layout(
        &mut self,
        view: ViewContext,
        document: &LayoutDocument,
        published: bool,
        command_id: Uuid,
    ) -> Result<AuthorRevision, Error> {
        require_author(view.scope)?;
        if document.entry.world != view.world {
            return Err(Error::WrongWorld);
        }
        // Validate all author bindings against the same immutable historical view.
        document.compile(self, view)?;
        if published {
            document.validate_publication()?;
        }
        let text =
            serde_json::to_string(document).map_err(|_| Error::InvalidInput("layout encoding"))?;
        if text.len() > 1_048_576 {
            return Err(Error::QueryBudgetExceeded);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = load_world(&tx, view.world)?;
        if world.main != view.branch {
            return Err(Error::CapabilityDenied);
        }
        let revision = append_revision(
            &tx,
            &world,
            world.main,
            view.revision,
            command_id,
            ("Local author", "Save entry layout and publication state"),
        )?;
        resolve_object(&tx, document.entry, view.branch, view.revision, view.day)?
            .ok_or(Error::NotFound)?;
        // Stable targets can be temporarily absent at the selected fictional day,
        // but a typo must not silently create an unresolved reference.
        for (_, link) in document.visible_links(Scope::Author) {
            if kind_of(&tx, link.target)?.is_none() {
                return Err(Error::NotFound);
            }
        }
        tx.execute(
            "INSERT INTO entry_layout_versions(world_id,entry_id,revision,document,published)
            VALUES (?1,?2,?3,?4,?5)",
            params![
                view.world.0.as_bytes(),
                document.entry.id.0.as_bytes(),
                revision,
                text,
                published
            ],
        )?;
        replace_public_page_index(&tx, document, published, revision)?;
        index_entry_links(&tx, document, revision)?;
        tx.commit()?;
        Ok(AuthorRevision(revision))
    }

    /// Compile an unsaved author draft without modifying publication or canonical state.
    pub fn preview_author_layout(
        &self,
        view: ViewContext,
        document: &LayoutDocument,
        control: &QueryControl,
    ) -> Result<Option<DisplayElement>, Error> {
        require_author(view.scope)?;
        if document.entry.world != view.world {
            return Err(Error::WrongWorld);
        }
        document.validate()?;
        let text =
            serde_json::to_string(document).map_err(|_| Error::InvalidInput("layout encoding"))?;
        if text.len() > 1_048_576 {
            return Err(Error::QueryBudgetExceeded);
        }
        self.with_query_control(control, |store| {
            {
                let tx = store.conn.unchecked_transaction()?;
                let world = load_world(&tx, view.world)?;
                if world.main != view.branch {
                    return Err(Error::CapabilityDenied);
                }
                if view.revision.0 < 0 || view.revision > world.revision {
                    return Err(Error::InvalidRevision);
                }
                resolve_object(&tx, document.entry, view.branch, view.revision, view.day)?
                    .ok_or(Error::NotFound)?;
            }
            document.compile(store, view)
        })
    }

    /// Editable definitions are author-only. Older author revisions remain reproducible.
    pub fn entry_layout(
        &self,
        view: ViewContext,
        entry: ObjectRef,
    ) -> Result<Option<(LayoutDocument, bool)>, Error> {
        require_author(view.scope)?;
        self.read_layout(view, entry)
    }

    /// Reader entry point. Only the persisted document can supply display content.
    pub fn display_entry(
        &self,
        view: ViewContext,
        entry: ObjectRef,
    ) -> Result<Option<DisplayElement>, Error> {
        if view.scope != Scope::Author {
            let tx = self.conn.unchecked_transaction()?;
            let Some((document, _)) = read_layout_at(&tx, view, entry)? else {
                return Ok(None);
            };
            return document.compile(self, view);
        }
        let Some((document, _)) = self.read_layout(view, entry)? else {
            return Ok(None);
        };
        document.compile(self, view)
    }

    /// Readers can navigate only to a released destination in their own scope.
    /// Never compile the destination: cyclic navigation must not recurse.
    pub(crate) fn entry_link_target_visible(
        &self,
        view: ViewContext,
        target: ObjectRef,
    ) -> Result<bool, Error> {
        if view.world != target.world {
            return Err(Error::WrongWorld);
        }
        if view.scope == Scope::Author {
            self.validate_query_view(view)?;
            return Ok(
                resolve_object(&self.conn, target, view.branch, view.revision, view.day)?.is_some(),
            );
        }
        Ok(
            read_layout_at(&self.conn, view, target)?.is_some_and(|(document, _)| {
                crate::access::can_view(view.scope, document.root.audience)
            }),
        )
    }

    /// Explicit presentation references, not semantic graph edges.
    pub fn entry_backlinks(
        &self,
        view: ViewContext,
        target: ObjectRef,
        limit: usize,
        control: &QueryControl,
    ) -> Result<EntryBacklinks, Error> {
        if !(1..=50).contains(&limit) {
            return Err(Error::InvalidInput("entry backlink limit"));
        }
        self.with_query_control(control, |store| {
            let tx = store.conn.unchecked_transaction()?;
            let world = load_world(&tx, view.world)?;
            if world.main != view.branch {
                return Err(Error::CapabilityDenied);
            }
            let mut result = EntryBacklinks {
                matches: Vec::new(),
                truncated: false,
            };
            if !store.entry_link_target_visible(view, target)? {
                return Ok(result);
            }
            let mut stmt = tx.prepare(
                "SELECT DISTINCT l.entry_id FROM entry_link_index l
                 WHERE l.world_id=?1 AND l.target_id=?2 AND l.revision<=?3
                   AND l.revision=(SELECT MAX(v.revision) FROM entry_layout_versions v
                       WHERE v.world_id=l.world_id AND v.entry_id=l.entry_id AND v.revision<=?3)
                 ORDER BY l.entry_id",
            )?;
            let ids = stmt.query_map(
                params![
                    view.world.0.as_bytes(),
                    target.id.0.as_bytes(),
                    view.revision.0
                ],
                |row| row.get::<_, [u8; 16]>(0),
            )?;
            for id in ids {
                store.check_query_control()?;
                let source = ObjectRef {
                    world: view.world,
                    id: ObjectId(Uuid::from_bytes(id?)),
                };
                let Some((document, _)) = read_layout_at(&tx, view, source)? else {
                    continue;
                };
                for (element, link) in document.visible_links(view.scope) {
                    if link.target != target {
                        continue;
                    }
                    if result.matches.len() == limit {
                        result.truncated = true;
                        return Ok(result);
                    }
                    result.matches.push(EntryBacklink {
                        source,
                        element,
                        label: link.label.clone(),
                    });
                }
            }
            Ok(result)
        })
    }

    /// Find literal, case-insensitive text within a single authorized entry.
    /// Never search stored layout JSON or the author full-text index.
    pub fn search_entry_text(
        &self,
        view: ViewContext,
        entry: ObjectRef,
        text: &str,
        limit: usize,
    ) -> Result<Option<EntryTextSearch>, Error> {
        let needle = entry_search_term(text, limit)?;
        let Some(page) = self.display_entry(view, entry)? else {
            return Ok(None);
        };
        Ok(Some(find_text_blocks(page, &needle, limit)))
    }

    /// Search released public pages at the current revision and selected day.
    /// The author registry, labels and private blocks are never search inputs.
    pub fn search_public_pages(
        &self,
        view: ViewContext,
        text: &str,
        limit: usize,
        control: &QueryControl,
    ) -> Result<PublicPageSearch, Error> {
        if view.scope != Scope::Public {
            return Err(Error::CapabilityDenied);
        }
        let needle = entry_search_term(text, limit)?;
        self.with_query_control(control, |store| {
            let tx = store.conn.unchecked_transaction()?;
            let world = load_world(&tx, view.world)?;
            if world.main != view.branch {
                return Err(Error::CapabilityDenied);
            }
            if view.revision != world.revision {
                return Err(Error::InvalidRevision);
            }
            // Narrow only on released public text. Short terms retain the
            // bounded fallback, since the trigram tokenizer cannot match them.
            let phrase = if needle.chars().count() >= 3 && !needle.contains('\0') {
                Some(format!("\"{}\"", needle.replace('"', "\"\"")))
            } else {
                None
            };
            let sql = if phrase.is_some() {
                "SELECT d.entry_id FROM public_page_search_fts
                 JOIN public_page_search_documents d ON d.doc_id=public_page_search_fts.rowid
                 WHERE d.world_id=?1 AND public_page_search_fts MATCH ?2
                 ORDER BY d.entry_id"
            } else {
                "SELECT entry_id FROM public_page_search_documents
                 WHERE world_id=?1 AND ?2 IS NULL ORDER BY entry_id"
            };
            let mut stmt = tx.prepare(sql)?;
            let ids = stmt.query_map(params![view.world.0.as_bytes(), phrase], |row| {
                row.get::<_, [u8; 16]>(0)
            })?;
            let mut result = PublicPageSearch {
                matches: Vec::new(),
                truncated: false,
            };
            let mut visible_pages = 0;
            for id in ids {
                store.check_query_control()?;
                let entry = ObjectRef {
                    world: view.world,
                    id: ObjectId(Uuid::from_bytes(id?)),
                };
                let Some((document, _)) = read_layout_at(&tx, view, entry)? else {
                    continue;
                };
                let Some(page) = document.compile(store, view)? else {
                    continue;
                };
                visible_pages += 1;
                if visible_pages > 512 {
                    return Err(Error::QueryBudgetExceeded);
                }
                let remaining = limit - result.matches.len();
                let found = find_text_blocks(page, &needle, remaining);
                for (element, text) in found.matches {
                    result.matches.push(PublicPageMatch {
                        entry,
                        element,
                        text,
                    });
                }
                if found.truncated {
                    result.truncated = true;
                    break;
                }
            }
            Ok(result)
        })
    }

    fn read_layout(
        &self,
        view: ViewContext,
        entry: ObjectRef,
    ) -> Result<Option<(LayoutDocument, bool)>, Error> {
        let tx = self.conn.unchecked_transaction()?;
        read_layout_at(&tx, view, entry)
    }
}

fn read_layout_at(
    conn: &Connection,
    view: ViewContext,
    entry: ObjectRef,
) -> Result<Option<(LayoutDocument, bool)>, Error> {
    if entry.world != view.world {
        return Err(Error::WrongWorld);
    }
    let world = load_world(conn, view.world)?;
    if world.main != view.branch {
        return Err(Error::CapabilityDenied);
    }
    if view.revision.0 < 0
        || view.revision > world.revision
        || (view.scope != Scope::Author && view.revision != world.revision)
    {
        return Err(Error::InvalidRevision);
    }
    if let Scope::Character(reader) = view.scope {
        require_person(
            conn,
            view.world,
            reader,
            view.branch,
            view.revision.0,
            view.day,
        )?;
    }
    let row: Option<(bool, String)> = conn.query_row(
        "SELECT published, document FROM entry_layout_versions WHERE world_id=?1 AND entry_id=?2
         AND revision<=?3 ORDER BY revision DESC LIMIT 1",
        params![view.world.0.as_bytes(),entry.id.0.as_bytes(),view.revision.0],
        |row| Ok((row.get(0)?,row.get(1)?))).optional()?;
    let Some((published, text)) = row else {
        return Ok(None);
    };
    if (!published && view.scope != Scope::Author)
        || resolve_object(conn, entry, view.branch, view.revision, view.day)?.is_none()
    {
        return Ok(None);
    }
    let doc: LayoutDocument = serde_json::from_str(&text)
        .map_err(|_| Error::Integrity("invalid stored entry layout".into()))?;
    if doc.entry != entry {
        return Err(Error::Integrity("layout entry mismatch".into()));
    }
    Ok(Some((doc, published)))
}

fn entry_search_term(text: &str, limit: usize) -> Result<String, Error> {
    let text = text.trim();
    if text.is_empty() || text.len() > 256 || !(1..=50).contains(&limit) {
        return Err(Error::InvalidInput("entry search text or limit"));
    }
    Ok(text.to_lowercase())
}

fn find_text_blocks(page: DisplayElement, needle: &str, limit: usize) -> EntryTextSearch {
    let mut result = EntryTextSearch {
        matches: Vec::new(),
        truncated: false,
    };
    let mut pending = vec![page];
    while let Some(element) = pending.pop() {
        match element.content {
            DisplayContent::Stack(children) => pending.extend(children.into_iter().rev()),
            DisplayContent::Text(text) if text.to_lowercase().contains(needle) => {
                if result.matches.len() == limit {
                    result.truncated = true;
                    break;
                }
                result.matches.push((element.id, text));
            }
            _ => {}
        }
    }
    result
}

// Derived documents never store author-only or character-only blocks.
// Replacing the previous document and changing publication share one transaction.
fn replace_public_page_index(
    conn: &Connection,
    document: &LayoutDocument,
    published: bool,
    revision: i64,
) -> Result<(), Error> {
    conn.execute(
        "DELETE FROM public_page_search_documents WHERE world_id=?1 AND entry_id=?2",
        params![
            document.entry.world.0.as_bytes(),
            document.entry.id.0.as_bytes()
        ],
    )?;
    if published && let Some(text) = document.public_index_text()? {
        conn.execute(
            "INSERT INTO public_page_search_documents(world_id,entry_id,revision,search_text)
             VALUES (?1,?2,?3,?4)",
            params![
                document.entry.world.0.as_bytes(),
                document.entry.id.0.as_bytes(),
                revision,
                text
            ],
        )?;
    }
    Ok(())
}

pub(super) fn rebuild_public_page_index(conn: &Connection) -> Result<(), Error> {
    conn.execute("DELETE FROM public_page_search_documents", [])?;
    let mut stmt = conn.prepare(
        "SELECT v.world_id,v.entry_id,v.revision,v.document
         FROM entry_layout_versions v WHERE v.published=1
         AND v.revision=(SELECT MAX(h.revision) FROM entry_layout_versions h
             WHERE h.world_id=v.world_id AND h.entry_id=v.entry_id)
         ORDER BY v.world_id,v.entry_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            Uuid::from_bytes(row.get(0)?),
            Uuid::from_bytes(row.get(1)?),
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (world, entry, revision, json) = row?;
        let document: LayoutDocument = serde_json::from_str(&json)
            .map_err(|_| Error::Integrity("invalid stored entry layout".into()))?;
        if document.entry.world.0 != world || document.entry.id.0 != entry {
            return Err(Error::Integrity("layout entry mismatch".into()));
        }
        replace_public_page_index(conn, &document, true, revision)?;
    }
    Ok(())
}

fn index_entry_links(
    conn: &Connection,
    document: &LayoutDocument,
    revision: i64,
) -> Result<(), Error> {
    document.validate()?;
    for (element, link) in document.visible_links(Scope::Author) {
        conn.execute(
            "INSERT INTO entry_link_index(world_id,entry_id,revision,element_id,target_id)
             VALUES (?1,?2,?3,?4,?5)",
            params![
                document.entry.world.0.as_bytes(),
                document.entry.id.0.as_bytes(),
                revision,
                element.0.as_bytes(),
                link.target.id.0.as_bytes()
            ],
        )?;
    }
    Ok(())
}

pub(super) fn rebuild_entry_link_index(conn: &Connection) -> Result<(), Error> {
    conn.execute("DELETE FROM entry_link_index", [])?;
    let mut stmt = conn.prepare(
        "SELECT world_id,entry_id,revision,document FROM entry_layout_versions
         ORDER BY world_id,entry_id,revision",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            Uuid::from_bytes(row.get(0)?),
            Uuid::from_bytes(row.get(1)?),
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (world, entry, revision, json) = row?;
        let document: LayoutDocument = serde_json::from_str(&json)
            .map_err(|_| Error::Integrity("invalid stored entry layout".into()))?;
        if document.entry.world.0 != world || document.entry.id.0 != entry {
            return Err(Error::Integrity("layout entry mismatch".into()));
        }
        index_entry_links(conn, &document, revision)?;
    }
    Ok(())
}
