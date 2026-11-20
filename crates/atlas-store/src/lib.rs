//! Local, author-scoped world persistence for Living History Atlas.
mod store;

use std::path::PathBuf;

pub use store::Store;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct WorldId(pub Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BranchId(pub Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ObjectId(pub Uuid);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AuthorRevision(pub i64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldDay(pub i64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ObjectRef {
    pub world: WorldId,
    pub id: ObjectId,
}

impl ObjectRef {
    pub fn new(world: WorldId) -> Self {
        Self {
            world,
            id: ObjectId(Uuid::new_v4()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    pub id: WorldId,
    pub name: String,
    pub main: BranchId,
    pub revision: AuthorRevision,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ValidInterval {
    pub start: Option<WorldDay>,
    pub end_exclusive: Option<WorldDay>,
}

/// An occurrence on the world's normalized time axis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventDate {
    Exact(WorldDay),
    Uncertain {
        earliest: Option<WorldDay>,
        latest: Option<WorldDay>,
        label: String,
    },
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Certainty {
    Definitely,
    Possibly,
    NotEstablished,
}

impl EventDate {
    /// Inclusive bounds describe possible occurrence days, independent of the record interval.
    pub fn certainty_on(&self, day: WorldDay) -> Certainty {
        match self {
            Self::Exact(exact) if *exact == day => Certainty::Definitely,
            Self::Uncertain {
                earliest, latest, ..
            } if *earliest == Some(day) && *latest == Some(day) => Certainty::Definitely,
            Self::Uncertain {
                earliest, latest, ..
            } if earliest.is_none_or(|start| start <= day)
                && latest.is_none_or(|end| day <= end) =>
            {
                Certainty::Possibly
            }
            _ => Certainty::NotEstablished,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectData {
    Person,
    HistoricalEvent { occurrence: EventDate },
    Source { citation: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub label: String,
    pub valid: ValidInterval,
    pub data: ObjectData,
    pub source: Option<ObjectRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version {
    pub object: ObjectRef,
    pub record: Record,
    pub recorded_at: AuthorRevision,
}

#[derive(Clone, Debug)]
pub enum Change {
    Create(ObjectRef, Record),
    /// Replace the newest historical state at a new author revision.
    Correct(ObjectRef, Record),
    /// Append a later historical state. Its start is required and closes a predecessor that overlaps it.
    RecordState(ObjectRef, Record),
}

/// Scope comes from the trusted local host, outside the stored world data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scope {
    Author,
    Public,
}

#[derive(Clone, Copy, Debug)]
pub struct ViewContext {
    pub world: WorldId,
    pub branch: BranchId,
    pub revision: AuthorRevision,
    pub day: WorldDay,
    pub scope: Scope,
}

#[derive(Clone, Debug)]
pub struct Command {
    pub world: WorldId,
    pub branch: BranchId,
    pub expected: AuthorRevision,
    pub id: Uuid,
    pub author: String,
    pub rationale: String,
    pub scope: Scope,
    pub changes: Vec<Change>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the requested operation requires the local author scope")]
    CapabilityDenied,
    #[error("the object reference belongs to a different world")]
    WrongWorld,
    #[error("the object is of a different kind")]
    WrongKind,
    #[error("world, branch, or object does not exist")]
    NotFound,
    #[error("an object with this identity already exists")]
    ObjectExists,
    #[error("a command may change an object only once")]
    RepeatedObject,
    #[error("the end of a valid interval must follow its start")]
    InvalidInterval,
    #[error("invalid uncertain occurrence bounds")]
    InvalidOccurrence,
    #[error("the requested author revision is outside the world's recorded history")]
    InvalidRevision,
    #[error("the command expected {expected:?}, but the world is at {actual:?}")]
    RevisionConflict {
        expected: AuthorRevision,
        actual: AuthorRevision,
    },
    #[error("the command already committed at {0:?}")]
    DuplicateCommand(AuthorRevision),
    #[error("the database schema version {0} is newer than this application supports")]
    SchemaTooNew(i64),
    #[error("the database is not a ConWorld world store")]
    WrongDatabase,
    #[error("database integrity violation in {0}")]
    Integrity(String),
    #[error("invalid input for {0}")]
    InvalidInput(&'static str),
    #[error("backup path is not valid UTF-8: {0:?}")]
    InvalidBackupPath(PathBuf),
    #[error("database operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("file operation failed: {0}")]
    Io(#[from] std::io::Error),
}
