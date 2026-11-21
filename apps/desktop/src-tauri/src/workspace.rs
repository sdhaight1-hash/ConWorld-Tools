//! Minimal persistent author workspace for Phase Two.
//! Canonical records live only in atlas-store; the Northbridge fixture is separate.
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use atlas_store::{
    ActionOutcome, AuthorRevision, Change, ClaimAudience, Command, Definition, EventDate,
    EventRole, ObjectData, ObjectId, ObjectRef, Record, Scope, Store, ValidInterval, Version,
    VersionRef, ViewContext, World, WorldDay, WorldId,
    discovery::{MentionDisposition, ReviewMention, SavePersonAlias},
    query::{
        Direction, ParameterBinding, PredicateInput, QUERY_SCHEMA_VERSION, QueryControl, QueryCursor,
        QueryFilter, QueryParameter, QuerySpec, SaveNamedLabelQuery,
    },
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

pub struct AuthorWorkspace {
    store: Mutex<Store>,
    continuation: Mutex<Option<SearchContinuation>>,
    query_jobs: Mutex<QueryJobs>,
}

#[derive(Default)]
struct QueryJobs {
    active: HashMap<Uuid, QueryControl>,
    cancelled: VecDeque<Uuid>,
}

struct SearchContinuation {
    token: Uuid,
    spec: QuerySpec,
    view: ViewContext,
    cursor: QueryCursor,
}

#[derive(Serialize)]
pub struct WorldSummary {
    id: String,
    name: String,
    revision: i64,
}

#[derive(Clone, Deserialize)]
pub struct PathQueryInput {
    world_id: String,
    day: i64,
    revision: Option<i64>,
    start_id: String,
    kind: String,
    direction: String,
    max_depth: u8,
    max_visited: usize,
    limit: usize,
}

#[derive(Deserialize)]
pub struct ParticipantEventsInput {
    world_id: String,
    day: i64,
    revision: Option<i64>,
    participant_id: String,
    role: String,
    limit: usize,
}

#[derive(Deserialize)]
pub struct CompositeQueryInput {
    world_id: String,
    day: i64,
    revision: Option<i64>,
    predicate: PredicateInput,
    limit: usize,
}

#[derive(Deserialize)]
pub struct SavedQueryInput {
    world_id: String,
    expected_revision: i64,
    query_id: Option<String>,
    name: String,
    mode: String,
    term: String,
    predicate: Option<PredicateInput>,
    limit: usize,
    #[serde(default)]
    parameterize: bool,
}

#[derive(Deserialize)]
pub struct ReplaySavedQueryInput {
    world_id: String,
    day: i64,
    revision: i64,
    query_id: String,
    bindings: Vec<ParameterBinding>,
}

#[derive(Serialize)]
pub struct SavedQuerySummary {
    id: String,
    name: String,
    mode: &'static str,
    term: String,
    predicate: Option<PredicateInput>,
    limit: usize,
    recorded_at: i64,
    parameters: Vec<QueryParameter>,
}

#[derive(Deserialize)]
pub struct PersonAliasInput {
    world_id: String,
    expected_revision: i64,
    day: i64,
    person_id: String,
    alias_id: Option<String>,
    label: String,
    starts_at: Option<i64>,
    ends_before: Option<i64>,
    active: bool,
}

#[derive(Serialize)]
pub struct PersonAliasSummary {
    id: String,
    person_id: String,
    label: String,
    starts_at: Option<i64>,
    ends_before: Option<i64>,
    active: bool,
    recorded_at: i64,
}

#[derive(Deserialize)]
pub struct ReviewMentionInput {
    world_id: String,
    day: i64,
    expected_revision: i64,
    source_id: String,
    source_branch: String,
    source_state_revision: i64,
    source_revision: i64,
    target_id: String,
    start_byte: usize,
    end_byte: usize,
    decision: String,
}

#[derive(Serialize)]
pub struct MentionDiscoveryRow {
    source_id: String,
    source_branch: String,
    source_state_revision: i64,
    source_revision: i64,
    source_label: String,
    statement: String,
    target_id: String,
    target_label: String,
    matched_label: String,
    start_byte: usize,
    end_byte: usize,
    collisions: usize,
    disposition: Option<&'static str>,
}

#[derive(Serialize)]
pub struct MentionDiscoverySummary {
    revision: i64,
    truncated: bool,
    rows: Vec<MentionDiscoveryRow>,
}

#[derive(Serialize)]
pub struct SearchRow {
    id: String,
    label: String,
    kind: &'static str,
    statement: Option<String>,
    source_citation: Option<String>,
}

#[derive(Serialize)]
pub struct SearchSummary {
    revision: i64,
    truncated: bool,
    next_cursor: Option<String>,
    rows: Vec<SearchRow>,
}

#[derive(Serialize)]
pub struct PersonSummary {
    id: String,
    label: String,
}

#[derive(Serialize)]
pub struct RelationshipSummary {
    id: String,
    subject_id: String,
    target_id: String,
    kind: String,
    trust: Option<u8>,
    tags: Vec<String>,
    starts_at: Option<i64>,
    cause: Option<CauseSummary>,
}

#[derive(Serialize)]
pub struct CauseSummary {
    day: i64,
    event_id: String,
    event: String,
    explanation: String,
    before_trust: Option<u8>,
    before_tags: Vec<String>,
    after_trust: Option<u8>,
    after_tags: Vec<String>,
    actor: Option<String>,
    action: Option<String>,
    intent: Option<String>,
    outcome: Option<&'static str>,
    source: String,
}

#[derive(Serialize)]
pub struct WorkspaceSnapshot {
    world: WorldSummary,
    view_revision: i64,
    day: i64,
    /// True when a complete editable scene exceeds a bounded read budget.
    scene_limited: bool,
    people: Vec<PersonSummary>,
    relationships: Vec<RelationshipSummary>,
    events: Vec<EventSummary>,
    definitions: Vec<DefinitionSummary>,
    aliases: Vec<PersonAliasSummary>,
}

impl WorkspaceSnapshot {
    fn search_only(world: World, context: ViewContext) -> Self {
        Self {
            world: summary(world),
            view_revision: context.revision.0,
            day: context.day.0,
            scene_limited: true,
            people: Vec::new(),
            relationships: Vec::new(),
            events: Vec::new(),
            definitions: Vec::new(),
            aliases: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct DefinitionSummary {
    id: String,
    label: String,
    category: &'static str,
    description: String,
    reverse_label: Option<String>,
    symmetric: bool,
}

#[derive(Serialize)]
pub struct EvidenceSummary {
    statement: String,
    source: Option<String>,
    audience: Option<String>,
    reader_id: Option<String>,
}

#[derive(Deserialize)]
pub struct DefinitionInput {
    world_id: String,
    expected_revision: i64,
    category: String,
    label: String,
    description: String,
    reverse_label: Option<String>,
    symmetric: bool,
}

#[derive(Deserialize)]
pub struct EvidenceInput {
    world_id: String,
    expected_revision: i64,
    event_id: String,
    day: i64,
    statement: String,
    citation: String,
    audience: String,
}

fn definition_category(definition: &Definition) -> &'static str {
    match definition {
        Definition::Relationship { .. } => "relationship",
        Definition::Tag => "tag",
        Definition::Role => "role",
    }
}

#[derive(Serialize)]
struct EventSummary {
    id: String,
    label: String,
    day: Option<i64>,
    citation: Option<String>,
    participants: Vec<ParticipantSummary>,
    actions: Vec<ActionSummary>,
}

#[derive(Serialize)]
struct ParticipantSummary {
    person_id: String,
    role: String,
}

#[derive(Serialize)]
struct ActionSummary {
    id: String,
    person_id: String,
    target_id: Option<String>,
    action: String,
    intent: String,
    outcome: &'static str,
    citation: Option<String>,
}

#[derive(Deserialize)]
pub struct RelationshipInput {
    world_id: String,
    subject_id: String,
    target_id: String,
    kind: String,
    trust: Option<u8>,
    tags: String,
    day: i64,
}

#[derive(Deserialize)]
pub struct RelationshipStateInput {
    world_id: String,
    expected_revision: i64,
    relationship_id: String,
    day: i64,
    trust: Option<u8>,
    tags: String,
}

#[derive(Clone, Deserialize)]
pub struct EventInput {
    world_id: String,
    expected_revision: i64,
    event_id: Option<String>,
    day: i64,
    event_label: String,
    source_citation: String,
    participants: Vec<ParticipantInput>,
    effect: Option<EffectInput>,
}

#[derive(Clone, Deserialize)]
struct ParticipantInput {
    person_id: String,
    role: String,
    action: Option<ActionInput>,
}

#[derive(Clone, Deserialize)]
struct ActionInput {
    id: Option<String>,
    action: String,
    intent: String,
    outcome: String,
    target_id: Option<String>,
}

#[derive(Clone, Deserialize)]
struct EffectInput {
    relationship_id: String,
    actor_row: usize,
    explanation: String,
    trust: Option<u8>,
    tags: String,
}

fn roles() -> [(&'static str, EventRole); 5] {
    [
        ("actor", EventRole::Actor),
        ("witness", EventRole::Witness),
        ("victim", EventRole::Victim),
        ("stakeholder", EventRole::Stakeholder),
        ("sponsor", EventRole::Sponsor),
    ]
}

fn outcomes() -> [(&'static str, ActionOutcome); 3] {
    [
        ("attempted", ActionOutcome::Attempted),
        ("succeeded", ActionOutcome::Succeeded),
        ("failed", ActionOutcome::Failed),
    ]
}

fn enum_label<T: PartialEq>(choices: &[(&'static str, T)], value: T) -> &'static str {
    choices
        .iter()
        .find(|(_, item)| *item == value)
        .expect("complete choices")
        .0
}

fn enum_value<T: Copy>(choices: &[(&str, T)], label: &str) -> Option<T> {
    choices
        .iter()
        .find(|(name, _)| *name == label)
        .map(|(_, value)| *value)
}

fn id(raw: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw).map_err(|_| "Invalid world or object identifier".into())
}

fn nonempty(value: &str, field: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 200 {
        return Err(format!("{field} must contain between 1 and 200 characters"));
    }
    Ok(value.into())
}

fn tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn reference(world: WorldId, raw: &str) -> Result<ObjectRef, String> {
    Ok(ObjectRef {
        world,
        id: ObjectId(id(raw)?),
    })
}

fn view(world: &World, day: i64) -> ViewContext {
    ViewContext {
        world: world.id,
        branch: world.main,
        revision: world.revision,
        day: WorldDay(day),
        scope: Scope::Author,
    }
}

fn summary(world: World) -> WorldSummary {
    WorldSummary {
        id: world.id.0.to_string(),
        name: world.name,
        revision: world.revision.0,
    }
}

fn commit(store: &mut Store, world: &World, changes: Vec<Change>) -> Result<(), String> {
    store
        .apply(&Command {
            world: world.id,
            branch: world.main,
            expected: world.revision,
            id: Uuid::new_v4(),
            author: "Local author".into(),
            rationale: "Phase Two authoring edit".into(),
            scope: Scope::Author,
            changes,
        })
        .map(|_| ())
        .map_err(|error| error.to_string())
}

impl AuthorWorkspace {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, atlas_store::Error> {
        Ok(Self {
            store: Mutex::new(Store::open(path)?),
            continuation: Mutex::new(None),
            query_jobs: Mutex::new(QueryJobs::default()),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Store>, String> {
        self.store
            .lock()
            .map_err(|_| "Author workspace storage lock was poisoned".into())
    }

    fn lock_continuation(&self) -> Result<MutexGuard<'_, Option<SearchContinuation>>, String> {
        self.continuation
            .lock()
            .map_err(|_| "Author workspace continuation lock was poisoned".into())
    }

    fn begin_query(&self, request: Uuid) -> Result<QueryControl, String> {
        let mut jobs = self
            .query_jobs
            .lock()
            .map_err(|_| "Query job lock poisoned")?;
        if jobs.cancelled.contains(&request) {
            return Err("query cancelled".into());
        }
        if jobs.active.len() >= 8 || jobs.active.contains_key(&request) {
            return Err("Too many active or duplicate query requests".into());
        }
        let control =
            QueryControl::new(Duration::from_secs(10)).map_err(|error| error.to_string())?;
        jobs.active.insert(request, control.clone());
        Ok(control)
    }

    fn finish_query(&self, request: Uuid) -> Result<(), String> {
        let mut jobs = self
            .query_jobs
            .lock()
            .map_err(|_| "Query job lock poisoned")?;
        jobs.active.remove(&request);
        Ok(())
    }

    fn cancel_query(&self, request: Uuid) -> Result<(), String> {
        let mut jobs = self
            .query_jobs
            .lock()
            .map_err(|_| "Query job lock poisoned")?;
        if let Some(control) = jobs.active.get(&request) {
            control.cancel();
        } else if !jobs.cancelled.contains(&request) {
            // A fast cancel can arrive before the async command registers.
            if jobs.cancelled.len() == 64 {
                jobs.cancelled.pop_front();
            }
            jobs.cancelled.push_back(request);
        }
        Ok(())
    }

    fn worlds(&self) -> Result<Vec<WorldSummary>, String> {
        self.lock()?
            .worlds(Scope::Author)
            .map(|worlds| worlds.into_iter().map(summary).collect())
            .map_err(|error| error.to_string())
    }

    fn create_world(&self, name: &str) -> Result<WorldSummary, String> {
        let name = nonempty(name, "World name")?;
        self.lock()?
            .create_world(&name, Scope::Author)
            .map(summary)
            .map_err(|error| error.to_string())
    }

    fn saved_queries(
        &self,
        world_id: &str,
        revision: Option<i64>,
    ) -> Result<Vec<SavedQuerySummary>, String> {
        let store = self.lock()?;
        let world = store
            .world(WorldId(id(world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let queries = store
            .saved_label_queries(
                world.id,
                AuthorRevision(revision.unwrap_or(world.revision.0)),
                Scope::Author,
            )
            .map_err(|error| error.to_string())?;
        queries
            .into_iter()
            .map(|query| {
                let (mode, term, predicate) = match &query.spec.filter {
                    QueryFilter::LabelContains(term) => ("contains", term.clone(), None),
                    QueryFilter::ExactLabel(term) => ("exact", term.clone(), None),
                    QueryFilter::ExactAlias(term) => ("alias", term.clone(), None),
                    QueryFilter::FullTextPhrase(term) => ("fulltext", term.clone(), None),
                    QueryFilter::ClaimFullTextPhrase(term) => ("claimtext", term.clone(), None),
                    filter => (
                        "composite",
                        String::new(),
                        Some(
                            PredicateInput::from_filter(filter)
                                .map_err(|error| error.to_string())?,
                        ),
                    ),
                };
                Ok(SavedQuerySummary {
                    id: query.id.to_string(),
                    name: query.name,
                    mode,
                    term,
                    predicate,
                    limit: query.spec.limit,
                    recorded_at: query.recorded_at.0,
                    parameters: query.parameters,
                })
            })
            .collect::<Result<Vec<_>, String>>()
    }

    fn save_query(&self, request: SavedQueryInput) -> Result<String, String> {
        let filter = match request.mode.as_str() {
            "contains" => QueryFilter::LabelContains(request.term),
            "exact" => QueryFilter::ExactLabel(request.term),
            "alias" => QueryFilter::ExactAlias(request.term),
            "fulltext" => QueryFilter::FullTextPhrase(request.term),
            "claimtext" => QueryFilter::ClaimFullTextPhrase(request.term),
            "composite" => {
                let mut nodes = 0;
                request
                    .predicate
                    .ok_or("Missing typed composite search predicate")?
                    .into_filter(WorldId(id(&request.world_id)?), 0, &mut nodes)
                    .map_err(|error| error.to_string())?
            }
            _ => return Err("Unsupported saved query type".into()),
        };
        let spec = QuerySpec {
            schema_version: QUERY_SCHEMA_VERSION,
            filter,
            limit: request.limit,
        };
        let query_id = request.query_id.as_deref().map(id).transpose()?;
        let mut store = self.lock()?;
        let definition = SaveNamedLabelQuery {
            world_id: WorldId(id(&request.world_id)?),
            expected: AuthorRevision(request.expected_revision),
            query_id,
            name: &request.name,
            spec: &spec,
            command_id: Uuid::new_v4(),
            scope: Scope::Author,
        };
        let saved = if request.parameterize {
            store.save_named_parameterized_query(definition)
        } else {
            store.save_named_label_query(definition)
        }
        .map_err(|error| error.to_string())?;
        Ok(saved.id.to_string())
    }

    fn replay_saved_query(
        &self,
        request: ReplaySavedQueryInput,
        control: &QueryControl,
    ) -> Result<SearchSummary, String> {
        let spec = {
            let store = self.lock()?;
            control.check().map_err(|error| error.to_string())?;
            let world = store
                .world(WorldId(id(&request.world_id)?), Scope::Author)
                .map_err(|error| error.to_string())?;
            let context = ViewContext {
                revision: AuthorRevision(request.revision),
                ..view(&world, request.day)
            };
            let query_id = id(&request.query_id)?;
            let saved = store
                .saved_label_queries(world.id, context.revision, Scope::Author)
                .map_err(|error| error.to_string())?
                .into_iter()
                .find(|query| query.id == query_id)
                .ok_or("Saved query does not exist at this revision.")?;
            saved
                .bind(context, &request.bindings)
                .map_err(|error| error.to_string())?
        };
        self.execute_filter(
            &request.world_id,
            request.day,
            Some(request.revision),
            spec.filter,
            spec.limit,
            control,
        )
    }

    fn save_person_alias(&self, input: PersonAliasInput) -> Result<String, String> {
        let world = WorldId(id(&input.world_id)?);
        let mut store = self.lock()?;
        let saved = store
            .save_person_alias(SavePersonAlias {
                world,
                person: ObjectRef {
                    world,
                    id: ObjectId(id(&input.person_id)?),
                },
                day: WorldDay(input.day),
                expected: AuthorRevision(input.expected_revision),
                alias_id: input.alias_id.as_deref().map(id).transpose()?,
                label: &input.label,
                starts_at: input.starts_at.map(WorldDay),
                ends_before: input.ends_before.map(WorldDay),
                active: input.active,
                command_id: Uuid::new_v4(),
                scope: Scope::Author,
            })
            .map_err(|error| error.to_string())?;
        Ok(saved.id.to_string())
    }

    fn review_mention(&self, input: ReviewMentionInput) -> Result<i64, String> {
        let decision = match input.decision.as_str() {
            "approved" => MentionDisposition::Approved,
            "dismissed" => MentionDisposition::Dismissed,
            _ => return Err("Unknown mention decision".into()),
        };
        let mut store = self.lock()?;
        let world = store
            .world(WorldId(id(&input.world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let revision = store
            .review_mention(ReviewMention {
                view: ViewContext {
                    revision: AuthorRevision(input.expected_revision),
                    ..view(&world, input.day)
                },
                expected: AuthorRevision(input.expected_revision),
                source: VersionRef {
                    object: ObjectRef {
                        world: world.id,
                        id: ObjectId(id(&input.source_id)?),
                    },
                    branch: atlas_store::BranchId(id(&input.source_branch)?),
                    state_revision: AuthorRevision(input.source_state_revision),
                    recorded_at: AuthorRevision(input.source_revision),
                },
                target: ObjectRef {
                    world: world.id,
                    id: ObjectId(id(&input.target_id)?),
                },
                start_byte: input.start_byte,
                end_byte: input.end_byte,
                decision,
                command_id: Uuid::new_v4(),
            })
            .map_err(|error| error.to_string())?;
        Ok(revision.0)
    }

    fn discover_mentions(
        &self,
        world_id: &str,
        day: i64,
        revision: Option<i64>,
        limit: usize,
    ) -> Result<MentionDiscoverySummary, String> {
        let store = self.lock()?;
        let world = store
            .world(WorldId(id(world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let context = ViewContext {
            revision: AuthorRevision(revision.unwrap_or(world.revision.0)),
            ..view(&world, day)
        };
        let discovery = store
            .discover_mentions(context, limit)
            .map_err(|error| error.to_string())?;
        Ok(MentionDiscoverySummary {
            revision: discovery.revision.0,
            truncated: discovery.truncated,
            rows: discovery
                .rows
                .into_iter()
                .map(|candidate| MentionDiscoveryRow {
                    source_id: candidate.source.object.id.0.to_string(),
                    source_branch: candidate.source.branch.0.to_string(),
                    source_state_revision: candidate.source.state_revision.0,
                    source_revision: candidate.source.recorded_at.0,
                    source_label: candidate.source_label,
                    statement: candidate.statement,
                    target_id: candidate.target.id.0.to_string(),
                    target_label: candidate.target_label,
                    matched_label: candidate.matched_label,
                    start_byte: candidate.start_byte,
                    end_byte: candidate.end_byte,
                    collisions: candidate.collisions,
                    disposition: match candidate.disposition {
                        Some(MentionDisposition::Approved) => Some("approved"),
                        Some(MentionDisposition::Dismissed) => Some("dismissed"),
                        None => None,
                    },
                })
                .collect(),
        })
    }

    fn search(
        &self,
        world_id: &str,
        day: i64,
        revision: Option<i64>,
        mode_term: (&str, &str),
        limit: usize,
        control: &QueryControl,
    ) -> Result<SearchSummary, String> {
        let (mode, term) = mode_term;
        let filter = match mode {
            "contains" => QueryFilter::LabelContains(term.into()),
            "exact" => QueryFilter::ExactLabel(term.into()),
            "alias" => QueryFilter::ExactAlias(term.into()),
            "fulltext" => QueryFilter::FullTextPhrase(term.into()),
            "claimtext" => QueryFilter::ClaimFullTextPhrase(term.into()),
            "backlinks" => QueryFilter::Backlinks(ObjectRef {
                world: WorldId(id(world_id)?),
                id: ObjectId(id(term.trim())?),
            }),
            _ => return Err("Unknown historical search mode".into()),
        };
        self.execute_filter(world_id, day, revision, filter, limit, control)
    }

    fn query_path(
        &self,
        request: PathQueryInput,
        control: &QueryControl,
    ) -> Result<SearchSummary, String> {
        let direction = match request.direction.as_str() {
            "outgoing" => Direction::Outgoing,
            "incoming" => Direction::Incoming,
            "both" => Direction::Both,
            _ => return Err("Unknown relationship direction".into()),
        };
        let filter = QueryFilter::RelationshipPath {
            start: ObjectRef {
                world: WorldId(id(&request.world_id)?),
                id: ObjectId(id(&request.start_id)?),
            },
            kind: request.kind,
            direction,
            max_depth: request.max_depth,
            max_visited: request.max_visited,
        };
        self.execute_filter(
            &request.world_id,
            request.day,
            request.revision,
            filter,
            request.limit,
            control,
        )
    }

    fn participant_events(
        &self,
        request: ParticipantEventsInput,
        control: &QueryControl,
    ) -> Result<SearchSummary, String> {
        let world_id = WorldId(id(&request.world_id)?);
        let role = match request.role.as_str() {
            "any" => None,
            "actor" => Some(EventRole::Actor),
            "witness" => Some(EventRole::Witness),
            "victim" => Some(EventRole::Victim),
            "stakeholder" => Some(EventRole::Stakeholder),
            "sponsor" => Some(EventRole::Sponsor),
            other => {
                let definition = other
                    .strip_prefix("definition:")
                    .ok_or("Unknown event participation role")?;
                Some(EventRole::Custom(ObjectRef {
                    world: world_id,
                    id: ObjectId(id(definition)?),
                }))
            }
        };
        self.execute_filter(
            &request.world_id,
            request.day,
            request.revision,
            QueryFilter::ParticipantEvents {
                participant: ObjectRef {
                    world: world_id,
                    id: ObjectId(id(&request.participant_id)?),
                },
                role,
            },
            request.limit,
            control,
        )
    }

    fn composite_query(
        &self,
        request: CompositeQueryInput,
        control: &QueryControl,
    ) -> Result<SearchSummary, String> {
        let mut nodes = 0;
        let predicate = request
            .predicate
            .into_filter(WorldId(id(&request.world_id)?), 0, &mut nodes)
            .map_err(|error| error.to_string())?;
        self.execute_filter(
            &request.world_id,
            request.day,
            request.revision,
            predicate,
            request.limit,
            control,
        )
    }

    fn execute_filter(
        &self,
        world_id: &str,
        day: i64,
        revision: Option<i64>,
        filter: QueryFilter,
        limit: usize,
        control: &QueryControl,
    ) -> Result<SearchSummary, String> {
        let store = self.lock()?;
        control.check().map_err(|error| error.to_string())?;
        let world = store
            .world(WorldId(id(world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let view = ViewContext {
            revision: AuthorRevision(revision.unwrap_or(world.revision.0)),
            ..view(&world, day)
        };
        let mut continuation = self.lock_continuation()?;
        *continuation = None;
        let spec = QuerySpec {
            schema_version: QUERY_SCHEMA_VERSION,
            filter,
            limit,
        };
        if matches!(spec.filter, QueryFilter::RelationshipPath { .. }) {
            let result = store
                .execute_query_controlled(&spec, view, control)
                .map_err(|error| error.to_string())?;
            return Self::search_summary(
                &store,
                view,
                result.rows,
                result.truncated,
                None,
                control,
            );
        }
        let page = store
            .execute_query_page_controlled(&spec, view, None, control)
            .map_err(|error| error.to_string())?;
        let next = page.next_cursor.map(|cursor| SearchContinuation {
            token: Uuid::new_v4(),
            spec,
            view,
            cursor,
        });
        let token = next.as_ref().map(|item| item.token.to_string());
        let summary =
            Self::search_summary(&store, view, page.rows, token.is_some(), token, control)?;
        control.check().map_err(|error| error.to_string())?;
        *continuation = next;
        Ok(summary)
    }

    fn next_page(
        &self,
        world_id: &str,
        day: i64,
        revision: i64,
        token: &str,
        control: &QueryControl,
    ) -> Result<SearchSummary, String> {
        let store = self.lock()?;
        control.check().map_err(|error| error.to_string())?;
        let mut continuation = self.lock_continuation()?;
        let current = continuation
            .as_ref()
            .filter(|item| {
                item.token.to_string() == token
                    && item.view.world.0.to_string() == world_id
                    && item.view.day.0 == day
                    && item.view.revision.0 == revision
            })
            .ok_or("Search page expired; run the search again.")?;
        let page = store
            .execute_query_page_controlled(
                &current.spec,
                current.view,
                Some(&current.cursor),
                control,
            )
            .map_err(|error| error.to_string())?;
        let view = current.view;
        let next = page.next_cursor.map(|cursor| SearchContinuation {
            token: Uuid::new_v4(),
            spec: current.spec.clone(),
            view,
            cursor,
        });
        let token = next.as_ref().map(|item| item.token.to_string());
        let summary =
            Self::search_summary(&store, view, page.rows, token.is_some(), token, control)?;
        control.check().map_err(|error| error.to_string())?;
        *continuation = next;
        Ok(summary)
    }

    fn search_summary(
        store: &Store,
        view: ViewContext,
        rows: Vec<Version>,
        truncated: bool,
        next_cursor: Option<String>,
        control: &QueryControl,
    ) -> Result<SearchSummary, String> {
        Ok(SearchSummary {
            revision: view.revision.0,
            truncated,
            next_cursor,
            rows: rows
                .into_iter()
                .map(|version| {
                    control.check().map_err(|error| error.to_string())?;
                    let statement = match &version.record.data {
                        ObjectData::Claim { statement, .. } => Some(statement.clone()),
                        _ => None,
                    };
                    let source_citation = if statement.is_some() {
                        version
                            .record
                            .source
                            .map(|source| store.get_object(source, view))
                            .transpose()
                            .map_err(|error| error.to_string())?
                            .flatten()
                            .and_then(|source| match source.record.data {
                                ObjectData::Source { citation } => Some(citation),
                                _ => None,
                            })
                    } else {
                        None
                    };
                    let kind = match version.record.data {
                        ObjectData::Person => "person",
                        ObjectData::Definition { .. } => "definition",
                        ObjectData::Relationship { .. } => "relationship",
                        ObjectData::EventInvolvement { .. } => "event_involvement",
                        ObjectData::ActorAction { .. } => "actor_action",
                        ObjectData::HistoricalEvent { .. } => "historical_event",
                        ObjectData::Source { .. } => "source",
                        ObjectData::Claim { .. } => "claim",
                    };
                    Ok(SearchRow {
                        id: version.object.id.0.to_string(),
                        label: version.record.label,
                        kind,
                        statement,
                        source_citation,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        })
    }

    fn snapshot(
        &self,
        world_id: &str,
        day: i64,
        revision: Option<i64>,
    ) -> Result<WorkspaceSnapshot, String> {
        let store = self.lock()?;
        *self.lock_continuation()? = None;
        let world = store
            .world(WorldId(id(world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let context = ViewContext {
            revision: AuthorRevision(revision.unwrap_or(world.revision.0)),
            ..view(&world, day)
        };
        let versions = match store.objects_at(context, 2_000) {
            Ok(versions) => versions,
            Err(atlas_store::Error::QueryBudgetExceeded) => {
                return Ok(WorkspaceSnapshot::search_only(world, context));
            }
            Err(error) => return Err(error.to_string()),
        };
        let alias_versions = match store.person_aliases(context) {
            Ok(aliases) => aliases,
            Err(atlas_store::Error::QueryBudgetExceeded) => {
                return Ok(WorkspaceSnapshot::search_only(world, context));
            }
            Err(error) => return Err(error.to_string()),
        };
        let mut people = Vec::new();
        let mut relationships = Vec::new();
        let mut events = HashMap::new();
        let mut participants = Vec::new();
        let mut definitions = Vec::new();
        let mut actions = Vec::new();
        for version in versions {
            let object_id = version.object.id.0.to_string();
            match version.record.data {
                ObjectData::Person => people.push(PersonSummary {
                    id: object_id,
                    label: version.record.label,
                }),
                ObjectData::Definition {
                    definition,
                    description,
                } => {
                    let category = definition_category(&definition);
                    let (reverse_label, symmetric) = match definition {
                        Definition::Relationship {
                            reverse_label,
                            symmetric,
                        } => (reverse_label, symmetric),
                        _ => (None, false),
                    };
                    definitions.push(DefinitionSummary {
                        id: object_id,
                        label: version.record.label,
                        description,
                        category,
                        reverse_label,
                        symmetric,
                    });
                }
                ObjectData::Relationship {
                    subject,
                    target,
                    kind,
                    trust,
                    tags,
                } => relationships.push(RelationshipSummary {
                    cause: Self::explain(&store, context, version.object, version.record.source)?,
                    id: object_id,
                    subject_id: subject.id.0.to_string(),
                    target_id: target.id.0.to_string(),
                    kind,
                    trust,
                    tags,
                    starts_at: version.record.valid.start.map(|day| day.0),
                }),
                ObjectData::HistoricalEvent { occurrence } => {
                    events.insert(
                        version.object,
                        EventSummary {
                            id: object_id,
                            label: version.record.label,
                            day: occurrence.exact_day().map(|day| day.0),
                            citation: Self::citation(&store, context, version.record.source)?,
                            participants: Vec::new(),
                            actions: Vec::new(),
                        },
                    );
                }
                ObjectData::EventInvolvement {
                    event,
                    participant,
                    role,
                } => {
                    participants.push((
                        event,
                        ParticipantSummary {
                            person_id: participant.id.0.to_string(),
                            role: match role {
                                EventRole::Custom(object) => {
                                    store
                                        .get_object(object, context)
                                        .map_err(|error| error.to_string())?
                                        .ok_or("Event role definition is unavailable")?
                                        .record
                                        .label
                                }
                                _ => enum_label(&roles(), role).into(),
                            },
                        },
                    ));
                }
                ObjectData::ActorAction {
                    event,
                    actor,
                    target,
                    action,
                    intent,
                    outcome,
                } => {
                    actions.push((
                        event,
                        ActionSummary {
                            id: object_id,
                            person_id: actor.id.0.to_string(),
                            target_id: target.map(|item| item.id.0.to_string()),
                            action,
                            intent,
                            outcome: enum_label(&outcomes(), outcome),
                            citation: Self::citation(&store, context, version.record.source)?,
                        },
                    ));
                }
                _ => {}
            }
        }
        for (event, participant) in participants {
            if let Some(record) = events.get_mut(&event) {
                record.participants.push(participant);
            }
        }
        for (event, action) in actions {
            if let Some(record) = events.get_mut(&event) {
                record.actions.push(action);
            }
        }
        let mut events: Vec<_> = events.into_values().collect();
        events.sort_by(|a, b| {
            a.day
                .cmp(&b.day)
                .then(a.label.cmp(&b.label))
                .then(a.id.cmp(&b.id))
        });
        people.sort_by(|a, b| a.label.cmp(&b.label).then(a.id.cmp(&b.id)));
        relationships.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.id.cmp(&b.id)));
        definitions.sort_by(|a, b| a.category.cmp(b.category).then(a.label.cmp(&b.label)));
        let aliases = alias_versions
            .into_iter()
            .map(|alias| PersonAliasSummary {
                id: alias.id.to_string(),
                person_id: alias.person.id.0.to_string(),
                label: alias.label,
                starts_at: alias.starts_at.map(|day| day.0),
                ends_before: alias.ends_before.map(|day| day.0),
                active: alias.active,
                recorded_at: alias.recorded_at.0,
            })
            .collect();
        Ok(WorkspaceSnapshot {
            world: summary(world),
            view_revision: context.revision.0,
            day,
            scene_limited: false,
            people,
            relationships,
            events,
            definitions,
            aliases,
        })
    }

    fn explain(
        store: &Store,
        context: ViewContext,
        bond: ObjectRef,
        source: Option<ObjectRef>,
    ) -> Result<Option<CauseSummary>, String> {
        let Some(effect) = store
            .effect_on(bond, context)
            .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        let event = store
            .get_object(effect.event.object, context)
            .map_err(|error| error.to_string())?
            .ok_or("Source event is not available at this historical day")?;
        let effect_view = ViewContext {
            day: effect.day,
            ..context
        };
        let previous = store
            .get_object(
                effect.before.object,
                ViewContext {
                    revision: effect.before.recorded_at,
                    ..effect_view
                },
            )
            .map_err(|error| error.to_string())?
            .ok_or("Prior relationship state is unavailable")?;
        let after = store
            .get_object(
                bond,
                ViewContext {
                    revision: effect.after.recorded_at,
                    ..effect_view
                },
            )
            .map_err(|error| error.to_string())?
            .ok_or("Recorded relationship effect is unavailable")?;
        let ObjectData::Relationship {
            trust: before_trust,
            tags: before_tags,
            ..
        } = previous.record.data
        else {
            return Err("Prior state is not a relationship".into());
        };
        let ObjectData::Relationship {
            trust: after_trust,
            tags: after_tags,
            ..
        } = after.record.data
        else {
            return Err("Recorded effect is not a relationship".into());
        };
        let mut actor = None;
        let mut action = None;
        let mut intent = None;
        let mut outcome = None;
        if let Some(link) = effect.action {
            let version = store
                .get_object(
                    link.object,
                    ViewContext {
                        revision: link.recorded_at,
                        ..effect_view
                    },
                )
                .map_err(|error| error.to_string())?
                .ok_or("Source actor action is not available at this historical day")?;
            if let ObjectData::ActorAction {
                actor: actor_id,
                action: action_name,
                intent: action_intent,
                outcome: action_outcome,
                ..
            } = version.record.data
            {
                actor = Some(
                    store
                        .get_object(actor_id, effect_view)
                        .map_err(|error| error.to_string())?
                        .ok_or("Event actor is unavailable")?
                        .record
                        .label,
                );
                action = Some(action_name);
                intent = Some(action_intent);
                outcome = Some(enum_label(&outcomes(), action_outcome));
            } else {
                return Err("Effect references an object that is not an actor action".into());
            }
        }
        let source = Self::citation(
            store,
            ViewContext {
                revision: effect.after.recorded_at,
                ..effect_view
            },
            after.record.source.or(source),
        )?
        .unwrap_or_else(|| "No source cited".into());
        Ok(Some(CauseSummary {
            day: effect.day.0,
            event_id: event.object.id.0.to_string(),
            event: event.record.label,
            explanation: after.record.label,
            before_trust,
            before_tags,
            after_trust,
            after_tags,
            actor,
            action,
            intent,
            outcome,
            source,
        }))
    }

    fn citation(
        store: &Store,
        context: ViewContext,
        source: Option<ObjectRef>,
    ) -> Result<Option<String>, String> {
        let Some(source) = source else {
            return Ok(None);
        };
        let version = store
            .get_object(source, context)
            .map_err(|error| error.to_string())?
            .ok_or("Event source is unavailable")?;
        match version.record.data {
            ObjectData::Source { citation } => Ok(Some(citation)),
            _ => Err("Event evidence does not reference a source".into()),
        }
    }

    fn create_person(&self, world_id: &str, name: &str) -> Result<(), String> {
        let name = nonempty(name, "Person name")?;
        let mut store = self.lock()?;
        let world = store
            .world(WorldId(id(world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let person = ObjectRef::new(world.id);
        commit(
            &mut store,
            &world,
            vec![Change::Create(
                person,
                Record {
                    label: name,
                    valid: ValidInterval::default(),
                    data: ObjectData::Person,
                    source: None,
                },
            )],
        )
    }

    fn create_relationship(&self, request: RelationshipInput) -> Result<(), String> {
        let kind = nonempty(&request.kind, "Relationship kind")?;
        let mut store = self.lock()?;
        let world = store
            .world(WorldId(id(&request.world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let subject = reference(world.id, &request.subject_id)?;
        let target = reference(world.id, &request.target_id)?;
        commit(
            &mut store,
            &world,
            vec![Change::Create(
                ObjectRef::new(world.id),
                Record {
                    label: format!("{kind}: {} → {}", subject.id.0, target.id.0),
                    valid: ValidInterval {
                        start: Some(WorldDay(request.day)),
                        end_exclusive: None,
                    },
                    data: ObjectData::Relationship {
                        subject,
                        target,
                        kind,
                        trust: request.trust,
                        tags: tags(&request.tags),
                    },
                    source: None,
                },
            )],
        )
    }

    fn relationship_state(
        store: &Store,
        world: &World,
        relationship_id: &str,
        day: i64,
        trust: Option<u8>,
        tags_input: &str,
    ) -> Result<(ObjectRef, Record), String> {
        let relationship = reference(world.id, relationship_id)?;
        let mut record = store
            .get_object(relationship, view(world, day))
            .map_err(|error| error.to_string())?
            .ok_or("Relationship does not exist at that date")?
            .record;
        let ObjectData::Relationship {
            trust: old_trust,
            tags: old_tags,
            ..
        } = &mut record.data
        else {
            return Err("Selected object is not a relationship".into());
        };
        if record
            .valid
            .start
            .is_some_and(|start| start >= WorldDay(day))
        {
            return Err("Choose a day later than the relationship's current state".into());
        }
        *old_trust = trust;
        *old_tags = tags(tags_input);
        record.valid.start = Some(WorldDay(day));
        record.source = None;
        Ok((relationship, record))
    }

    fn edit_world(store: &Store, world_id: &str, expected: i64) -> Result<World, String> {
        let world = store
            .world(WorldId(id(world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        if world.revision.0 != expected {
            return Err("The world changed. Refresh the view before saving.".into());
        }
        Ok(world)
    }

    fn record_relationship_state(&self, request: RelationshipStateInput) -> Result<(), String> {
        let mut store = self.lock()?;
        let world = Self::edit_world(&store, &request.world_id, request.expected_revision)?;
        let (relationship, record) = Self::relationship_state(
            &store,
            &world,
            &request.relationship_id,
            request.day,
            request.trust,
            &request.tags,
        )?;
        commit(
            &mut store,
            &world,
            vec![Change::RecordState(relationship, record)],
        )
    }

    fn save_definition(&self, input: DefinitionInput) -> Result<(), String> {
        let label = nonempty(&input.label, "Definition name")?;
        let description = nonempty(&input.description, "Definition description")?;
        let definition = match input.category.as_str() {
            "relationship" => Definition::Relationship {
                reverse_label: input
                    .reverse_label
                    .as_deref()
                    .map(|raw| nonempty(raw, "Inverse label"))
                    .transpose()?,
                symmetric: input.symmetric,
            },
            "tag" if !label.contains(',') => Definition::Tag,
            "role"
                if !roles()
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case(&label)) =>
            {
                Definition::Role
            }
            _ => return Err("Invalid definition category or reserved name".into()),
        };
        let mut store = self.lock()?;
        let world = Self::edit_world(&store, &input.world_id, input.expected_revision)?;
        let catalog = store
            .objects_at(view(&world, 0), 2_000)
            .map_err(|error| error.to_string())?;
        let existing = catalog.into_iter().find(|item| {
            matches!(&item.record.data, ObjectData::Definition { definition, .. }
                if definition_category(definition) == input.category)
                && item.record.label.to_lowercase() == label.to_lowercase()
        });
        let record = Record {
            label: existing
                .as_ref()
                .map(|item| item.record.label.clone())
                .unwrap_or(label),
            valid: ValidInterval::default(),
            data: ObjectData::Definition {
                definition,
                description,
            },
            source: None,
        };
        let change = match existing {
            Some(item) if item.record == record => {
                return Err("Definition is already up to date".into());
            }
            Some(item) => Change::Correct(item.object, record),
            None => Change::Create(ObjectRef::new(world.id), record),
        };
        commit(&mut store, &world, vec![change])
    }

    fn record_evidence(&self, input: EvidenceInput) -> Result<(), String> {
        let statement = nonempty(&input.statement, "Statement")?;
        let citation = nonempty(&input.citation, "Citation")?;
        let mut store = self.lock()?;
        let world = Self::edit_world(&store, &input.world_id, input.expected_revision)?;
        let event = reference(world.id, &input.event_id)?;
        if !store
            .get_object(event, view(&world, input.day))
            .map_err(|error| error.to_string())?
            .is_some_and(|item| matches!(item.record.data, ObjectData::HistoricalEvent { .. }))
        {
            return Err("Select an event visible on this day".into());
        }
        let audience = match input.audience.as_str() {
            "author" => ClaimAudience::Author,
            "public" => ClaimAudience::Public,
            raw => ClaimAudience::Character(reference(world.id, raw)?),
        };
        let source = ObjectRef::new(world.id);
        let claim = ObjectRef::new(world.id);
        commit(
            &mut store,
            &world,
            vec![
                Change::Create(
                    source,
                    Record {
                        label: "Statement source".into(),
                        valid: ValidInterval::default(),
                        data: ObjectData::Source { citation },
                        source: None,
                    },
                ),
                Change::Create(
                    claim,
                    Record {
                        label: "Reported account".into(),
                        valid: ValidInterval {
                            start: Some(WorldDay(input.day)),
                            end_exclusive: None,
                        },
                        data: ObjectData::Claim {
                            subject: event,
                            predicate: "reported_account".into(),
                            statement,
                            audience,
                        },
                        source: Some(source),
                    },
                ),
            ],
        )
    }

    fn evidence(
        &self,
        world_id: &str,
        event_id: &str,
        day: i64,
        revision: Option<i64>,
        audience: &str,
    ) -> Result<Vec<EvidenceSummary>, String> {
        let store = self.lock()?;
        let world = store
            .world(WorldId(id(world_id)?), Scope::Author)
            .map_err(|error| error.to_string())?;
        let scope = match audience {
            "author" => Scope::Author,
            "public" => Scope::Public,
            raw => Scope::Character(reference(world.id, raw)?),
        };
        let context = ViewContext {
            revision: AuthorRevision(revision.unwrap_or(world.revision.0)),
            scope,
            ..view(&world, day)
        };
        store
            .evidence_at(reference(world.id, event_id)?, context)
            .map(|items| {
                items
                    .into_iter()
                    .map(|item| {
                        let (audience, reader_id) = match item.audience {
                            Some(ClaimAudience::Author) => (Some("Author only".into()), None),
                            Some(ClaimAudience::Public) => (Some("Public".into()), None),
                            Some(ClaimAudience::Character(reader)) => {
                                (Some("Character".into()), Some(reader.id.0.to_string()))
                            }
                            None => (None, None),
                        };
                        EvidenceSummary {
                            statement: item.statement,
                            source: item.source,
                            audience,
                            reader_id,
                        }
                    })
                    .collect()
            })
            .map_err(|error| error.to_string())
    }

    fn record_event(&self, request: EventInput) -> Result<String, String> {
        let event_label = nonempty(&request.event_label, "Event name")?;
        let citation = nonempty(&request.source_citation, "Source citation")?;
        let mut store = self.lock()?;
        let world = Self::edit_world(&store, &request.world_id, request.expected_revision)?;
        let editing = request.event_id.is_some();
        let event = request
            .event_id
            .as_deref()
            .map(|raw| reference(world.id, raw))
            .transpose()?
            .unwrap_or_else(|| ObjectRef::new(world.id));
        let existing = if editing {
            Some(
                store
                    .get_object(event, view(&world, request.day))
                    .map_err(|error| error.to_string())?
                    .ok_or("Event does not exist at this date")?
                    .record,
            )
        } else {
            None
        };
        let day = match existing.as_ref().map(|record| &record.data) {
            Some(ObjectData::HistoricalEvent { occurrence }) => occurrence
                .exact_day()
                .ok_or("Only dated events can be edited here")?,
            Some(_) => return Err("Selected object is not an event".into()),
            None => WorldDay(request.day),
        };
        let context = view(&world, day.0);
        let previous_source = existing.as_ref().and_then(|record| record.source);
        let mut changes = Vec::new();
        let source = match previous_source {
            Some(source)
                if Self::citation(&store, context, Some(source))?.as_deref() == Some(&citation) =>
            {
                source
            }
            _ => {
                let source = ObjectRef::new(world.id);
                changes.push(Change::Create(
                    source,
                    Record {
                        label: format!("{event_label} source"),
                        valid: ValidInterval::default(),
                        data: ObjectData::Source { citation },
                        source: None,
                    },
                ));
                source
            }
        };
        let sourced = |label: String, data| Record {
            label,
            valid: ValidInterval {
                start: Some(day),
                end_exclusive: None,
            },
            data,
            source: Some(source),
        };
        match existing {
            Some(mut record) => {
                if record.label != event_label || record.source != Some(source) {
                    record.label = event_label;
                    record.source = Some(source);
                    changes.push(Change::Correct(event, record));
                }
            }
            None => changes.push(Change::Create(
                event,
                sourced(
                    event_label,
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(day),
                    },
                ),
            )),
        }

        let mut roles_seen = HashSet::new();
        if editing {
            for version in store
                .objects_at(context, 2_000)
                .map_err(|error| error.to_string())?
            {
                if let ObjectData::EventInvolvement {
                    event: own_event,
                    participant,
                    role,
                } = version.record.data
                    && own_event == event
                {
                    roles_seen.insert((participant, role));
                }
            }
        }
        let mut action_ids = Vec::new();
        for row in request.participants {
            let person = reference(world.id, &row.person_id)?;
            let (role, role_label) = if let Some(raw) = row.role.strip_prefix("definition:") {
                let object = reference(world.id, raw)?;
                let version = store
                    .get_object(object, context)
                    .map_err(|error| error.to_string())?
                    .ok_or("Unknown event role definition")?;
                (EventRole::Custom(object), version.record.label)
            } else {
                let role = enum_value(&roles(), &row.role).ok_or("Invalid event role")?;
                (role, enum_label(&roles(), role).into())
            };
            let new_role = roles_seen.insert((person, role));
            if new_role {
                changes.push(Change::Create(
                    ObjectRef::new(world.id),
                    sourced(
                        role_label,
                        ObjectData::EventInvolvement {
                            event,
                            participant: person,
                            role,
                        },
                    ),
                ));
            }
            let mut action_id = None;
            match (role, row.action) {
                (EventRole::Actor, Some(input)) => {
                    let action = nonempty(&input.action, "Actor action")?;
                    let intent = nonempty(&input.intent, "Actor intent")?;
                    let outcome =
                        enum_value(&outcomes(), &input.outcome).ok_or("Invalid action outcome")?;
                    let target = input
                        .target_id
                        .as_deref()
                        .map(|raw| reference(world.id, raw))
                        .transpose()?;
                    if let Some(target) = target {
                        let version = store
                            .get_object(target, context)
                            .map_err(|error| error.to_string())?;
                        if !version.is_some_and(|version| {
                            matches!(version.record.data, ObjectData::Person)
                        }) {
                            return Err("Action target must be a person in this world".into());
                        }
                    }
                    let data = ObjectData::ActorAction {
                        event,
                        actor: person,
                        target,
                        action: action.clone(),
                        intent,
                        outcome,
                    };
                    let object = if let Some(action_id) = input.id {
                        if !editing {
                            return Err("Action corrections require an existing event".into());
                        }
                        let object = reference(world.id, &action_id)?;
                        let mut prior = store
                            .get_object(object, context)
                            .map_err(|error| error.to_string())?
                            .ok_or("Action does not exist at this event date")?
                            .record;
                        if !matches!(prior.data, ObjectData::ActorAction { event: owner, actor, .. }
                            if owner == event && actor == person)
                        {
                            return Err("Action belongs to a different event or actor".into());
                        }
                        if prior.data != data {
                            prior.data = data;
                            prior.label = action;
                            prior.source = Some(source);
                            changes.push(Change::Correct(object, prior));
                        }
                        object
                    } else {
                        let object = ObjectRef::new(world.id);
                        changes.push(Change::Create(object, sourced(action, data)));
                        object
                    };
                    action_id = Some(object);
                }
                (EventRole::Actor, None) => return Err("Every actor row needs an action".into()),
                (_, Some(_)) => return Err("Choose the actor role to record an action".into()),
                (_, None) if !new_role => {
                    return Err("That person already has this event role".into());
                }
                (_, None) => {}
            }
            action_ids.push(action_id);
        }
        if let Some(effect) = request.effect {
            let action = action_ids
                .get(effect.actor_row)
                .copied()
                .flatten()
                .ok_or("Choose the specific actor action that caused the change")?;
            let (target, mut record) = Self::relationship_state(
                &store,
                &world,
                &effect.relationship_id,
                day.0,
                effect.trust,
                &effect.tags,
            )?;
            record.label = nonempty(&effect.explanation, "Effect explanation")?;
            record.source = Some(source);
            changes.push(Change::ActionEffect {
                event,
                action,
                target,
                record,
            });
        }
        if changes.is_empty() {
            return Err("Change the event details or add a role or action before saving".into());
        }
        commit(&mut store, &world, changes)?;
        Ok(event.id.0.to_string())
    }
}

#[tauri::command]
pub fn workspace_worlds(
    workspace: State<'_, AuthorWorkspace>,
) -> Result<Vec<WorldSummary>, String> {
    workspace.worlds()
}

#[tauri::command]
pub fn workspace_create_world(
    workspace: State<'_, AuthorWorkspace>,
    name: String,
) -> Result<WorldSummary, String> {
    workspace.create_world(&name)
}

#[tauri::command]
pub fn workspace_snapshot(
    workspace: State<'_, AuthorWorkspace>,
    world_id: String,
    day: i64,
    revision: Option<i64>,
) -> Result<WorkspaceSnapshot, String> {
    workspace.snapshot(&world_id, day, revision)
}

async fn run_query_job(
    app: AppHandle,
    request_id: String,
    execute: impl FnOnce(&AuthorWorkspace, &QueryControl) -> Result<SearchSummary, String>
    + Send
    + 'static,
) -> Result<SearchSummary, String> {
    let request = id(&request_id)?;
    let control = app.state::<AuthorWorkspace>().begin_query(request)?;
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = app.state::<AuthorWorkspace>();
        let result = execute(&workspace, &control);
        workspace.finish_query(request)?;
        control.check().map_err(|error| error.to_string())?;
        result
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn workspace_cancel_query(
    workspace: State<'_, AuthorWorkspace>,
    request_id: String,
) -> Result<(), String> {
    workspace.cancel_query(id(&request_id)?)
}

#[tauri::command]
#[allow(
    clippy::too_many_arguments,
    reason = "Keep the named Tauri IPC contract stable; the request ID is independent of query data"
)]
pub async fn workspace_search(
    app: AppHandle,
    request_id: String,
    world_id: String,
    day: i64,
    revision: Option<i64>,
    mode: String,
    term: String,
    limit: usize,
) -> Result<SearchSummary, String> {
    run_query_job(app, request_id, move |workspace, control| {
        workspace.search(&world_id, day, revision, (&mode, &term), limit, control)
    })
    .await
}

#[tauri::command]
pub async fn workspace_next_search_page(
    app: AppHandle,
    request_id: String,
    world_id: String,
    day: i64,
    revision: i64,
    token: String,
) -> Result<SearchSummary, String> {
    run_query_job(app, request_id, move |workspace, control| {
        workspace.next_page(&world_id, day, revision, &token, control)
    })
    .await
}

#[tauri::command]
pub fn workspace_save_person_alias(
    workspace: State<'_, AuthorWorkspace>,
    request: PersonAliasInput,
) -> Result<String, String> {
    workspace.save_person_alias(request)
}

#[tauri::command]
pub fn workspace_review_mention(
    workspace: State<'_, AuthorWorkspace>,
    request: ReviewMentionInput,
) -> Result<i64, String> {
    workspace.review_mention(request)
}

#[tauri::command]
pub fn workspace_discover_mentions(
    workspace: State<'_, AuthorWorkspace>,
    world_id: String,
    day: i64,
    revision: Option<i64>,
    limit: usize,
) -> Result<MentionDiscoverySummary, String> {
    workspace.discover_mentions(&world_id, day, revision, limit)
}

#[tauri::command]
pub async fn workspace_composite_query(
    app: AppHandle,
    request_id: String,
    request: CompositeQueryInput,
) -> Result<SearchSummary, String> {
    run_query_job(app, request_id, move |workspace, control| {
        workspace.composite_query(request, control)
    })
    .await
}

#[tauri::command]
pub async fn workspace_query_participant_events(
    app: AppHandle,
    request_id: String,
    request: ParticipantEventsInput,
) -> Result<SearchSummary, String> {
    run_query_job(app, request_id, move |workspace, control| {
        workspace.participant_events(request, control)
    })
    .await
}

#[tauri::command]
pub async fn workspace_query_path(
    app: AppHandle,
    request_id: String,
    request: PathQueryInput,
) -> Result<SearchSummary, String> {
    run_query_job(app, request_id, move |workspace, control| {
        workspace.query_path(request, control)
    })
    .await
}

#[tauri::command]
pub fn workspace_saved_queries(
    workspace: State<'_, AuthorWorkspace>,
    world_id: String,
    revision: Option<i64>,
) -> Result<Vec<SavedQuerySummary>, String> {
    workspace.saved_queries(&world_id, revision)
}

#[tauri::command]
pub async fn workspace_replay_saved_query(
    app: AppHandle,
    request_id: String,
    request: ReplaySavedQueryInput,
) -> Result<SearchSummary, String> {
    run_query_job(app, request_id, move |workspace, control| {
        workspace.replay_saved_query(request, control)
    })
    .await
}

#[tauri::command]
pub fn workspace_save_query(
    workspace: State<'_, AuthorWorkspace>,
    request: SavedQueryInput,
) -> Result<String, String> {
    workspace.save_query(request)
}

#[tauri::command]
pub fn workspace_create_person(
    workspace: State<'_, AuthorWorkspace>,
    world_id: String,
    name: String,
) -> Result<(), String> {
    workspace.create_person(&world_id, &name)
}

#[tauri::command]
pub fn workspace_create_relationship(
    workspace: State<'_, AuthorWorkspace>,
    request: RelationshipInput,
) -> Result<(), String> {
    workspace.create_relationship(request)
}

#[tauri::command]
pub fn workspace_record_relationship_state(
    workspace: State<'_, AuthorWorkspace>,
    request: RelationshipStateInput,
) -> Result<(), String> {
    workspace.record_relationship_state(request)
}

#[tauri::command]
pub fn workspace_record_event(
    workspace: State<'_, AuthorWorkspace>,
    request: EventInput,
) -> Result<String, String> {
    workspace.record_event(request)
}

#[tauri::command]
pub fn workspace_save_definition(
    workspace: State<'_, AuthorWorkspace>,
    request: DefinitionInput,
) -> Result<(), String> {
    workspace.save_definition(request)
}

#[tauri::command]
pub fn workspace_record_evidence(
    workspace: State<'_, AuthorWorkspace>,
    request: EvidenceInput,
) -> Result<(), String> {
    workspace.record_evidence(request)
}

#[tauri::command]
pub fn workspace_evidence(
    workspace: State<'_, AuthorWorkspace>,
    world_id: String,
    event_id: String,
    day: i64,
    revision: Option<i64>,
    audience: String,
) -> Result<Vec<EvidenceSummary>, String> {
    workspace.evidence(&world_id, &event_id, day, revision, &audience)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_control() -> QueryControl {
        QueryControl::new(Duration::from_secs(10)).expect("test query timeout")
    }

    #[test]
    fn query_jobs_cancel_exact_identity_without_storage_lock()
    -> Result<(), Box<dyn std::error::Error>> {
        let workspace = AuthorWorkspace::open(":memory:")?;
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let ca = workspace.begin_query(a)?;
        let cb = workspace.begin_query(b)?;
        let lock = workspace.lock()?;
        workspace.cancel_query(a)?;
        assert!(matches!(
            ca.check(),
            Err(atlas_store::Error::QueryCancelled)
        ));
        cb.check()?;
        drop(lock);
        workspace.finish_query(a)?;
        workspace.finish_query(b)?;
        let queued = Uuid::new_v4();
        workspace.cancel_query(queued)?;
        assert!(workspace.begin_query(queued).is_err());
        Ok(())
    }

    #[test]
    fn oversized_scene_keeps_indexed_author_search_available_at_exact_revision()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let workspace = AuthorWorkspace::open(directory.path().join("large.sqlite"))?;
        let world = workspace.create_world("Large archive")?;
        let focus;
        {
            let mut store = workspace.lock()?;
            let current = store.world(WorldId(id(&world.id)?), Scope::Author)?;
            focus = ObjectRef::new(current.id);
            let person = |label: String| Record {
                label,
                valid: ValidInterval::default(),
                data: ObjectData::Person,
                source: None,
            };
            commit(
                &mut store,
                &current,
                vec![Change::Create(focus, person("Obsidian navigator".into()))],
            )?;
            let current = store.world(current.id, Scope::Author)?;
            let changes = (0..2_005)
                .map(|number| {
                    Change::Create(
                        ObjectRef::new(current.id),
                        person(format!("Inhabitant {number}")),
                    )
                })
                .collect();
            commit(&mut store, &current, changes)?;
        }
        let latest = workspace.snapshot(&world.id, 0, None)?;
        assert!(latest.scene_limited);
        assert!(latest.people.is_empty());
        assert!(latest.events.is_empty());
        assert_eq!(latest.view_revision, 2);
        let found = workspace.search(
            &world.id,
            0,
            Some(2),
            ("fulltext", "Obsidian"),
            10,
            &test_control(),
        )?;
        assert_eq!(found.revision, latest.view_revision);
        assert_eq!(found.rows.len(), 1);
        assert_eq!(found.rows[0].id, focus.id.0.to_string());
        let first = workspace.search(
            &world.id,
            0,
            Some(2),
            ("fulltext", "Inhabitant"),
            2,
            &test_control(),
        )?;
        assert_eq!(first.rows.len(), 2);
        let token = first.next_cursor.as_deref().unwrap();
        assert!(
            workspace
                .next_page(&world.id, 1, 2, token, &test_control())
                .is_err()
        );
        let second = workspace.next_page(&world.id, 0, 2, token, &test_control())?;
        assert_eq!(second.rows.len(), 2);
        assert_ne!(second.rows[0].id, first.rows[0].id);
        assert!(
            workspace
                .next_page(&world.id, 0, 2, token, &test_control())
                .is_err()
        );
        let next = second.next_cursor.as_deref().unwrap();
        let historical = workspace.snapshot(&world.id, 0, Some(1))?;
        assert!(
            workspace
                .next_page(&world.id, 0, 2, next, &test_control())
                .is_err()
        );
        assert!(!historical.scene_limited);
        assert_eq!(historical.people.len(), 1);
        assert!(workspace.snapshot(&world.id, 0, Some(3)).is_err());
        Ok(())
    }

    #[test]
    fn at_08_09_13_configured_roles_and_reports_preserve_outcomes_and_visibility()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("knowledge.sqlite");
        let workspace = AuthorWorkspace::open(&path)?;
        let world = workspace.create_world("Northbridge")?;
        let id = world.id;
        workspace.create_person(&id, "Alice")?;
        workspace.create_person(&id, "Beatrice")?;
        workspace.create_person(&id, "Morgan")?;
        let seed = workspace.snapshot(&id, 0, None)?;
        let alice = seed.people[0].id.clone();
        let morgan = seed.people[2].id.clone();
        let definition =
            |expected_revision, category: &str, label: &str, description: &str| DefinitionInput {
                world_id: id.clone(),
                expected_revision,
                category: category.into(),
                label: label.into(),
                description: description.into(),
                reverse_label: (category == "relationship").then(|| "sponsored by".into()),
                symmetric: false,
            };
        workspace.save_definition(definition(3, "role", "Mediator", "Facilitates negotiation"))?;
        workspace.save_definition(definition(4, "role", "Financier", "Funds the initiative"))?;
        workspace.save_definition(definition(5, "tag", "allied", "Shares policy goals"))?;
        workspace.save_definition(definition(
            6,
            "relationship",
            "sponsors",
            "Provides material support",
        ))?;
        let catalog = workspace.snapshot(&id, 0, None)?.definitions;
        let role = |label: &str| {
            format!(
                "definition:{}",
                catalog.iter().find(|item| item.label == label).unwrap().id
            )
        };
        let mut request = EventInput {
            world_id: id.clone(),
            expected_revision: 7,
            event_id: None,
            day: 10,
            event_label: "Council vote".into(),
            source_citation: "Official minutes".into(),
            participants: vec![
                actor(&alice, "Prevent disclosure", "failed"),
                ParticipantInput {
                    person_id: alice.clone(),
                    role: "victim".into(),
                    action: None,
                },
                ParticipantInput {
                    person_id: morgan.clone(),
                    role: role("Mediator"),
                    action: None,
                },
                ParticipantInput {
                    person_id: morgan.clone(),
                    role: role("Financier"),
                    action: None,
                },
            ],
            effect: None,
        };
        let label = request.participants[2].role.clone();
        request.participants[2].role = role("allied");
        assert!(workspace.record_event(request.clone()).is_err());
        assert!(workspace.snapshot(&id, 10, None)?.events.is_empty());
        request.participants[2].role = label;
        let event_id = workspace.record_event(request)?;
        workspace.save_definition(definition(8, "role", "Mediator", "Negotiates a settlement"))?;
        let revised = workspace.snapshot(&id, 10, None)?;
        let before = workspace.snapshot(&id, 10, Some(8))?;
        let mediator = |catalog: &[DefinitionSummary]| {
            let entry = catalog
                .iter()
                .find(|item| item.label == "Mediator")
                .unwrap();
            (entry.id.clone(), entry.description.clone())
        };
        assert_eq!(
            mediator(&before.definitions).0,
            mediator(&revised.definitions).0
        );
        assert_ne!(
            mediator(&before.definitions).1,
            mediator(&revised.definitions).1
        );
        for (index, (audience, statement)) in [
            ("author", "The sealed ledger identifies the sponsor."),
            (&alice, "Alice privately heard a claim of bribery."),
            (
                "public",
                "Rumor says the effort to prevent disclosure succeeded.",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            workspace.record_evidence(EvidenceInput {
                world_id: id.clone(),
                expected_revision: 9 + index as i64,
                event_id: event_id.clone(),
                day: 11,
                statement: statement.into(),
                citation: "PRIVATE source notes".into(),
                audience: audience.into(),
            })?;
        }
        drop(workspace);
        let workspace = AuthorWorkspace::open(&path)?;
        let author = workspace.evidence(&id, &event_id, 11, None, "author")?;
        assert_eq!(author.len(), 3);
        assert!(
            author
                .iter()
                .all(|item| item.source.as_deref() == Some("PRIVATE source notes"))
        );
        let public = workspace.evidence(&id, &event_id, 11, None, "public")?;
        assert_eq!(public.len(), 1);
        assert!(public[0].statement.contains("Rumor"));
        assert!(
            public[0].source.is_none()
                && public[0].audience.is_none()
                && public[0].reader_id.is_none()
        );
        let alice_view = workspace.evidence(&id, &event_id, 11, None, &alice)?;
        assert_eq!(alice_view.len(), 2);
        assert!(
            alice_view
                .iter()
                .any(|item| item.statement.starts_with("Alice privately"))
        );
        assert_eq!(
            workspace.evidence(&id, &event_id, 11, None, &morgan)?.len(),
            1
        );
        assert!(
            workspace
                .evidence(&id, &event_id, 10, None, "public")?
                .is_empty()
        );
        assert!(
            workspace
                .evidence(&id, &event_id, 11, Some(11), "public")
                .is_err()
        );
        let event = workspace.snapshot(&id, 11, None)?.events.pop().unwrap();
        assert_eq!(event.actions[0].outcome, "failed");
        let roles: HashSet<_> = event
            .participants
            .iter()
            .map(|item| item.role.as_str())
            .collect();
        assert_eq!(
            roles,
            HashSet::from(["actor", "victim", "Mediator", "Financier"])
        );
        let foreign = workspace.create_world("Other world")?;
        workspace.create_person(&foreign.id, "Unrelated")?;
        let foreign_id = workspace.snapshot(&foreign.id, 11, None)?.people[0]
            .id
            .clone();
        assert!(
            workspace
                .evidence(&id, &event_id, 11, None, &foreign_id)
                .is_err()
        );
        Ok(())
    }

    fn actor(person_id: &str, action: &str, outcome: &str) -> ParticipantInput {
        ParticipantInput {
            person_id: person_id.into(),
            role: "actor".into(),
            action: Some(ActionInput {
                id: None,
                action: action.into(),
                intent: "shape the succession dispute".into(),
                outcome: outcome.into(),
                target_id: None,
            }),
        }
    }

    #[test]
    fn author_workspace_reopens_with_event_attributed_relationship_history()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let database = directory.path().join("author.sqlite");
        let world_id;
        let alice;
        let beatrice;
        let relationship;
        {
            let workspace = AuthorWorkspace::open(&database)?;
            let world = workspace.create_world("Archive")?;
            world_id = world.id;
            workspace.create_person(&world_id, "Alice")?;
            workspace.create_person(&world_id, "Beatrice")?;
            let people = workspace.snapshot(&world_id, 0, None)?.people;
            alice = people
                .iter()
                .find(|p| p.label == "Alice")
                .unwrap()
                .id
                .clone();
            beatrice = people
                .iter()
                .find(|p| p.label == "Beatrice")
                .unwrap()
                .id
                .clone();
            workspace.create_relationship(RelationshipInput {
                world_id: world_id.clone(),
                subject_id: alice.clone(),
                target_id: beatrice.clone(),
                kind: "friendship".into(),
                trust: Some(90),
                tags: "friends".into(),
                day: 0,
            })?;
            relationship = workspace.snapshot(&world_id, 0, None)?.relationships[0]
                .id
                .clone();
            let mut event = EventInput {
                world_id: world_id.clone(),
                expected_revision: 3,
                event_id: None,
                day: 10,
                event_label: "Disclosed letter".into(),
                source_citation: "Northbridge folio 14".into(),
                participants: vec![
                    actor(&alice, "block disclosure", "failed"),
                    actor(&beatrice, "disclose letter", "succeeded"),
                    ParticipantInput {
                        person_id: alice.clone(),
                        role: "witness".into(),
                        action: None,
                    },
                ],
                effect: Some(EffectInput {
                    relationship_id: relationship.clone(),
                    actor_row: 2,
                    explanation: "Beatrice broke Alice's confidence".into(),
                    trust: Some(12),
                    tags: "estranged".into(),
                }),
            };
            assert!(workspace.record_event(event.clone()).is_err());
            let rolled_back = workspace.snapshot(&world_id, 10, None)?;
            assert_eq!(rolled_back.world.revision, 3);
            assert!(rolled_back.events.is_empty());
            event.effect.as_mut().unwrap().actor_row = 1;
            workspace.record_event(event.clone())?;
            assert!(
                workspace.record_event(event.clone()).is_err(),
                "stale form must fail"
            );
            event.expected_revision = 4;
            assert!(
                workspace.record_event(event).is_err(),
                "same-day second effect must roll back"
            );
            assert_eq!(
                workspace.snapshot(&world_id, 9, None)?.relationships[0].trust,
                Some(90)
            );
        }
        let reopened = AuthorWorkspace::open(&database)?;
        assert_eq!(reopened.worlds()?.len(), 1);
        let after = reopened.snapshot(&world_id, 10, None)?;
        assert_eq!(after.people.len(), 2);
        assert_eq!(after.relationships[0].id, relationship);
        assert_eq!(after.relationships[0].trust, Some(12));
        assert_eq!(after.relationships[0].tags, vec!["estranged"]);
        let cause = after.relationships[0].cause.as_ref().unwrap();
        assert_eq!(cause.event, "Disclosed letter");
        assert_eq!(cause.actor.as_deref(), Some("Beatrice"));
        assert_eq!(cause.outcome, Some("succeeded"));
        assert_eq!(cause.day, 10);
        assert_eq!(cause.source, "Northbridge folio 14");
        assert_eq!(cause.before_trust, Some(90));
        assert_eq!(cause.before_tags, vec!["friends"]);
        assert_eq!(cause.after_trust, Some(12));
        assert_eq!(cause.after_tags, vec!["estranged"]);
        assert_eq!(after.events.len(), 1);
        let event = &after.events[0];
        assert_eq!(cause.event_id, event.id);
        assert_eq!(event.participants.len(), 3);
        assert_eq!(event.actions.len(), 2);
        assert!(
            event
                .actions
                .iter()
                .any(|action| action.person_id == alice && action.outcome == "failed")
        );
        assert!(
            event
                .actions
                .iter()
                .any(|action| action.person_id == beatrice && action.outcome == "succeeded")
        );
        assert!(
            reopened.snapshot(&world_id, 9, None)?.relationships[0]
                .cause
                .is_none()
        );
        assert_eq!(
            reopened.snapshot(&world_id, 9, None)?.relationships[0].trust,
            Some(90)
        );
        assert_eq!(after.world.revision, 4);
        assert_eq!(after.view_revision, 4);
        let prior_revision = reopened.snapshot(&world_id, 10, Some(3))?;
        assert_eq!(prior_revision.view_revision, 3);
        assert_eq!(prior_revision.relationships[0].trust, Some(90));
        assert!(prior_revision.relationships[0].cause.is_none());
        assert!(prior_revision.events.is_empty());
        assert!(reopened.snapshot(&world_id, 10, Some(5)).is_err());
        let mut correction = actor(&beatrice, "published the letter", "succeeded");
        correction.action.as_mut().unwrap().id = event
            .actions
            .iter()
            .find(|action| action.person_id == beatrice)
            .map(|action| action.id.clone());
        reopened.record_event(EventInput {
            world_id: world_id.clone(),
            expected_revision: 4,
            event_id: Some(event.id.clone()),
            day: 15,
            event_label: "Published letter".into(),
            source_citation: "Revised folio 14".into(),
            participants: vec![
                correction,
                ParticipantInput {
                    person_id: beatrice.clone(),
                    role: "sponsor".into(),
                    action: None,
                },
            ],
            effect: None,
        })?;
        let corrected = reopened.snapshot(&world_id, 15, None)?;
        assert_eq!(corrected.events[0].id, event.id);
        assert_eq!(corrected.events[0].label, "Published letter");
        assert_eq!(corrected.events[0].day, Some(10));
        assert_eq!(corrected.events[0].participants.len(), 4);
        assert!(
            corrected.events[0]
                .actions
                .iter()
                .any(|action| action.action == "published the letter")
        );
        let original_cause = corrected.relationships[0].cause.as_ref().unwrap();
        assert_eq!(original_cause.action.as_deref(), Some("disclose letter"));
        assert_eq!(original_cause.source, "Northbridge folio 14");
        assert_eq!(
            reopened.snapshot(&world_id, 15, Some(4))?.events[0].label,
            "Disclosed letter"
        );
        reopened.record_relationship_state(RelationshipStateInput {
            world_id: world_id.clone(),
            expected_revision: 5,
            relationship_id: relationship,
            day: 16,
            trust: None,
            tags: "civil".into(),
        })?;
        let amended = reopened.snapshot(&world_id, 16, None)?;
        assert_eq!(amended.relationships[0].trust, None);
        assert_eq!(amended.relationships[0].tags, vec!["civil"]);
        assert!(amended.relationships[0].cause.is_none());
        assert!(
            reopened.snapshot(&world_id, 15, None)?.relationships[0]
                .cause
                .is_some()
        );
        Ok(())
    }
    #[test]
    fn phase_3_search_respects_author_revision_and_keeps_ambiguous_names()
    -> Result<(), Box<dyn std::error::Error>> {
        let workspace = AuthorWorkspace::open(":memory:")?;
        let world = workspace.create_world("Search fixture")?;
        workspace.create_person(&world.id, "Alice Vale")?;
        workspace.create_person(&world.id, "Alice Vale")?;
        workspace.create_person(&world.id, "Beatrice Thorne")?;
        let matches = workspace.search(
            &world.id,
            0,
            None,
            ("exact", " alice vale "),
            20,
            &test_control(),
        )?;
        assert_eq!(matches.revision, 3);
        assert_eq!(matches.rows.len(), 2);
        assert_ne!(matches.rows[0].id, matches.rows[1].id);
        assert!(matches.rows.iter().all(|row| row.kind == "person"));
        assert_eq!(
            workspace
                .search(
                    &world.id,
                    0,
                    Some(1),
                    ("contains", "alice"),
                    20,
                    &test_control()
                )?
                .rows
                .len(),
            1
        );
        assert_eq!(
            workspace
                .search(
                    &world.id,
                    0,
                    Some(0),
                    ("contains", "alice"),
                    20,
                    &test_control()
                )?
                .rows
                .len(),
            0
        );
        assert!(
            workspace
                .search(
                    &world.id,
                    0,
                    None,
                    ("unexpected", "alice"),
                    20,
                    &test_control()
                )
                .is_err()
        );
        assert!(
            workspace
                .search(&world.id, 0, None, ("exact", "  "), 20, &test_control())
                .is_err()
        );
        assert!(
            workspace
                .search(
                    &world.id,
                    0,
                    None,
                    ("contains", "alice"),
                    0,
                    &test_control()
                )
                .is_err()
        );
        let beatrice = workspace
            .snapshot(&world.id, 0, None)?
            .people
            .into_iter()
            .find(|person| person.label == "Beatrice Thorne")
            .unwrap()
            .id;
        workspace.create_relationship(RelationshipInput {
            world_id: world.id.clone(),
            subject_id: matches.rows[0].id.clone(),
            target_id: beatrice,
            kind: "friendship".into(),
            trust: None,
            tags: String::new(),
            day: 0,
        })?;
        let references = workspace.search(
            &world.id,
            0,
            None,
            ("backlinks", &matches.rows[0].id),
            20,
            &test_control(),
        )?;
        assert_eq!(references.rows.len(), 1);
        assert_eq!(references.rows[0].kind, "relationship");
        assert!(
            workspace
                .search(
                    &world.id,
                    0,
                    None,
                    ("backlinks", "not-a-uuid"),
                    20,
                    &test_control()
                )
                .is_err()
        );
        Ok(())
    }
    #[test]
    fn phase_3_typed_relationship_paths_are_dated_bounded_and_directional()
    -> Result<(), Box<dyn std::error::Error>> {
        let workspace = AuthorWorkspace::open(":memory:")?;
        let world = workspace.create_world("Path fixture")?;
        for name in ["Alice", "Beatrice", "Cara"] {
            workspace.create_person(&world.id, name)?;
        }
        let people = workspace.snapshot(&world.id, 0, None)?.people;
        let person = |name: &str| {
            people
                .iter()
                .find(|item| item.label == name)
                .unwrap()
                .id
                .clone()
        };
        for (from, to) in [("Alice", "Beatrice"), ("Beatrice", "Cara")] {
            workspace.create_relationship(RelationshipInput {
                world_id: world.id.clone(),
                subject_id: person(from),
                target_id: person(to),
                kind: "friendship".into(),
                trust: None,
                tags: String::new(),
                day: 10,
            })?;
        }
        let request = PathQueryInput {
            world_id: world.id.clone(),
            day: 10,
            revision: None,
            start_id: person("Alice"),
            kind: "friendship".into(),
            direction: "outgoing".into(),
            max_depth: 2,
            max_visited: 3,
            limit: 20,
        };
        let found = workspace.query_path(request.clone(), &test_control())?;
        assert_eq!(found.revision, 5);
        let ids: HashSet<_> = found.rows.iter().map(|row| row.id.clone()).collect();
        assert_eq!(ids, HashSet::from([person("Beatrice"), person("Cara")]));
        assert!(!found.truncated);

        let one_hop = workspace.query_path(
            PathQueryInput {
                max_depth: 1,
                ..request.clone()
            },
            &test_control(),
        )?;
        assert_eq!(one_hop.rows.len(), 1);
        let limited = workspace.query_path(
            PathQueryInput {
                max_visited: 2,
                ..request.clone()
            },
            &test_control(),
        )?;
        assert_eq!(limited.rows.len(), 1);
        assert!(limited.truncated);

        let earlier = workspace.query_path(
            PathQueryInput {
                day: 9,
                ..request.clone()
            },
            &test_control(),
        )?;
        assert!(earlier.rows.is_empty());
        let archived = workspace.query_path(
            PathQueryInput {
                revision: Some(3),
                ..request.clone()
            },
            &test_control(),
        )?;
        assert!(archived.rows.is_empty());
        let reverse = workspace.query_path(
            PathQueryInput {
                start_id: person("Cara"),
                direction: "incoming".into(),
                ..request.clone()
            },
            &test_control(),
        )?;
        assert_eq!(reverse.rows.len(), 2);

        let invalid_direction = workspace.query_path(
            PathQueryInput {
                direction: "sideways".into(),
                ..request.clone()
            },
            &test_control(),
        );
        assert!(invalid_direction.is_err());
        let invalid_depth = workspace.query_path(
            PathQueryInput {
                max_depth: 0,
                ..request.clone()
            },
            &test_control(),
        );
        assert!(invalid_depth.is_err());
        let invalid_visited = workspace.query_path(
            PathQueryInput {
                max_visited: 0,
                ..request.clone()
            },
            &test_control(),
        );
        assert!(invalid_visited.is_err());
        let invalid_start = workspace.query_path(
            PathQueryInput {
                start_id: "not-a-uuid".into(),
                ..request
            },
            &test_control(),
        );
        assert!(invalid_start.is_err());
        Ok(())
    }
}
