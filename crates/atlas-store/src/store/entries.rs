//! Main-branch authored layouts and explicit, revocable publication.
use super::*;
use crate::layout::{DisplayElement, LayoutDocument};

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
        tx.commit()?;
        Ok(AuthorRevision(revision))
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
        let Some((document, _)) = self.read_layout(view, entry)? else {
            return Ok(None);
        };
        document.compile(self, view)
    }

    fn read_layout(
        &self,
        view: ViewContext,
        entry: ObjectRef,
    ) -> Result<Option<(LayoutDocument, bool)>, Error> {
        if entry.world != view.world {
            return Err(Error::WrongWorld);
        }
        let tx = self.conn.unchecked_transaction()?;
        let world = load_world(&tx, view.world)?;
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
                &tx,
                view.world,
                reader,
                view.branch,
                view.revision.0,
                view.day,
            )?;
        }
        let row: Option<(bool, String)> = tx.query_row(
            "SELECT published, document FROM entry_layout_versions WHERE world_id=?1 AND entry_id=?2
             AND revision<=?3 ORDER BY revision DESC LIMIT 1",
            params![view.world.0.as_bytes(),entry.id.0.as_bytes(),view.revision.0],
            |row| Ok((row.get(0)?,row.get(1)?))).optional()?;
        let Some((published, text)) = row else {
            return Ok(None);
        };
        if (!published && view.scope != Scope::Author)
            || resolve_object(&tx, entry, view.branch, view.revision, view.day)?.is_none()
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
}
