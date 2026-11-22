//! Phase 8 reproducible fictional ancestry intervals. No nucleotide or genome data.
use crate::{
    Error, ObjectRef,
    genealogy::{ParentageKind, Pedigree},
};

pub const MODEL_VERSION: u16 = 1;
pub const MODEL_LENGTH: u32 = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SyntheticSegment {
    pub start: u32,
    pub end: u32,
    pub contributor: ObjectRef,
}

/// A versioned toy crossover over two accepted biological contributor references.
/// Segment coordinates are dimensionless and carry no medical interpretation.
pub fn fictional_segments(
    pedigree: &Pedigree,
    child: ObjectRef,
    seed: u64,
    model: u16,
) -> Result<[SyntheticSegment; 2], Error> {
    if model != MODEL_VERSION {
        return Err(Error::InvalidInput("synthetic model version"));
    }
    let parents = pedigree.parents(child, ParentageKind::Biological)?;
    let [first, second] = parents.as_slice() else {
        return Err(Error::InvalidInput(
            "synthetic model requires two accepted biological contributors",
        ));
    };
    // Fixed SplitMix64 mixing and little-endian UUID halves keep outputs reproducible
    // across toolchain releases and operating systems.
    let mut mixed = mix(seed ^ u64::from(model));
    for id in [child, *first, *second] {
        let number = id.id.0.as_u128();
        mixed = mix(mixed ^ number as u64);
        mixed = mix(mixed ^ (number >> 64) as u64);
    }
    let crossover = 1 + (mixed % (u64::from(MODEL_LENGTH) - 1)) as u32;
    let (left, right) = if mixed & (1 << 63) == 0 {
        (*first, *second)
    } else {
        (*second, *first)
    };
    Ok([
        SyntheticSegment {
            start: 0,
            end: crossover,
            contributor: left,
        },
        SyntheticSegment {
            start: crossover,
            end: MODEL_LENGTH,
            contributor: right,
        },
    ])
}

