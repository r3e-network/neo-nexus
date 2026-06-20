use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::workspace_integrity::ForeignKeyViolation;

pub(in crate::workspace_integrity) fn pragma_texts(
    connection: &Connection,
    pragma: &str,
) -> Result<Vec<String>> {
    let mut statement = connection.prepare(&format!("PRAGMA {pragma}"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .with_context(|| format!("failed to read PRAGMA {pragma}"))
}

pub(in crate::workspace_integrity) fn pragma_u32(
    connection: &Connection,
    pragma: &str,
) -> Result<u32> {
    connection
        .query_row(&format!("PRAGMA {pragma}"), [], |row| row.get::<_, u32>(0))
        .with_context(|| format!("failed to read PRAGMA {pragma}"))
}

pub(in crate::workspace_integrity) fn pragma_u64(
    connection: &Connection,
    pragma: &str,
) -> Result<u64> {
    connection
        .query_row(&format!("PRAGMA {pragma}"), [], |row| row.get::<_, u64>(0))
        .with_context(|| format!("failed to read PRAGMA {pragma}"))
}

pub(in crate::workspace_integrity) fn foreign_key_violations(
    connection: &Connection,
) -> Result<Vec<ForeignKeyViolation>> {
    let mut statement = connection.prepare("PRAGMA foreign_key_check")?;
    let rows = statement.query_map([], |row| {
        Ok(ForeignKeyViolation {
            table: row.get(0)?,
            row_id: row.get(1)?,
            parent_table: row.get(2)?,
            foreign_key_index: row.get(3)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to read PRAGMA foreign_key_check")
}

pub(in crate::workspace_integrity) fn foreign_key_check_error(
    error: anyhow::Error,
) -> ForeignKeyViolation {
    ForeignKeyViolation {
        table: "pragma_foreign_key_check".to_string(),
        row_id: None,
        parent_table: error.to_string(),
        foreign_key_index: -1,
    }
}
