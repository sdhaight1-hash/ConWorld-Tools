//! Phase 5 dated, typed parentage over explicitly accepted records.
use std::collections::{HashMap, HashSet};

use crate::{
    Error, ObjectData, ObjectRef, Store, ValidInterval, ViewContext, WorldDay, WorldId,
    graph::acyclic,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ParentageKind {
    Biological,
    Adoptive,
    Legal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Parentage {
    pub parent: ObjectRef,
    pub child: ObjectRef,
    pub kind: ParentageKind,
    pub valid: ValidInterval,
    pub accepted: bool,
}

#[derive(Debug)]
pub struct Pedigree {
    world: WorldId,
    links: Vec<Parentage>,
}

pub(crate) fn validate_interval(interval: ValidInterval) -> Result<(), Error> {
    if matches!((interval.start, interval.end_exclusive), (Some(from), Some(until)) if from >= until)
    {
        Err(Error::InvalidInterval)
    } else {
        Ok(())
    }
}

pub(crate) fn active_on(interval: ValidInterval, day: WorldDay) -> bool {
    interval.start.is_none_or(|from| from <= day)
        && interval.end_exclusive.is_none_or(|until| day < until)
}

impl Pedigree {
    /// Build from a single world, branch and author revision supplied by the caller.
    pub fn at(world: WorldId, day: WorldDay, records: &[Parentage]) -> Result<Self, Error> {
        if records.len() > 10_000 {
            return Err(Error::QueryBudgetExceeded);
        }
        let mut links = Vec::new();
        let mut seen = HashSet::new();
        let mut edges = HashMap::<ObjectRef, Vec<ObjectRef>>::new();
        for record in records {
            if record.parent.world != world || record.child.world != world {
                return Err(Error::WrongWorld);
            }
            validate_interval(record.valid)?;
            if !record.accepted || !active_on(record.valid, day) {
                continue;
            }
            if record.parent == record.child {
                return Err(Error::InvalidInput("self parentage"));
            }
            if !seen.insert((record.child, record.parent, record.kind)) {
                return Err(Error::InvalidInput("duplicate accepted parentage"));
            }
            links.push(*record);
            if record.kind == ParentageKind::Biological {
                edges.entry(record.parent).or_default().push(record.child);
            }
        }
        if !acyclic(&edges) {
            return Err(Error::InvalidInput("accepted biological ancestry cycle"));
        }
        Ok(Self { world, links })
    }

    pub fn parents(&self, child: ObjectRef, kind: ParentageKind) -> Result<Vec<ObjectRef>, Error> {
        if child.world != self.world {
            return Err(Error::WrongWorld);
        }
        let mut parents: Vec<_> = self
            .links
            .iter()
            .filter(|item| item.child == child && item.kind == kind)
            .map(|item| item.parent)
            .collect();
        parents.sort_unstable_by_key(|item| item.id.0);
        Ok(parents)
    }
}

impl Store {
    /// Validate supplied parentage endpoints against the canonical author snapshot
    /// before using the resulting pedigree in another subsystem.
    pub fn pedigree_at(&self, view: ViewContext, records: &[Parentage]) -> Result<Pedigree, Error> {
        if view.scope != crate::Scope::Author {
            return Err(Error::CapabilityDenied);
        }
        let pedigree = Pedigree::at(view.world, view.day, records)?;
        let references = pedigree
            .links
            .iter()
            .flat_map(|link| [link.parent, link.child])
            .collect();
        let snapshot = self.referenced_versions(view, references)?;
        let people: HashSet<_> = snapshot
            .into_iter()
            .filter(|item| matches!(item.record.data, ObjectData::Person))
            .map(|item| item.object)
            .collect();
        if pedigree
            .links
            .iter()
            .any(|link| !people.contains(&link.parent) || !people.contains(&link.child))
        {
            return Err(Error::InvalidInput(
                "parentage endpoints must be visible people",
            ));
        }
        Ok(pedigree)
    }
}