fn mix(value: u64) -> u64 {
    let mut value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

/// Shared dimensionless intervals from the same immediate fictional contributor.
/// Distinct people can have zero overlap even with a shared biological parent.
pub fn shared_segments(
    pedigree: &Pedigree,
    first: ObjectRef,
    second: ObjectRef,
    seed: u64,
    model: u16,
) -> Result<Vec<SyntheticSegment>, Error> {
    if first == second {
        return Err(Error::InvalidInput("select two distinct people"));
    }
    let left = fictional_segments(pedigree, first, seed, model)?;
    let right = fictional_segments(pedigree, second, seed, model)?;
    let mut shared = Vec::new();
    for a in left {
        for b in right {
            let start = a.start.max(b.start);
            let end = a.end.min(b.end);
            if a.contributor == b.contributor && start < end {
                shared.push(SyntheticSegment {
                    start,
                    end,
                    contributor: a.contributor,
                });
            }
        }
    }
    Ok(shared)
}

impl crate::Store {
    /// Compare accepted biological parents from the same historical author view.
    pub fn family_match_at(
        &self,
        view: crate::ViewContext,
        first: ObjectRef,
        second: ObjectRef,
        seed: u64,
    ) -> Result<Vec<SyntheticSegment>, Error> {
        if first == second {
            return Err(Error::InvalidInput("select two distinct people"));
        }
        let mut parents = Vec::new();
        for child in [first, second] {
            for link in self.family_links(view, child)? {
                if link.child == child && link.kind == ParentageKind::Biological {
                    parents.push(crate::genealogy::Parentage {
                        parent: link.parent,
                        child,
                        kind: link.kind,
                        accepted: true,
                        valid: crate::ValidInterval {
                            start: Some(crate::WorldDay(link.starts_at)),
                            end_exclusive: link.ends_before.map(crate::WorldDay),
                        },
                    });
                }
            }
        }
        let pedigree = Pedigree::at(view.world, view.day, &parents)?;
        shared_segments(&pedigree, first, second, seed, MODEL_VERSION)
    }
}

/// Version 2 toy autosomal model: two dimensionless homologues, founder allele
/// identity rather than actual DNA. No medical or real population inference.
pub const FOUNDER_MODEL_VERSION: u16 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct FounderAllele {
    pub founder: ObjectRef,
    pub homologue: u8,
    pub unresolved_parent: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct FounderSegment {
    pub start: u32,
    pub end: u32,
    pub allele: FounderAllele,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct FounderGenome {
    pub model: u16,
    pub homologues: [Vec<FounderSegment>; 2],
}

impl crate::Store {
    /// Resolve accepted biological ancestors at exactly one author view. A
    /// missing parent becomes a distinct unresolved founder; no legal link is used.
    pub fn founder_genome(
        &self,
        view: crate::ViewContext,
        subject: ObjectRef,
        seed: u64,
    ) -> Result<FounderGenome, Error> {
        use std::collections::{HashMap, HashSet};
        fn visit(
            store: &crate::Store,
            view: crate::ViewContext,
            subject: ObjectRef,
            seed: u64,
            depth: u8,
            active: &mut HashSet<ObjectRef>,
            memo: &mut HashMap<ObjectRef, FounderGenome>,
        ) -> Result<FounderGenome, Error> {
            if let Some(value) = memo.get(&subject) {
                return Ok(value.clone());
            }
            if depth > 16 || memo.len() + active.len() >= 512 {
                return Err(Error::QueryBudgetExceeded);
            }
            if !active.insert(subject) {
                return Err(Error::InvalidInput("accepted biological ancestry cycle"));
            }
            let mut parents: Vec<_> = store
                .family_links(view, subject)?
                .into_iter()
                .filter(|link| link.child == subject && link.kind == ParentageKind::Biological)
                .map(|link| link.parent)
                .collect();
            parents.sort_unstable_by_key(|item| item.id.0);
            if parents.len() > 2 {
                return Err(Error::InvalidInput(
                    "more than two accepted biological parents",
                ));
            }
            let mut homologues = [Vec::new(), Vec::new()];
            for (slot, segments) in homologues.iter_mut().enumerate() {
                if let Some(parent) = parents.get(slot) {
                    let genome = visit(store, view, *parent, seed, depth + 1, active, memo)?;
                    let id = subject.id.0.as_u128();
                    let random = mix(seed ^ id as u64 ^ mix((id >> 64) as u64) ^ slot as u64);
                    let boundary = 1 + (random % u64::from(MODEL_LENGTH - 1)) as u32;
                    let first = (random >> 63) as usize;
                    for (index, lo, hi) in
                        [(first, 0, boundary), (1 - first, boundary, MODEL_LENGTH)]
                    {
                        for segment in &genome.homologues[index] {
                            let start = segment.start.max(lo);
                            let end = segment.end.min(hi);
                            if start < end {
                                if let Some(last) = segments.last_mut() {
                                    let last: &mut FounderSegment = last;
                                    if last.end == start && last.allele == segment.allele {
                                        last.end = end;
                                        continue;
                                    }
                                }
                                segments.push(FounderSegment {
                                    start,
                                    end,
                                    allele: segment.allele,
                                });
                            }
                        }
                    }
                } else {
                    segments.push(FounderSegment {
                        start: 0,
                        end: MODEL_LENGTH,
                        allele: FounderAllele {
                            founder: subject,
                            homologue: slot as u8,
                            unresolved_parent: true,
                        },
                    });
                }
                if segments.len() > 4096 {
                    return Err(Error::QueryBudgetExceeded);
                }
            }
            active.remove(&subject);
            let result = FounderGenome {
                model: FOUNDER_MODEL_VERSION,
                homologues,
            };
            memo.insert(subject, result.clone());
            Ok(result)
        }
        visit(
            self,
            view,
            subject,
            seed,
            0,
            &mut HashSet::new(),
            &mut HashMap::new(),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct FounderMatch {
    pub first_homologue: u8,
    pub second_homologue: u8,
    pub segment: FounderSegment,
}

impl crate::Store {
    /// Match identical founder alleles, retaining both homologue identities.
    /// These intervals are not a real DNA match probability or centimorgan estimate.
    pub fn founder_matches(
        &self,
        view: crate::ViewContext,
        first: ObjectRef,
        second: ObjectRef,
        seed: u64,
    ) -> Result<Vec<FounderMatch>, Error> {
        if first == second {
            return Err(Error::InvalidInput("select two distinct people"));
        }
        let a = self.founder_genome(view, first, seed)?;
        let b = self.founder_genome(view, second, seed)?;
        let mut result = Vec::new();
        for (i, left) in a.homologues.iter().enumerate() {
            for (j, right) in b.homologues.iter().enumerate() {
                let (mut x, mut y) = (0, 0);
                while x < left.len() && y < right.len() {
                    let l = left[x];
                    let r = right[y];
                    let start = l.start.max(r.start);
                    let end = l.end.min(r.end);
                    if start < end && l.allele == r.allele {
                        if result.len() >= 4096 {
                            return Err(Error::QueryBudgetExceeded);
                        }
                        result.push(FounderMatch {
                            first_homologue: i as u8,
                            second_homologue: j as u8,
                            segment: FounderSegment {
                                start,
                                end,
                                allele: l.allele,
                            },
                        });
                    }
                    if l.end <= r.end {
                        x += 1;
                    }
                    if r.end <= l.end {
                        y += 1;
                    }
                }
            }
        }
        Ok(result)
    }
}

pub const CHROMOSOME_MODEL_VERSION: u16 = 3;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ToyChromosome {
    pub chromosome: u8,
    pub homologues: [Vec<FounderSegment>; 2],
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ChromosomePainter {
    pub model: u16,
    pub length: u32,
    pub chromosomes: Vec<ToyChromosome>,
}

impl crate::Store {
    /// A bounded independent toy autosomal chromosome model with reproducible
    /// chromosome-specific crossover streams. Not actual biological coordinates.
    pub fn chromosome_painter(
        &self,
        view: crate::ViewContext,
        subject: ObjectRef,
        seed: u64,
        count: u8,
    ) -> Result<ChromosomePainter, Error> {
        if !(1..=8).contains(&count) {
            return Err(Error::InvalidInput("toy chromosome count"));
        }
        let mut chromosomes = Vec::new();
        let mut total = 0;
        for chromosome in 1..=count {
            let chromosome_seed =
                mix(seed ^ (u64::from(chromosome) << 32) ^ u64::from(CHROMOSOME_MODEL_VERSION));
            let genome = self.founder_genome(view, subject, chromosome_seed)?;
            total += genome.homologues.iter().map(Vec::len).sum::<usize>();
            if total > 8192 {
                return Err(Error::QueryBudgetExceeded);
            }
            chromosomes.push(ToyChromosome {
                chromosome,
                homologues: genome.homologues,
            });
        }
        Ok(ChromosomePainter {
            model: CHROMOSOME_MODEL_VERSION,
            length: MODEL_LENGTH,
            chromosomes,
        })
    }
}
