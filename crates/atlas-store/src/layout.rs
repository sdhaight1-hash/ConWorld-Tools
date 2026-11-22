//! Phase 4 layout validation, stable edits and audience-filtered compilation.
use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    AuthorRevision, ClaimAudience, Error, ObjectRef, Scope, Store, ViewContext, WorldId,
    access::{can_view, restrict},
    query::{ParameterBinding, QuerySpec},
};

pub const LAYOUT_SCHEMA_VERSION: u16 = 1;
const MAX_ELEMENTS: usize = 512;
const MAX_DEPTH: usize = 16;
const MAX_QUERIES: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ElementId(pub Uuid);

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayoutDocument {
    pub schema_version: u16,
    pub entry: ObjectRef,
    pub root: LayoutElement,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayoutElement {
    pub id: ElementId,
    pub audience: ClaimAudience,
    pub content: ElementContent,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ElementContent {
    Stack(Vec<LayoutElement>),
    Panel(Vec<LayoutElement>),
    Grid {
        columns: u8,
        children: Vec<LayoutElement>,
    },
    Template {
        key: TemplateKind,
        version: u16,
        children: Vec<LayoutElement>,
    },
    Text(String),
    Query(QuerySpec),
    SavedQuery(SavedQueryBinding),
    FamilyTable(ObjectRef),
    NetworkTable(ObjectRef),
    ImpactTable(ObjectRef),
    PublishedReferences(ObjectRef),
    ObjectLink(ObjectLink),
}

/// Built-in immutable starting layouts. Instantiation copies presentation only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateKind {
    Blank,
    Character,
    Family,
    Organization,
    Event,
    Conflict,
    Location,
    Genealogy,
    Source,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectLink {
    pub target: ObjectRef,
    pub label: String,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntryBacklink {
    pub source: ObjectRef,
    pub element: ElementId,
    pub label: String,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntryBacklinks {
    pub matches: Vec<EntryBacklink>,
    pub truncated: bool,
}

/// Pin the definition, not its results. Runtime data follows the layout's view.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedQueryBinding {
    pub world: WorldId,
    pub query_id: Uuid,
    pub definition_revision: i64,
    pub bindings: Vec<ParameterBinding>,
}

impl SavedQueryBinding {
    pub fn resolve(&self, store: &Store, view: ViewContext) -> Result<QuerySpec, Error> {
        if view.scope != Scope::Author {
            return Err(Error::CapabilityDenied);
        }
        if self.world != view.world {
            return Err(Error::WrongWorld);
        }
        let pinned = AuthorRevision(self.definition_revision);
        if pinned.0 <= 0 || pinned > view.revision {
            return Err(Error::InvalidRevision);
        }
        let saved = store
            .saved_label_query(self.world, pinned, view.scope, self.query_id)?
            .ok_or(Error::NotFound)?;
        if saved.recorded_at != pinned {
            return Err(Error::InvalidRevision);
        }
        saved.bind(view, &self.bindings)
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DisplayElement {
    pub id: ElementId,
    pub content: DisplayContent,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DisplayContent {
    Stack(Vec<DisplayElement>),
    Panel(Vec<DisplayElement>),
    Grid {
        columns: u8,
        children: Vec<DisplayElement>,
    },
    Text(String),
    ObjectLink(ObjectLink),
    ReferenceRows {
        links: Vec<ObjectLink>,
        truncated: bool,
    },
    QueryRows {
        records: Vec<(ObjectRef, String)>,
        truncated: bool,
    },
}

/// Matches are drawn only from the compiled, audience-authorized page.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntryTextSearch {
    pub matches: Vec<(ElementId, String)>,
    pub truncated: bool,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PublicPageMatch {
    pub entry: ObjectRef,
    pub element: ElementId,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PublicPageSearch {
    pub matches: Vec<PublicPageMatch>,
    pub truncated: bool,
}

impl LayoutDocument {
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema_version != LAYOUT_SCHEMA_VERSION {
            return Err(Error::InvalidInput("layout schema"));
        }
        if !matches!(self.root.content, ElementContent::Stack(_)) {
            return Err(Error::InvalidInput("layout root must be a stack"));
        }
        let mut visited = HashSet::new();
        let mut pending = vec![(&self.root, 0)];
        while let Some((element, depth)) = pending.pop() {
            if depth > MAX_DEPTH || visited.len() == MAX_ELEMENTS {
                return Err(Error::QueryBudgetExceeded);
            }
            if !visited.insert(element.id) {
                return Err(Error::InvalidInput("duplicate layout element ID"));
            }
            if let ClaimAudience::Character(reader) = element.audience
                && reader.world != self.entry.world
            {
                return Err(Error::WrongWorld);
            }
            match &element.content {
                ElementContent::Template { version, .. } if *version != 1 => {
                    return Err(Error::InvalidInput("layout template version"));
                }
                ElementContent::Grid { columns, .. } if !(1..=4).contains(columns) => {
                    return Err(Error::InvalidInput("layout grid columns"));
                }
                ElementContent::Stack(children)
                | ElementContent::Panel(children)
                | ElementContent::Template { children, .. }
                | ElementContent::Grid { children, .. } => {
                    if children.len() + pending.len() + visited.len() > MAX_ELEMENTS {
                        return Err(Error::QueryBudgetExceeded);
                    }
                    pending.extend(children.iter().map(|child| (child, depth + 1)));
                }
                ElementContent::Text(text) if text.len() > 16_384 => {
                    return Err(Error::InvalidInput("layout text length"));
                }
                ElementContent::ObjectLink(link) => {
                    if link.target.world != self.entry.world {
                        return Err(Error::WrongWorld);
                    }
                    if link.label.trim().is_empty() || link.label.len() > 800 {
                        return Err(Error::InvalidInput("layout link label"));
                    }
                }
                ElementContent::FamilyTable(person)
                | ElementContent::NetworkTable(person)
                | ElementContent::ImpactTable(person)
                | ElementContent::PublishedReferences(person) => {
                    if person.world != self.entry.world {
                        return Err(Error::WrongWorld);
                    }
                }
                ElementContent::SavedQuery(binding) => {
                    if binding.world != self.entry.world {
                        return Err(Error::WrongWorld);
                    }
                    if binding.definition_revision <= 0 || binding.bindings.len() > 32 {
                        return Err(Error::InvalidInput("saved layout binding"));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// The query engine currently returns author records. Such bindings can
    /// live in private subtrees but cannot be released to any reader.
    pub(crate) fn validate_publication(&self) -> Result<(), Error> {
        let mut pending = vec![(&self.root, ClaimAudience::Public)];
        while let Some((element, parent)) = pending.pop() {
            let audience = restrict(element.audience, parent);
            if audience == ClaimAudience::Author {
                continue;
            }
            match &element.content {
                ElementContent::Query(_)
                | ElementContent::SavedQuery(_)
                | ElementContent::FamilyTable(_)
                | ElementContent::NetworkTable(_)
                | ElementContent::ImpactTable(_) => {
                    return Err(Error::CapabilityDenied);
                }
                ElementContent::Stack(children)
                | ElementContent::Panel(children)
                | ElementContent::Template { children, .. }
                | ElementContent::Grid { children, .. } => {
                    pending.extend(children.iter().map(|child| (child, audience)))
                }
                ElementContent::Text(_)
                | ElementContent::ObjectLink(_)
                | ElementContent::PublishedReferences(_) => {}
            }
        }
        Ok(())
    }

    /// Use the same public subtree gate and literal text as the Display projection.
    /// This is only a conservative candidate index; canonical reads still authorize results.
    pub(crate) fn public_index_text(&self) -> Result<Option<String>, Error> {
        self.validate()?;
        self.validate_publication()?;
        if !can_view(Scope::Public, self.root.audience) {
            return Ok(None);
        }
        let mut text = String::new();
        let mut pending = vec![&self.root];
        while let Some(element) = pending.pop() {
            if !can_view(Scope::Public, element.audience) {
                continue;
            }
            match &element.content {
                ElementContent::Stack(children)
                | ElementContent::Panel(children)
                | ElementContent::Template { children, .. }
                | ElementContent::Grid { children, .. } => pending.extend(children.iter().rev()),
                ElementContent::Text(block) => {
                    // SQLite tokenization may stop at NUL; normalize it only in
                    // candidates. Queries containing NUL use the exact fallback.
                    text.push_str(&block.to_lowercase().replace('\0', " "));
                    text.push('\n');
                }
                // Destination-dependent captions are excluded from static text search.
                ElementContent::ObjectLink(_) | ElementContent::PublishedReferences(_) => {}
                ElementContent::Query(_)
                | ElementContent::SavedQuery(_)
                | ElementContent::FamilyTable(_)
                | ElementContent::NetworkTable(_)
                | ElementContent::ImpactTable(_) => {
                    return Err(Error::CapabilityDenied);
                }
            }
        }
        Ok(Some(text))
    }

    pub(crate) fn visible_links(&self, scope: Scope) -> Vec<(ElementId, &ObjectLink)> {
        let mut links = Vec::new();
        let mut pending = vec![&self.root];
        while let Some(element) = pending.pop() {
            if !can_view(scope, element.audience) {
                continue;
            }
            match &element.content {
                ElementContent::Stack(children)
                | ElementContent::Panel(children)
                | ElementContent::Template { children, .. }
                | ElementContent::Grid { children, .. } => pending.extend(children.iter().rev()),
                ElementContent::ObjectLink(link) => links.push((element.id, link)),
                _ => {}
            }
        }
        links
    }

    pub fn reparent(&mut self, id: ElementId, target: ElementId) -> Result<(), Error> {
        self.validate()?;
        let mut updated = self.clone();
        let moved =
            take_child(&mut updated.root, id, ClaimAudience::Public).ok_or(Error::NotFound)?;
        let destination = find_container(&mut updated.root, target).ok_or(Error::NotFound)?;
        destination.push(moved);
        updated.validate()?;
        *self = updated;
        Ok(())
    }

    pub fn compile(
        &self,
        store: &Store,
        view: ViewContext,
    ) -> Result<Option<DisplayElement>, Error> {
        if self.entry.world != view.world {
            return Err(Error::WrongWorld);
        }
        if !can_view(view.scope, self.root.audience) {
            return Ok(None);
        }
        if view.scope == Scope::Author {
            self.validate()?;
        }
        if self.schema_version != LAYOUT_SCHEMA_VERSION
            || !matches!(self.root.content, ElementContent::Stack(_))
        {
            return Err(Error::InvalidInput("layout schema or root"));
        }
        project(&self.root, store, view, 0, &mut ProjectionBudget::default())
    }
}

fn take_child(
    element: &mut LayoutElement,
    id: ElementId,
    enclosing: ClaimAudience,
) -> Option<LayoutElement> {
    let inherited = restrict(element.audience, enclosing);
    let children = match &mut element.content {
        ElementContent::Stack(children)
        | ElementContent::Panel(children)
        | ElementContent::Template { children, .. }
        | ElementContent::Grid { children, .. } => children,
        _ => return None,
    };
    if let Some(index) = children.iter().position(|child| child.id == id) {
        let mut moved = children.remove(index);
        moved.audience = restrict(moved.audience, inherited);
        return Some(moved);
    }
    children
        .iter_mut()
        .find_map(|child| take_child(child, id, inherited))
}

fn find_container(element: &mut LayoutElement, id: ElementId) -> Option<&mut Vec<LayoutElement>> {
    let children = match &mut element.content {
        ElementContent::Stack(children)
        | ElementContent::Panel(children)
        | ElementContent::Template { children, .. }
        | ElementContent::Grid { children, .. } => children,
        _ => return None,
    };
    if element.id == id {
        Some(children)
    } else {
        children
            .iter_mut()
            .find_map(|child| find_container(child, id))
    }
}

#[derive(Default)]
struct ProjectionBudget {
    seen: HashSet<ElementId>,
    queries: usize,
}

fn project(
    element: &LayoutElement,
    store: &Store,
    view: ViewContext,
    depth: usize,
    budget: &mut ProjectionBudget,
) -> Result<Option<DisplayElement>, Error> {
    if !can_view(view.scope, element.audience) {
        return Ok(None);
    }
    if depth > MAX_DEPTH || budget.seen.len() == MAX_ELEMENTS {
        return Err(Error::QueryBudgetExceeded);
    }
    if !budget.seen.insert(element.id) {
        return Err(Error::InvalidInput("duplicate layout element ID"));
    }
    let content = match &element.content {
        ElementContent::Stack(children)
        | ElementContent::Panel(children)
        | ElementContent::Template { children, .. }
        | ElementContent::Grid { children, .. } => {
            let mut projected = Vec::new();
            for child in children {
                if let Some(child) = project(child, store, view, depth + 1, budget)? {
                    projected.push(child);
                }
            }
            match &element.content {
                ElementContent::Panel(_) => DisplayContent::Panel(projected),
                ElementContent::Template { version, .. } => {
                    if *version != 1 {
                        return Err(Error::InvalidInput("layout template version"));
                    }
                    DisplayContent::Panel(projected)
                }
                ElementContent::Grid { columns, .. } => {
                    if !(1..=4).contains(columns) {
                        return Err(Error::InvalidInput("layout grid columns"));
                    }
                    DisplayContent::Grid {
                        columns: *columns,
                        children: projected,
                    }
                }
                _ => DisplayContent::Stack(projected),
            }
        }
        ElementContent::Text(text) => {
            if text.len() > 16_384 {
                return Err(Error::InvalidInput("layout text length"));
            }
            DisplayContent::Text(text.clone())
        }
        ElementContent::ObjectLink(link) => {
            if !store.entry_link_target_visible(view, link.target)? {
                return Ok(None);
            }
            DisplayContent::ObjectLink(link.clone())
        }
        ElementContent::PublishedReferences(target) => {
            if budget.queries == MAX_QUERIES {
                return Err(Error::QueryBudgetExceeded);
            }
            budget.queries += 1;
            let references = store.published_references(view, *target)?;
            DisplayContent::ReferenceRows {
                links: references
                    .matches
                    .into_iter()
                    .map(|item| ObjectLink {
                        target: item.source,
                        label: item.label,
                    })
                    .collect(),
                truncated: references.truncated,
            }
        }
        ElementContent::Query(_)
        | ElementContent::SavedQuery(_)
        | ElementContent::FamilyTable(_)
        | ElementContent::NetworkTable(_)
        | ElementContent::ImpactTable(_) => {
            if view.scope != Scope::Author {
                return Err(Error::CapabilityDenied);
            }
            if budget.queries == MAX_QUERIES {
                return Err(Error::QueryBudgetExceeded);
            }
            budget.queries += 1;
            if let ElementContent::ImpactTable(start) = &element.content {
                let impact = store.persisted_impact(view, *start, 8, 200)?;
                let truncated = impact.truncated || impact.events.len() > 50;
                let mut records = Vec::new();
                for object in impact.events.into_iter().take(50) {
                    if let Some(version) = store.get_object(object, view)? {
                        records.push((object, version.record.label));
                    }
                }
                return Ok(Some(DisplayElement {
                    id: element.id,
                    content: DisplayContent::QueryRows { records, truncated },
                }));
            }
            if let ElementContent::NetworkTable(focus) = &element.content {
                let facts = store.network_facts(view, *focus)?;
                let truncated = facts.len() > 50;
                let records = facts
                    .into_iter()
                    .take(50)
                    .map(|fact| {
                        let other = if fact.input.subject == *focus {
                            fact.input.target
                        } else {
                            fact.input.subject
                        };
                        (
                            other,
                            format!(
                                "{} → {} · {} · {}",
                                fact.subject_label,
                                fact.target_label,
                                fact.input.kind.name(),
                                fact.input.role
                            ),
                        )
                    })
                    .collect();
                return Ok(Some(DisplayElement {
                    id: element.id,
                    content: DisplayContent::QueryRows { records, truncated },
                }));
            }
            if let ElementContent::FamilyTable(person) = &element.content {
                let links = store.family_links(view, *person)?;
                let truncated = links.len() > 50;
                let records = links
                    .into_iter()
                    .take(50)
                    .map(|link| {
                        let (relative, direction) = if link.parent == *person {
                            (link.child, "child")
                        } else {
                            (link.parent, "parent")
                        };
                        let kind = match link.kind {
                            crate::genealogy::ParentageKind::Biological => "biological",
                            crate::genealogy::ParentageKind::Adoptive => "adoptive",
                            crate::genealogy::ParentageKind::Legal => "legal",
                        };
                        (
                            relative,
                            format!("{} · {kind} {direction}", link.relative_label),
                        )
                    })
                    .collect();
                return Ok(Some(DisplayElement {
                    id: element.id,
                    content: DisplayContent::QueryRows { records, truncated },
                }));
            }
            let result = match &element.content {
                ElementContent::Query(query) => store.execute_query(query, view)?,
                ElementContent::SavedQuery(binding) => {
                    store.execute_query(&binding.resolve(store, view)?, view)?
                }
                _ => unreachable!("query component"),
            };
            DisplayContent::QueryRows {
                records: result
                    .rows
                    .into_iter()
                    .map(|item| (item.object, item.record.label))
                    .collect(),
                truncated: result.truncated,
            }
        }
    };
    Ok(Some(DisplayElement {
        id: element.id,
        content,
    }))
}
