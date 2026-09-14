//! Comparing a workspace against one this build would create.
//!
//! Not against a list. Two declarations of one schema drift, because only one
//! of them is exercised — and this one already had: the required-table list
//! omitted `node_signer_bindings`, whose unique index is the anti-double-signing
//! constraint, along with `node_hermes_agents`, `node_runtime_quarantine` and
//! `api_tokens`.

use std::collections::BTreeSet;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::workspace_integrity::{RequiredIndexCheck, RequiredTableCheck};

use super::super::schema::ReferenceSchema;
use super::sqlite;

pub(in crate::workspace_integrity) fn required_table_checks(
    connection: &Connection,
    reference: &ReferenceSchema,
) -> Result<Vec<RequiredTableCheck>> {
    reference
        .tables
        .iter()
        .map(|(table, expected)| {
            let columns = table_columns(connection, table)?;
            let missing_columns = expected
                .iter()
                .filter(|column| !columns.contains(*column))
                .cloned()
                .collect::<Vec<_>>();
            Ok(RequiredTableCheck {
                table: table.clone(),
                present: !columns.is_empty(),
                column_count: columns.len(),
                expected_column_count: expected.len(),
                missing_columns,
            })
        })
        .collect()
}

pub(in crate::workspace_integrity) fn required_index_checks(
    connection: &Connection,
    reference: &ReferenceSchema,
) -> Result<Vec<RequiredIndexCheck>> {
    let mut checks = Vec::new();
    for (table, expected) in &reference.indexes {
        let present = table_indexes(connection, table)?;
        for index in expected {
            checks.push(RequiredIndexCheck {
                table: table.clone(),
                index: index.clone(),
                present: present.contains(index),
            });
        }
    }
    Ok(checks)
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
