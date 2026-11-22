//! Typed, explicitly sourced institutional and causal metadata; draft identities stay separate.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ObjectRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityKind {
    Person,
    Organization,
    Household,
    FamilyHouse,
}

impl IdentityKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Organization => "organization",
            Self::Household => "household",
            Self::FamilyHouse => "family_house",
        }
    }

    pub(crate) fn data(self) -> crate::ObjectData {
        match self {
            Self::Person => crate::ObjectData::Person,
            Self::Organization => crate::ObjectData::Organization,
            Self::Household => crate::ObjectData::Household,
            Self::FamilyHouse => crate::ObjectData::FamilyHouse,
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, crate::Error> {
        match value {
            "person" => Ok(Self::Person),
            "organization" => Ok(Self::Organization),
            "household" => Ok(Self::Household),
            "family_house" => Ok(Self::FamilyHouse),
            _ => Err(crate::Error::WrongKind),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkKind {
    Membership,
    Marriage,
    Succession,
    ConfirmedCause,
    AllegedCause,
}

impl NetworkKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Membership => "membership",
            Self::Marriage => "marriage",
            Self::Succession => "succession",
            Self::ConfirmedCause => "confirmed_cause",
            Self::AllegedCause => "alleged_cause",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, crate::Error> {
        match value {
            "membership" => Ok(Self::Membership),
            "marriage" => Ok(Self::Marriage),
            "succession" => Ok(Self::Succession),
            "confirmed_cause" => Ok(Self::ConfirmedCause),
            "alleged_cause" => Ok(Self::AllegedCause),
            _ => Err(crate::Error::WrongKind),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkInput {
    pub subject: ObjectRef,
    pub target: ObjectRef,
    pub source: ObjectRef,
    pub kind: NetworkKind,
    pub role: String,
    pub starts_at: i64,
    pub ends_before: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkFact {
    pub id: Uuid,
    pub input: NetworkInput,
    pub subject_label: String,
    pub target_label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostProposal {
    pub id: Uuid,
    pub claim: ObjectRef,
    pub label: String,
    pub kind: IdentityKind,
    pub day: i64,
    pub recorded_revision: i64,
    pub reviewed: bool,
    pub promoted: Option<ObjectRef>,
}
