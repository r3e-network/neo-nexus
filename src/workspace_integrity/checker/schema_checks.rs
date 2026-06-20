use std::collections::BTreeSet;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::workspace_integrity::{RequiredIndexCheck, RequiredTableCheck};

use super::super::schema::{required_tables, REQUIRED_INDEXES};
use super::sqlite;

pub(in crate::workspace_integrity) fn required_table_checks(
    connection: &Connection,
) -> Result<Vec<RequiredTableCheck>> {
    required_tables()
        .map(|required| {
            let columns = table_columns(connection, required.name)?;
            let missing_columns = required
                .columns
                .iter()
                .filter(|column| !columns.contains(**column))
                .map(|column| (*column).to_string())
                .collect::<Vec<_>>();
            Ok(RequiredTableCheck {
                table: required.name.to_string(),
                present: !columns.is_empty(),
                column_count: columns.len(),
                expected_column_count: required.columns.len(),
                missing_columns,
            })
        })
        .collect()
}

pub(in crate::workspace_integrity) fn required_index_checks(
    connection: &Connection,
) -> Result<Vec<RequiredIndexCheck>> {
    REQUIRED_INDEXES
        .iter()
        .map(|required| {
            let indexes = table_indexes(connection, required.table)?;
            Ok(RequiredIndexCheck {
                table: required.table.to_string(),
                index: required.name.to_string(),
                present: indexes.contains(required.name),
            })
        })
        .collect()
}

fn table_columns(connection: &Connection, table: &str) -> Result<BTreeSet<String>> {
    let mut statement =
        connection.prepare(&format!("PRAGMA table_info({})", sqlite::identifier(table)))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    let columns = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .with_context(|| format!("failed to inspect table {table}"))?;
    Ok(columns.into_iter().collect())
}

fn table_indexes(connection: &Connection, table: &str) -> Result<BTreeSet<String>> {
    let mut statement =
        connection.prepare(&format!("PRAGMA index_list({})", sqlite::identifier(table)))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    let indexes = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .with_context(|| format!("failed to inspect indexes for table {table}"))?;
    Ok(indexes.into_iter().collect())
}
