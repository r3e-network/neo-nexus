use anyhow::{bail, Result};
use rusqlite::Connection;

/// Returns true when `identifier` is a safe SQL identifier containing only
/// lowercase ASCII letters, digits, and underscores (e.g. `nodes`, `rpc_port`).
fn is_safe_identifier(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

pub(in crate::repository) fn add_column_if_missing(
    connection: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<()> {
    if !is_safe_identifier(table) {
        bail!("unsafe SQL table identifier rejected: {table:?}");
    }
    if !is_safe_identifier(column) {
        bail!("unsafe SQL column identifier rejected: {column:?}");
    }

    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    for existing in columns {
        if existing? == column {
            return Ok(());
        }
    }

    connection.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
        [],
    )?;
    Ok(())
}
