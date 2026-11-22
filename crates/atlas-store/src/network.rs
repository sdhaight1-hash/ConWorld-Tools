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
    Faction,
    Conflict,
}

impl IdentityKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Organization => "organization",
            Self::Household => "household",
            Self::FamilyHouse => "family_house",
            Self::Faction => "faction",
            Self::Conflict => "conflict",
        }
    }

    pub(crate) fn data(self) -> crate::ObjectData {
        match self {
            Self::Person => crate::ObjectData::Person,
            Self::Organization => crate::ObjectData::Organization,
            Self::Household => crate::ObjectData::Household,
            Self::FamilyHouse => crate::ObjectData::FamilyHouse,
            Self::Faction => crate::ObjectData::Faction,
            Self::Conflict => crate::ObjectData::Conflict,
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, crate::Error> {
        match value {
            "person" => Ok(Self::Person),
            "organization" => Ok(Self::Organization),
            "household" => Ok(Self::Household),
            "family_house" => Ok(Self::FamilyHouse),
            "faction" => Ok(Self::Faction),
            "conflict" => Ok(Self::Conflict),
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
    ConflictPosition,
}

impl NetworkKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Membership => "membership",
            Self::Marriage => "marriage",
            Self::Succession => "succession",
            Self::ConfirmedCause => "confirmed_cause",
            Self::AllegedCause => "alleged_cause",
            Self::ConflictPosition => "conflict_position",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, crate::Error> {
        match value {
            "membership" => Ok(Self::Membership),
            "marriage" => Ok(Self::Marriage),
            "succession" => Ok(Self::Succession),
            "confirmed_cause" => Ok(Self::ConfirmedCause),
            "alleged_cause" => Ok(Self::AllegedCause),
            "conflict_position" => Ok(Self::ConflictPosition),
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostCandidate {
    pub label: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub existing: Vec<ObjectRef>,
}

/// Find bounded sequences of capitalized words, preserving original UTF-8 bytes.
/// Single-word candidates remain manual to avoid floods of sentence starters.
pub(crate) fn capitalized_spans(text: &str) -> Vec<(usize, usize)> {
    let mut words = Vec::new();
    let mut start = None;
    for (index, character) in text.char_indices() {
        if character.is_alphabetic() {
            start.get_or_insert(index);
        } else if let Some(from) = start.take() {
            words.push((from, index));
        }
    }
    if let Some(from) = start {
        words.push((from, text.len()));
    }
    let mut result = Vec::new();
    let mut i = 0;
    while i < words.len() && result.len() < 50 {
        let (from, end) = words[i];
        if !text[from..end].chars().next().is_some_and(char::is_uppercase) {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < words.len() && j - i < 4 &&
            text[words[j-1].1..words[j].0].chars().all(char::is_whitespace) &&
            text[words[j].0..words[j].1].chars().next().is_some_and(char::is_uppercase) {
            j += 1;
        }
        if j - i >= 2 && words[j-1].1 - from <= 200 {
            result.push((from, words[j-1].1));
        }
        i = j;
    }
    result
}
