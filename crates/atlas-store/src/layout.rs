//! Phase 4 layout validation, stable edits and audience-filtered compilation.
use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    ClaimAudience, Error, ObjectRef, Scope, Store, ViewContext,
    access::{can_view, restrict},
    query::QuerySpec,
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
    Text(String),
    Query(QuerySpec),
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DisplayElement {
    pub id: ElementId,
    pub content: DisplayContent,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DisplayContent {
    Stack(Vec<DisplayElement>),
    Text(String),
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
                ElementContent::Stack(children) => {
                    if children.len() + pending.len() + visited.len() > MAX_ELEMENTS {
                        return Err(Error::QueryBudgetExceeded);
                    }
                    pending.extend(children.iter().map(|child| (child, depth + 1)));
                }
                ElementContent::Text(text) if text.len() > 16_384 => {
                    return Err(Error::InvalidInput("layout text length"));
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
                ElementContent::Query(_) => return Err(Error::CapabilityDenied),
                ElementContent::Stack(children) => {
                    pending.extend(children.iter().map(|child| (child, audience)))
                }
                ElementContent::Text(_) => {}
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
                ElementContent::Stack(children) => pending.extend(children.iter().rev()),
                ElementContent::Text(block) => {
                    // SQLite tokenization may stop at NUL; normalize it only in
                    // candidates. Queries containing NUL use the exact fallback.
                    text.push_str(&block.to_lowercase().replace('\0', " "));
                    text.push('\n');
                }
                ElementContent::Query(_) => return Err(Error::CapabilityDenied),
            }
        }
        Ok(Some(text))
    }

    pub fn reparent(&mut self, id: ElementId, target: ElementId) -> Result<(), Error> {
        self.validate()?;
        let mut updated = self.clone();
        let moved =
            take_child(&mut updated.root, id, ClaimAudience::Public).ok_or(Error::NotFound)?;
        let destination = find_stack(&mut updated.root, target).ok_or(Error::NotFound)?;
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
    let ElementContent::Stack(children) = &mut element.content else {
        return None;
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

fn find_stack(element: &mut LayoutElement, id: ElementId) -> Option<&mut Vec<LayoutElement>> {
    let ElementContent::Stack(children) = &mut element.content else {
        return None;
    };
    if element.id == id {
        Some(children)
    } else {
        children.iter_mut().find_map(|child| find_stack(child, id))
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
        ElementContent::Stack(children) => {
            let mut projected = Vec::new();
            for child in children {
                if let Some(child) = project(child, store, view, depth + 1, budget)? {
                    projected.push(child);
                }
            }
            DisplayContent::Stack(projected)
        }
        ElementContent::Text(text) => {
            if text.len() > 16_384 {
                return Err(Error::InvalidInput("layout text length"));
            }
            DisplayContent::Text(text.clone())
        }
        ElementContent::Query(query) => {
            if view.scope != Scope::Author {
                return Err(Error::CapabilityDenied);
            }
            if budget.queries == MAX_QUERIES {
                return Err(Error::QueryBudgetExceeded);
            }
            budget.queries += 1;
            let result = store.execute_query(query, view)?;
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
