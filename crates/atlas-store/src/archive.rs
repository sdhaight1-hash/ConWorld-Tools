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
    if schema >= 42 {
        crate::store::site_area::verify_site_area_integrity(&tx)?;
    }
    if schema >= 43 {
        crate::store::annotation::verify_annotation_integrity(&tx)?;
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

/// Preserve the committed old schema while the caller holds the migration
/// write reservation. A failed copy or verification prevents any migration.
pub(crate) fn preserve_before_migration(path: &Path, schema: i64) -> Result<(), Error> {
    let path = std::path::absolute(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| Error::InvalidBackupPath(path.clone()))?;
    let mut name = path
        .file_name()
        .ok_or_else(|| Error::InvalidBackupPath(path.clone()))?
        .to_os_string();
    name.push(format!(".pre-schema-{schema}-{}.sqlite", Uuid::new_v4()));
    let destination = parent.join(name);
    let temp = tempfile::NamedTempFile::new_in(parent)?.into_temp_path();
    let filename = temp
        .to_str()
        .ok_or_else(|| Error::InvalidBackupPath(temp.to_path_buf()))?;
    let source = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    source.execute("VACUUM INTO ?1", [filename])?;
    drop(source);

    if schema >= 3 {
        let manifest = inspect_archive(&temp, Scope::Author)?;
        if manifest.schema != schema {
            return Err(Error::Integrity("migration backup schema mismatch".into()));
        }
    } else {
        // Versions 1 and 2 predate the standalone archive contract. Preserve
        // their raw database snapshot without upgrading it or inventing history.
        let copy = Connection::open_with_flags(&temp, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let app_id: i64 = copy.pragma_query_value(None, "application_id", |row| row.get(0))?;
        let version: i64 = copy.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if app_id != APPLICATION_ID || version != schema {
            return Err(Error::WrongDatabase);
        }
        let integrity: String = copy.pragma_query_value(None, "quick_check", |row| row.get(0))?;
        if integrity != "ok" {
            return Err(Error::Integrity(integrity));
        }
        verify_foreign_keys(&copy)?;
    }
    std::fs::OpenOptions::new()
        .write(true)
        .open(&temp)?
        .sync_all()?;
    temp.persist_noclobber(&destination)
        .map_err(|error| error.error)?;
    Ok(())
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
