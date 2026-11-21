//! Phase 6 dated conflict positions and deliberate cause/effect edges.
use std::collections::{HashMap, HashSet};

use crate::{
    Error, ObjectData, ObjectRef, Store, ValidInterval, ViewContext, WorldDay,
    genealogy::{active_on, validate_interval},
    graph::{acyclic, walk},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkStatus {
    Confirmed,
    Alleged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalLink {
    pub cause: ObjectRef,
    pub consequence: ObjectRef,
    pub source: ObjectRef,
    pub status: LinkStatus,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Impact {
    pub events: Vec<ObjectRef>,
    pub truncated: bool,
}

impl Store {
    /// Follow supplied explicit links from one authorized historical snapshot.
    pub fn confirmed_impact(
        &self,
        view: ViewContext,
        start: ObjectRef,
        links: &[CausalLink],
        max_depth: u8,
        max_visited: usize,
    ) -> Result<Impact, Error> {
        if view.scope != crate::Scope::Author {
            return Err(Error::CapabilityDenied);
        }
        if start.world != view.world
            || links.iter().any(|link| {
                [link.cause, link.consequence, link.source]
                    .iter()
                    .any(|object| object.world != view.world)
            })
        {
            return Err(Error::WrongWorld);
        }
        if links.len() > 4_000
            || !(1..=16).contains(&max_depth)
            || !(1..=2_000).contains(&max_visited)
        {
            return Err(Error::QueryBudgetExceeded);
        }
        let references = std::iter::once(start)
            .chain(
                links
                    .iter()
                    .filter(|link| link.status == LinkStatus::Confirmed)
                    .flat_map(|link| [link.cause, link.consequence, link.source]),
            )
            .collect();
        let snapshot = self.referenced_versions(view, references)?;
        let mut events = HashMap::new();
        let mut sources = HashSet::new();
        for item in snapshot {
            match item.record.data {
                ObjectData::HistoricalEvent { occurrence } => {
                    events.insert(item.object, occurrence.exact_day());
                }
                ObjectData::Source { .. } => {
                    sources.insert(item.object);
                }
                _ => {}
            }
        }
        if !events.contains_key(&start) {
            return Err(Error::NotFound);
        }
        let mut edges = HashMap::<ObjectRef, Vec<ObjectRef>>::new();
        for link in links
            .iter()
            .filter(|item| item.status == LinkStatus::Confirmed)
        {
            let (Some(Some(cause_day)), Some(Some(effect_day))) =
                (events.get(&link.cause), events.get(&link.consequence))
            else {
                continue;
            };
            if !sources.contains(&link.source) {
                continue;
            }
            if link.cause == link.consequence || cause_day > effect_day {
                return Err(Error::InvalidInput("invalid confirmed causal ordering"));
            }
            edges.entry(link.cause).or_default().push(link.consequence);
        }
        if !acyclic(&edges) {
            return Err(Error::InvalidInput("confirmed causal cycle"));
        }
        let (mut reached, truncated) = walk(start, &mut edges, max_depth, max_visited);
        reached.remove(&start);
        let mut reached: Vec<_> = reached.into_iter().collect();
        reached.sort_unstable_by_key(|object| (events[object], object.id.0));
        Ok(Impact {
            events: reached,
            truncated,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stance {
    Supports,
    Opposes,
    Mediates,
    Neutral,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConflictPosition {
    pub issue: ObjectRef,
    pub faction: ObjectRef,
    pub stance: Stance,
    pub valid: ValidInterval,
}

pub fn positions_at(
    issue: ObjectRef,
    day: WorldDay,
    records: &[ConflictPosition],
) -> Result<Vec<ConflictPosition>, Error> {
    if records.len() > 10_000 {
        return Err(Error::QueryBudgetExceeded);
    }
    let mut positions = Vec::new();
    let mut factions = HashSet::new();
    for item in records.iter().filter(|item| item.issue == issue) {
        if item.faction.world != issue.world {
            return Err(Error::WrongWorld);
        }
        validate_interval(item.valid)?;
        if active_on(item.valid, day) {
            if !factions.insert(item.faction) {
                return Err(Error::InvalidInput("overlapping active conflict positions"));
            }
            positions.push(*item);
        }
    }
    positions.sort_unstable_by_key(|item| item.faction.id.0);
    Ok(positions)
}
