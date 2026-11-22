use super::*;
use crate::layout::{
    CustomTemplateSummary, ElementContent, ElementId, LayoutDocument, LayoutElement,
};

fn template_world(conn: &Connection, view: ViewContext) -> Result<World, Error> {
    require_author(view.scope)?;
    let world = load_world(conn, view.world)?;
    if world.main != view.branch {
        return Err(Error::CapabilityDenied);
    }
    if view.revision.0 < 0 || view.revision > world.revision {
        return Err(Error::InvalidRevision);
    }
    Ok(world)
}

impl Store {
    /// Snapshot a saved presentation, never a canonical object or evaluated query rows.
    pub fn save_custom_template(
        &mut self,
        view: ViewContext,
        template_id: Option<Uuid>,
        source: ObjectRef,
        name: &str,
        command_id: Uuid,
    ) -> Result<CustomTemplateSummary, Error> {
        template_world(&self.conn, view)?;
        if name.trim().is_empty() || name.len() > 200 {
            return Err(Error::InvalidInput("template name"));
        }
        let (document, _) = self.entry_layout(view, source)?.ok_or(Error::NotFound)?;
        document.validate()?;
        let json = serde_json::to_string(&document)
            .map_err(|_| Error::InvalidInput("template encoding"))?;
        if json.len() > 1_048_576 {
            return Err(Error::QueryBudgetExceeded);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = template_world(&tx, view)?;
        let id = template_id.unwrap_or_else(Uuid::new_v4);
        if template_id.is_some() {
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM custom_canvas_templates WHERE world_id=?1 AND id=?2 AND revision<=?3)",
                params![view.world.0.as_bytes(), id.as_bytes(), view.revision.0], |row| row.get(0))?;
            if !exists {
                return Err(Error::NotFound);
            }
        }
        let revision = append_revision(
            &tx,
            &world,
            world.main,
            view.revision,
            command_id,
            ("Local author", "Save custom Canvas template"),
        )?;
        tx.execute("INSERT INTO custom_canvas_templates(world_id,id,revision,name,source_id,document) VALUES(?1,?2,?3,?4,?5,?6)",
            params![view.world.0.as_bytes(),id.as_bytes(),revision,name.trim(),source.id.0.as_bytes(),json])?;
        tx.commit()?;
        Ok(CustomTemplateSummary {
            id,
            name: name.trim().into(),
            source,
            revision,
        })
    }

    pub fn custom_templates(&self, view: ViewContext) -> Result<Vec<CustomTemplateSummary>, Error> {
        template_world(&self.conn, view)?;
        let mut stmt = self.conn.prepare(
            "SELECT id,name,source_id,revision FROM custom_canvas_templates t WHERE world_id=?1
             AND revision=(SELECT MAX(revision) FROM custom_canvas_templates v
                 WHERE v.world_id=t.world_id AND v.id=t.id AND v.revision<=?2)
             ORDER BY name,id LIMIT 201",
        )?;
        let mut rows = stmt.query(params![view.world.0.as_bytes(), view.revision.0])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            if result.len() == 200 {
                return Err(Error::QueryBudgetExceeded);
            }
            result.push(CustomTemplateSummary {
                id: Uuid::from_bytes(row.get(0)?),
                name: row.get(1)?,
                source: ObjectRef {
                    world: view.world,
                    id: ObjectId(Uuid::from_bytes(row.get(2)?)),
                },
                revision: row.get(3)?,
            });
        }
        Ok(result)
    }

    pub(crate) fn require_custom_template(
        &self,
        view: ViewContext,
        id: Uuid,
        revision: i64,
    ) -> Result<(), Error> {
        template_world(&self.conn, view)?;
        if revision <= 0 || revision > view.revision.0 {
            return Err(Error::InvalidRevision);
        }
        let exists: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM custom_canvas_templates WHERE world_id=?1 AND id=?2 AND revision=?3)",
            params![view.world.0.as_bytes(),id.as_bytes(),revision],|row|row.get(0))?;
        if !exists {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    pub fn instantiate_custom_template(
        &self,
        view: ViewContext,
        id: Uuid,
        revision: i64,
        destination: ObjectRef,
    ) -> Result<LayoutElement, Error> {
        self.require_custom_template(view, id, revision)?;
        if self.get_object(destination, view)?.is_none() {
            return Err(Error::NotFound);
        }
        let json:String=self.conn.query_row("SELECT document FROM custom_canvas_templates WHERE world_id=?1 AND id=?2 AND revision=?3",
            params![view.world.0.as_bytes(),id.as_bytes(),revision],|row|row.get(0))?;
        let document: LayoutDocument = serde_json::from_str(&json)
            .map_err(|_| Error::Integrity("invalid stored template".into()))?;
        document.validate()?;
        if document.entry.world != view.world {
            return Err(Error::WrongWorld);
        }
        let source = document.entry;
        let mut root = document.root;
        fresh_elements(&mut root, source, destination);
        let instance = LayoutElement {
            id: ElementId(Uuid::new_v4()),
            audience: ClaimAudience::Author,
            content: ElementContent::CustomTemplate {
                template_id: id,
                definition_revision: revision,
                children: vec![root],
            },
        };
        let check = LayoutDocument {
            schema_version: crate::layout::LAYOUT_SCHEMA_VERSION,
            entry: destination,
            root: LayoutElement {
                id: ElementId(Uuid::new_v4()),
                audience: ClaimAudience::Public,
                content: ElementContent::Stack(vec![instance.clone()]),
            },
        };
        check.validate()?;
        Ok(instance)
    }
}

fn fresh_elements(element: &mut LayoutElement, source: ObjectRef, destination: ObjectRef) {
    element.id = ElementId(Uuid::new_v4());
    let rebind = |target: &mut ObjectRef| {
        if *target == source {
            *target = destination;
        }
    };
    match &mut element.content {
        ElementContent::Stack(children)
        | ElementContent::Panel(children)
        | ElementContent::Grid { children, .. }
        | ElementContent::Template { children, .. }
        | ElementContent::CustomTemplate { children, .. } => {
            for child in children {
                fresh_elements(child, source, destination);
            }
        }
        ElementContent::Image(image) => {
            for hotspot in &mut image.hotspots { hotspot.id = ElementId(Uuid::new_v4()); }
        }
        ElementContent::FamilyTable(target)
        | ElementContent::NetworkTable(target)
        | ElementContent::ImpactTable(target)
        | ElementContent::PublishedReferences(target)
        | ElementContent::PublishedStatements(target) => rebind(target),
        ElementContent::Query(query) => match &mut query.filter {
            QueryFilter::ParticipantEvents { participant, .. } => rebind(participant),
            QueryFilter::EventRole { event, .. } => rebind(event),
            QueryFilter::RelationshipPath { start, .. } => rebind(start),
            QueryFilter::Backlinks(target) => rebind(target),
            _ => {}
        },
        _ => {}
    }
}
