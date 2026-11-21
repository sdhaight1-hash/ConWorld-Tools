//! Phase 7 evidence and presentation permission checks.
use crate::{ClaimAudience, Scope};

/// Apply audience rules before projecting claim text or processing a layout subtree.
pub fn can_view(scope: Scope, audience: ClaimAudience) -> bool {
    match (scope, audience) {
        (Scope::Author, _) | (_, ClaimAudience::Public) => true,
        (Scope::Character(viewer), ClaimAudience::Character(reader)) => viewer == reader,
        _ => false,
    }
}

/// Intersect permissions inherited from an enclosing layout element.
pub(crate) fn restrict(audience: ClaimAudience, enclosing: ClaimAudience) -> ClaimAudience {
    match (audience, enclosing) {
        (ClaimAudience::Public, other) | (other, ClaimAudience::Public) => other,
        (ClaimAudience::Character(a), ClaimAudience::Character(b)) if a == b => audience,
        _ => ClaimAudience::Author,
    }
}
