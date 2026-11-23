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
    Classification,
    SiteArea,
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

impl FactFamily {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Parentage => "parentage",
            Self::Classification => "classification",
            Self::SiteArea => "site_area",
        }
    }
}

/// An annotation is a reference to an existing sourced Claim, not a second statement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FactAnnotation {
    pub id: Uuid,
    pub fact: FactRef,
    pub claim: ObjectRef,
    pub recorded_at: crate::AuthorRevision,
}

/// Canonical fact data, never a second persisted copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticFactData {
    Network(crate::network::NetworkInput),
    Parentage(crate::genealogy::Parentage),
    Classification(ClassificationInput),
    SiteArea(SiteAreaInput),
}

/// The original author revision and branch are retained even on inherited reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedFact {
    pub reference: FactRef,
    pub recorded_at: crate::AuthorRevision,
    pub source_branch: crate::BranchId,
    pub rationale: String,
    /// The stable predecessor of a corrected fact, if its family supports correction.
    pub corrects: Option<Uuid>,
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

/// Initial curated profiles. A profile is classification metadata, not a new object kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConceptProfile {
    Culture,
    Language,
    Material,
    Technology,
}

impl ConceptProfile {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Culture => "culture",
            Self::Language => "language",
            Self::Material => "material",
            Self::Technology => "technology",
        }
    }

    pub(crate) fn parse(key: &str) -> Result<Self, Error> {
        match key {
            "culture" => Ok(Self::Culture),
            "language" => Ok(Self::Language),
            "material" => Ok(Self::Material),
            "technology" => Ok(Self::Technology),
            _ => Err(Error::InvalidInput("concept profile")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClassificationInput {
    pub subject: ObjectRef,
    pub profile: ConceptProfile,
    pub descriptor: DefinitionPin,
    pub source: ObjectRef,
    pub starts_at: crate::WorldDay,
    pub ends_before: Option<crate::WorldDay>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Classification {
    pub id: Uuid,
    pub input: ClassificationInput,
    pub recorded_at: crate::AuthorRevision,
    pub source_branch: crate::BranchId,
}

/// Fixed, version-one units for a recorded site-area measurement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AreaUnit {
    Hectare,
    SquareMetre,
}

impl AreaUnit {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Hectare => "ha",
            Self::SquareMetre => "m2",
        }
    }

    pub(crate) fn parse(key: &str) -> Result<Self, Error> {
        match key {
            "ha" => Ok(Self::Hectare),
            "m2" => Ok(Self::SquareMetre),
            _ => Err(Error::InvalidInput("site area unit")),
        }
    }
}

/// A missing fact, an explicit unknown reading, and a known zero are distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum AreaReading {
    Unknown,
    Known { amount: i64, unit: AreaUnit },
}

/// Explicit, unit-preserving predicates. Unknown is not missing or zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "test", rename_all = "snake_case", deny_unknown_fields)]
pub enum AreaTest {
    Unknown,
    AtLeast { amount: i64, unit: AreaUnit },
}

impl AreaTest {
    pub(crate) fn validate(self) -> Result<(), Error> {
        if let Self::AtLeast { amount, .. } = self
            && amount < 0
        {
            return Err(Error::InvalidInput("site area threshold"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SiteAreaInput {
    pub place: ObjectRef,
    pub descriptor: DefinitionPin,
    pub source: ObjectRef,
    pub reading: AreaReading,
    pub starts_at: crate::WorldDay,
    pub ends_before: Option<crate::WorldDay>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SiteAreaFact {
    pub id: Uuid,
    pub input: SiteAreaInput,
    pub recorded_at: crate::AuthorRevision,
    pub source_branch: crate::BranchId,
    pub corrects: Option<Uuid>,
}

/// A governed descriptor pins the exact historical definition, not its label.
/// This is an author-side reference check, not a persisted classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefinitionPin {
    pub version: crate::VersionRef,
}

impl Store {
    /// Check that a pinned definition is still the selected historical version.
    /// Stale pins fail closed; callers must not silently reinterpret old data.
    pub fn validate_definition_pin(
        &self,
        pin: DefinitionPin,
        view: ViewContext,
    ) -> Result<Version, Error> {
        let reference = SemanticRef::Object(pin.version.object);
        let version = self
            .resolve_semantic_object(reference, view)?
            .ok_or(Error::NotFound)?;
        if !matches!(&version.record.data, crate::ObjectData::Definition { .. }) {
            return Err(Error::WrongKind);
        }
        if version.reference() != pin.version {
            return Err(Error::InvalidInput("definition version pin"));
        }
        Ok(version)
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
