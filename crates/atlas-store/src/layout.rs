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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId(pub Uuid);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutDocument {
    pub schema_version: u16,
    pub entry: ObjectRef,
    pub root: LayoutElement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutElement {
    pub id: ElementId,
    pub audience: ClaimAudience,
    pub content: ElementContent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementContent {
    Stack(Vec<LayoutElement>),
    Text(String),
    Query(QuerySpec),
}

#[derive(Debug, PartialEq, Eq)]
pub struct DisplayElement {
    pub id: ElementId,
    pub content: DisplayContent,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DisplayContent {
    Stack(Vec<DisplayElement>),
    Text(String),
    QueryRows {
        records: Vec<(ObjectRef, String)>,
        truncated: bool,
    },
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
