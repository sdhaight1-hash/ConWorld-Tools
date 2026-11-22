//! Typed, explicitly sourced institutional and causal metadata; draft identities stay separate.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ObjectRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityKind {
    Person,
    Concept,
    Organization,
    Household,
    FamilyHouse,
    Faction,
    Conflict,
    Office,
    Title,
    Asset,
    Population,
    ConflictIssue,
    ConflictPhase,
    SuccessionRule,
}

impl IdentityKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Concept => "concept",
            Self::Organization => "organization",
            Self::Household => "household",
            Self::FamilyHouse => "family_house",
            Self::Faction => "faction",
            Self::Conflict => "conflict",
            Self::Office => "office",
            Self::Title => "title",
            Self::Asset => "asset",
            Self::Population => "population",
            Self::ConflictIssue => "conflict_issue",
            Self::ConflictPhase => "conflict_phase",
            Self::SuccessionRule => "succession_rule",
        }
    }

    pub(crate) fn data(self) -> crate::ObjectData {
        match self {
            Self::Person => crate::ObjectData::Person,
            Self::Concept => crate::ObjectData::Concept,
            Self::Organization => crate::ObjectData::Organization,
            Self::Household => crate::ObjectData::Household,
            Self::FamilyHouse => crate::ObjectData::FamilyHouse,
            Self::Faction => crate::ObjectData::Faction,
            Self::Conflict => crate::ObjectData::Conflict,
            Self::Office => crate::ObjectData::Office,
            Self::Title => crate::ObjectData::Title,
            Self::Asset => crate::ObjectData::Asset,
            Self::Population => crate::ObjectData::Population,
            Self::ConflictIssue => crate::ObjectData::ConflictIssue,
            Self::ConflictPhase => crate::ObjectData::ConflictPhase,
            Self::SuccessionRule => crate::ObjectData::SuccessionRule,
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, crate::Error> {
        match value {
            "person" => Ok(Self::Person),
            "concept" => Ok(Self::Concept),
            "organization" => Ok(Self::Organization),
            "household" => Ok(Self::Household),
            "family_house" => Ok(Self::FamilyHouse),
            "faction" => Ok(Self::Faction),
            "conflict" => Ok(Self::Conflict),
            "office" => Ok(Self::Office),
            "title" => Ok(Self::Title),
            "asset" => Ok(Self::Asset),
            "population" => Ok(Self::Population),
            "conflict_issue" => Ok(Self::ConflictIssue),
            "conflict_phase" => Ok(Self::ConflictPhase),
            "succession_rule" => Ok(Self::SuccessionRule),
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
    OfficeHolding,
    OfficeOf,
    TitleHolding,
    TitleOf,
    Ownership,
    IssueOf,
    PhaseOf,
    RuleOf,
    PopulationMember,
    HeirDesignation,
    IdentityRedirect,
    Alliance,
    Settlement,
    CultureLanguage,
    WorkingMaterial,
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
            Self::OfficeHolding => "office_holding",
            Self::OfficeOf => "office_of",
            Self::TitleHolding => "title_holding",
            Self::TitleOf => "title_of",
            Self::Ownership => "ownership",
            Self::IssueOf => "issue_of",
            Self::PhaseOf => "phase_of",
            Self::RuleOf => "rule_of",
            Self::PopulationMember => "population_member",
            Self::HeirDesignation => "heir_designation",
            Self::IdentityRedirect => "identity_redirect",
            Self::Alliance => "alliance",
            Self::Settlement => "settlement",
            Self::CultureLanguage => "culture_language",
            Self::WorkingMaterial => "working_material",
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
            "office_holding" => Ok(Self::OfficeHolding),
            "office_of" => Ok(Self::OfficeOf),
            "title_holding" => Ok(Self::TitleHolding),
            "title_of" => Ok(Self::TitleOf),
            "ownership" => Ok(Self::Ownership),
            "issue_of" => Ok(Self::IssueOf),
            "phase_of" => Ok(Self::PhaseOf),
            "rule_of" => Ok(Self::RuleOf),
            "population_member" => Ok(Self::PopulationMember),
            "heir_designation" => Ok(Self::HeirDesignation),
            "identity_redirect" => Ok(Self::IdentityRedirect),
            "alliance" => Ok(Self::Alliance),
            "settlement" => Ok(Self::Settlement),
            "culture_language" => Ok(Self::CultureLanguage),
            "working_material" => Ok(Self::WorkingMaterial),
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
        if !text[from..end]
            .chars()
            .next()
            .is_some_and(char::is_uppercase)
        {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < words.len()
            && j - i < 4
            && text[words[j - 1].1..words[j].0]
                .chars()
                .all(char::is_whitespace)
            && text[words[j].0..words[j].1]
                .chars()
                .next()
                .is_some_and(char::is_uppercase)
        {
            j += 1;
        }
        if j - i >= 2 && words[j - 1].1 - from <= 200 {
            result.push((from, words[j - 1].1));
        }
        i = j;
    }
    result
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BranchSummary {
    pub id: crate::BranchId,
    pub parent: Option<crate::BranchId>,
    pub day: Option<i64>,
    pub created_revision: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SuccessionPreview {
    pub rule: Option<String>,
    pub incumbent: Option<ObjectRef>,
    pub designated: Option<ObjectRef>,
    pub eligible: bool,
    pub explanation: String,
}
