//! Shared reference vocabulary for the governed ontology (AE-02).
//! No new registry or fact storage is introduced.
use crate::{Error, ObjectRef, Scope, Store, Version, ViewContext, WorldId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactFamily {
    Network,
    Parentage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FactRef {
    pub world: WorldId,
    pub family: FactFamily,
    pub id: Uuid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "reference", rename_all = "snake_case")]
pub enum SemanticRef {
    Object(ObjectRef),
    Fact(FactRef),
}

impl SemanticRef {
    pub fn world(self) -> WorldId {
        match self {
            Self::Object(reference) => reference.world,
            Self::Fact(reference) => reference.world,
        }
    }

    /// Raw reference resolution is author-only until scoped projections exist.
    pub fn validate_author_context(self, view: ViewContext) -> Result<(), Error> {
        if view.scope != Scope::Author {
            return Err(Error::CapabilityDenied);
        }
        if self.world() != view.world {
            return Err(Error::WrongWorld);
        }
        Ok(())
    }
}

/// Canonical fact data, never a second persisted copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticFactData {
    Network(crate::network::NetworkInput),
    Parentage(crate::genealogy::Parentage),
}

/// The original author revision and branch are retained even on inherited reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedFact {
    pub reference: FactRef,
    pub recorded_at: crate::AuthorRevision,
    pub source_branch: crate::BranchId,
    pub rationale: String,
    pub data: SemanticFactData,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolvedSemanticRef {
    Object(Version),
    Fact(ResolvedFact),
}

impl Store {
    /// Resolve one canonical identity in the supplied historical author context.
    pub fn resolve_semantic_ref(
        &self,
        reference: SemanticRef,
        view: ViewContext,
    ) -> Result<Option<ResolvedSemanticRef>, Error> {
        reference.validate_author_context(view)?;
        match reference {
            SemanticRef::Object(object) => Ok(self
                .get_object(object, view)?
                .map(ResolvedSemanticRef::Object)),
            SemanticRef::Fact(fact) => Ok(self
                .resolve_semantic_fact(fact, view)?
                .map(ResolvedSemanticRef::Fact)),
        }
    }

    /// Delegate object resolution to the existing historical registry.
    pub fn resolve_semantic_object(
        &self,
        reference: SemanticRef,
        view: ViewContext,
    ) -> Result<Option<Version>, Error> {
        reference.validate_author_context(view)?;
        match reference {
            SemanticRef::Object(object) => self.get_object(object, view),
            SemanticRef::Fact(_) => Err(Error::WrongKind),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthorRevision, BranchId, WorldDay};

    #[test]
    fn reference_families_do_not_alias_and_round_trip() {
        let world = WorldId(Uuid::new_v4());
        let network = SemanticRef::Fact(FactRef {
            world,
            family: FactFamily::Network,
            id: Uuid::new_v4(),
        });
        let SemanticRef::Fact(fact) = network else {
            unreachable!()
        };
        let parentage = SemanticRef::Fact(FactRef {
            family: FactFamily::Parentage,
            ..fact
        });
        assert_ne!(network, parentage);
        let encoded = serde_json::to_string(&network).unwrap();
        assert_eq!(
            serde_json::from_str::<SemanticRef>(&encoded).unwrap(),
            network
        );
    }

    #[test]
    fn raw_references_deny_non_author_and_cross_world_context() {
        let world = WorldId(Uuid::new_v4());
        let reference = SemanticRef::Object(ObjectRef::new(world));
        let view = ViewContext {
            world,
            branch: BranchId(Uuid::new_v4()),
            revision: AuthorRevision(0),
            day: WorldDay(0),
            scope: Scope::Author,
        };
        assert!(reference.validate_author_context(view).is_ok());
        assert!(matches!(
            reference.validate_author_context(ViewContext {
                scope: Scope::Public,
                ..view
            }),
            Err(Error::CapabilityDenied)
        ));
        assert!(matches!(
            reference.validate_author_context(ViewContext {
                world: WorldId(Uuid::new_v4()),
                ..view
            }),
            Err(Error::WrongWorld)
        ));
    }
}
