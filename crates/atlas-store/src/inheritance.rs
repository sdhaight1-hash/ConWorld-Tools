//! Phase 8 reproducible fictional ancestry intervals. No nucleotide or genome data.
use crate::{
    Error, ObjectRef,
    genealogy::{ParentageKind, Pedigree},
};

pub const MODEL_VERSION: u16 = 1;
pub const MODEL_LENGTH: u32 = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
