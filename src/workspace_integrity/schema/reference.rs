//! What a correct workspace looks like, read from one.
//!
//! The expected schema used to be a second, hand-maintained declaration of the
//! first: a list of table names and their columns, kept beside the creation
//! statements and updated by remembering to. It had already drifted. The
//! integrity checker's required-table list omitted `node_signer_bindings` —
//! whose unique index is the anti-double-signing constraint — along with
//! `node_hermes_agents`, `node_runtime_quarantine` and `api_tokens`. A
//! workspace missing any of them passed.
//!
//! Two declarations of one schema will always drift, because only one of them
//! is exercised. So this one is not a declaration at all: a workspace is built
//! in memory by the production schema builder, its `sqlite_master` is read, and
//! *that* is the expectation. The creation statements are the single source,
//! and a table added without a matching entry here is impossible — there are no
//! entries.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use rusqlite::Connection;

/// The tables and indexes a workspace this build creates will have.
pub(in crate::workspace_integrity) struct ReferenceSchema {
    /// Table name → its columns.
    pub(in crate::workspace_integrity) tables: BTreeMap<String, BTreeSet<String>>,
    /// Table name → the indexes created on it.
    ///
    /// Only the named ones: SQLite invents `sqlite_autoindex_*` for primary
    /// keys and unique constraints, and those are a property of the column
    /// definitions rather than something a migration can forget.
    pub(in crate::workspace_integrity) indexes: BTreeMap<String, BTreeSet<String>>,
}

impl ReferenceSchema {
    /// Build a workspace in memory and read its shape.
    ///
    /// In memory rather than on disk: nothing is written anywhere, there is
    /// nothing to clean up, and the check cannot fail because a temporary
    /// directory was unavailable on the host it is diagnosing.
    pub(in crate::workspace_integrity) fn build() -> Result<Self> {
        let connection = Connection::open_in_memory()
            .context("failed to open an in-memory reference workspace")?;
        // The production schema builder, asked what it builds. That is the
        // whole point: there is no second list to keep in step.
        crate::repository::create_schema(&connection)
            .context("failed to create the reference schema to compare against")?;
        Self::read(&connection)
    }

    fn read(connection: &Connection) -> Result<Self> {
        let mut tables = BTreeMap::new();
        for name in object_names(connection, "table")? {
            // SQLite's own bookkeeping tables are not ours to require.
            if name.starts_with("sqlite_") {
                continue;
            }
            tables.insert(name.clone(), columns_of(connection, &name)?);
        }

        let mut indexes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut statement = connection.prepare(
            "SELECT name, tbl_name FROM sqlite_master
             WHERE type = 'index' AND name NOT LIKE 'sqlite_autoindex_%'",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (index, table) = row?;
            indexes.entry(table).or_default().insert(index);
        }

        anyhow::ensure!(
            tables.len() > 10,
            "the reference workspace has only {} tables; it did not initialise",
            tables.len()
        );
        Ok(Self { tables, indexes })
    }
}

fn object_names(connection: &Connection, kind: &str) -> Result<Vec<String>> {
    let mut statement =
        connection.prepare("SELECT name FROM sqlite_master WHERE type = ?1 ORDER BY name")?;
    let rows = statement.query_map([kind], |row| row.get::<_, String>(0))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .with_context(|| format!("failed to list {kind}s in the reference workspace"))
}

fn columns_of(connection: &Connection, table: &str) -> Result<BTreeSet<String>> {
    let mut statement = connection.prepare(&format!(
        "PRAGMA table_info({})",
        crate::workspace_integrity::checker::sqlite::identifier(table)
    ))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    rows.collect::<rusqlite::Result<BTreeSet<_>>>()
        .with_context(|| format!("failed to inspect reference table {table}"))
}

#[cfg(test)]
#[path = "../../../tests/unit/workspace_integrity/reference/tests.rs"]
mod tests;
