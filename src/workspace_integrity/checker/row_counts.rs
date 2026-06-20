use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::workspace_integrity::{RequiredTableCheck, TableRowCount};

use super::sqlite;

pub(in crate::workspace_integrity) fn row_counts(
    connection: &Connection,
    table_checks: &[RequiredTableCheck],
) -> Result<Vec<TableRowCount>> {
    table_checks
        .iter()
        .filter(|table| table.present)
        .map(|table| {
            let rows = connection.query_row(
                &format!("SELECT COUNT(*) FROM {}", sqlite::identifier(&table.table)),
                [],
                |row| row.get::<_, u64>(0),
            )?;
            Ok(TableRowCount {
                table: table.table.clone(),
                rows,
            })
        })
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to count workspace table rows")
}
