use std::collections::{HashMap, HashSet, VecDeque};

use crate::ObjectRef;

/// The starting node counts against the visit budget. Return partial results only
/// when an additional eligible node would exceed that budget.
pub(crate) fn walk(
    start: ObjectRef,
    edges: &mut HashMap<ObjectRef, Vec<ObjectRef>>,
    depth: u8,
    budget: usize,
) -> (HashSet<ObjectRef>, bool) {
    for adjacent in edges.values_mut() {
        adjacent.sort_unstable_by_key(|item| item.id.0);
    }
    let mut visited = HashSet::from([start]);
    let mut pending = VecDeque::from([(start, 0)]);
    while let Some((node, level)) = pending.pop_front() {
        if level == depth {
            continue;
        }
        for next in edges.get(&node).into_iter().flatten().copied() {
            if !visited.contains(&next) {
                if visited.len() == budget {
                    return (visited, true);
                }
                visited.insert(next);
                pending.push_back((next, level + 1));
            }
        }
    }
    (visited, false)
}

pub(crate) fn acyclic(edges: &HashMap<ObjectRef, Vec<ObjectRef>>) -> bool {
    let mut indegree = HashMap::<ObjectRef, usize>::new();
    for (origin, destinations) in edges {
        indegree.entry(*origin).or_default();
        for destination in destinations {
            *indegree.entry(*destination).or_default() += 1;
        }
    }
    let mut ready: VecDeque<_> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(object, _)| *object)
        .collect();
    let mut visited = 0;
    while let Some(origin) = ready.pop_front() {
        visited += 1;
        for destination in edges.get(&origin).into_iter().flatten() {
            let degree = indegree
                .get_mut(destination)
                .expect("endpoint has an indegree");
            *degree -= 1;
            if *degree == 0 {
                ready.push_back(*destination);
            }
        }
    }
    visited == indegree.len()
}
