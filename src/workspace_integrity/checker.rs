mod clock;
mod connection;
mod pragmas;
mod row_counts;
mod schema_checks;
mod sqlite;

use anyhow::{Context, Result};
use std::{fs, path::Path};

use clock::current_unix_time;
use connection::open_read_only;
use pragmas::{
    foreign_key_check_error, foreign_key_violations, pragma_texts, pragma_u32, pragma_u64,
};
use row_counts::row_counts;
use schema_checks::{required_index_checks, required_table_checks};

use super::WorkspaceIntegrityReport;

pub struct WorkspaceIntegrityChecker;

impl WorkspaceIntegrityChecker {
    pub fn check(
        database: impl AsRef<Path>,
        application_version: impl Into<String>,
    ) -> Result<WorkspaceIntegrityReport> {
        Self::check_at(database, application_version, current_unix_time()?)
    }

    pub fn check_at(
        database: impl AsRef<Path>,
        application_version: impl Into<String>,
        checked_at_unix: u64,
    ) -> Result<WorkspaceIntegrityReport> {
        let database = database.as_ref();
        if !database.is_file() {
            anyhow::bail!(
                "workspace database {} does not exist; pass an existing neonexus.db",
                database.display()
            );
        }

        let metadata = fs::metadata(database)
            .with_context(|| format!("failed to inspect database {}", database.display()))?;
        let connection = open_read_only(database)?;
        let integrity_check = pragma_texts(&connection, "integrity_check")
            .unwrap_or_else(|error| vec![format!("integrity_check failed: {error}")]);
        let foreign_key_violations = foreign_key_violations(&connection)
            .unwrap_or_else(|error| vec![foreign_key_check_error(error)]);
        let required_tables = required_table_checks(&connection)?;
        let required_indexes = required_index_checks(&connection)?;
        let row_counts = row_counts(&connection, &required_tables)?;

        let mut report = WorkspaceIntegrityReport {
            schema_version: 1,
            status: "unknown".to_string(),
            application: "NeoNexus",
            application_version: application_version.into(),
            checked_at_unix,
            database: database.display().to_string(),
            database_bytes: metadata.len(),
            sqlite_user_version: pragma_u32(&connection, "user_version")?,
            sqlite_page_size: pragma_u64(&connection, "page_size")?,
            sqlite_page_count: pragma_u64(&connection, "page_count")?,
            sqlite_freelist_count: pragma_u64(&connection, "freelist_count")?,
            integrity_check,
            foreign_key_violations,
            required_tables,
            required_indexes,
            row_counts,
        };
        report.status = report.status_label().to_string();
        Ok(report)
    }
}
