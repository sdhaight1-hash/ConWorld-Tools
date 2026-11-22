//! Phase 9 standalone SQLite archive inspection, with no mutation or migration.
use std::path::Path;

use rusqlite::{Connection, OpenFlags};
use uuid::Uuid;

use crate::{
    AuthorRevision, Error, Scope, WorldId,
    store::{APPLICATION_ID, SCHEMA_VERSION, verify_foreign_keys},
};

pub const ARCHIVE_FORMAT_VERSION: u16 = 1;

#[derive(Debug, PartialEq, Eq)]
pub struct ArchiveManifest {
    pub format: u16,
    pub schema: i64,
    pub worlds: Vec<ArchiveWorld>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArchiveWorld {
    pub id: WorldId,
    pub name: String,
    pub revision: AuthorRevision,
}

/// Inspect an exported snapshot independently of the active store.
/// The archive schema must provide explicit author revision history.
pub fn inspect_archive(path: impl AsRef<Path>, scope: Scope) -> Result<ArchiveManifest, Error> {
    if scope != Scope::Author {
        return Err(Error::CapabilityDenied);
    }
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let tx = conn.unchecked_transaction()?;
    let app_id: i64 = tx.pragma_query_value(None, "application_id", |row| row.get(0))?;
    let schema: i64 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if app_id != APPLICATION_ID || schema < 3 {
        return Err(Error::WrongDatabase);
    }
    if schema > SCHEMA_VERSION {
        return Err(Error::SchemaTooNew(schema));
    }
    crate::store::verify_semantic_schema_lineage(&tx, schema)?;
    let integrity: String = tx.pragma_query_value(None, "quick_check", |row| row.get(0))?;
    if integrity != "ok" {
        return Err(Error::Integrity(integrity));
    }
    verify_foreign_keys(&tx)?;
    if schema >= 36 {
        crate::store::media::verify_media_hashes(&tx)?;
    }
    if schema >= 38 {
        crate::store::classification::verify_classification_integrity(&tx)?;
    }
    let mut statement = tx.prepare(
        "SELECT w.id, w.name, COALESCE(MAX(r.revision), 0)
         FROM worlds w LEFT JOIN author_revisions r ON r.world_id = w.id
         GROUP BY w.id, w.name ORDER BY w.id",
    )?;
    let worlds = statement
        .query_map([], |row| {
            Ok(ArchiveWorld {
                id: WorldId(Uuid::from_bytes(row.get(0)?)),
                name: row.get(1)?,
                revision: AuthorRevision(row.get(2)?),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ArchiveManifest {
        format: ARCHIVE_FORMAT_VERSION,
        schema,
        worlds,
    })
}

/// Restore to a new database beside the caller's chosen destination. Existing
/// worlds are never overwritten. Validate and migrate before publishing the copy.
pub fn restore_archive(
    archive: &Path,
    destination: &Path,
    scope: Scope,
) -> Result<ArchiveManifest, Error> {
    if scope != Scope::Author {
        return Err(Error::CapabilityDenied);
    }
    let destination = std::path::absolute(destination)?;
    let parent = destination
        .parent()
        .ok_or_else(|| Error::InvalidBackupPath(destination.clone()))?;
    let temp = tempfile::NamedTempFile::new_in(parent)?.into_temp_path();
    let filename = temp
        .to_str()
        .ok_or_else(|| Error::InvalidBackupPath(temp.to_path_buf()))?;
    let source = Connection::open_with_flags(archive, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    source.execute("VACUUM INTO ?1", [filename])?;
    drop(source);
    inspect_archive(&temp, scope)?;
    drop(crate::Store::open(&temp)?);
    let manifest = inspect_archive(&temp, scope)?;
    temp.persist_noclobber(&destination)
        .map_err(|error| error.error)?;
    Ok(manifest)
}
