use std::path::Path;

use anyhow::Result;

use crate::repository::Repository;

mod clock;
mod snapshot;
mod summary;
mod writer;

use super::schema::{WorkspaceBackup, WorkspaceBackupExport};

pub struct WorkspaceBackupExporter;

impl WorkspaceBackupExporter {
    pub fn write(
        repository: &Repository,
        output_dir: impl AsRef<Path>,
        application_version: &str,
    ) -> Result<WorkspaceBackupExport> {
        let exported_at_unix = clock::current_unix_time()?;
        let backup = Self::snapshot(repository, application_version, exported_at_unix)?;
        let written = writer::write_backup_file(&backup, output_dir.as_ref(), exported_at_unix)?;
        Ok(summary::backup_export_summary(backup, written))
    }

    pub fn snapshot(
        repository: &Repository,
        application_version: &str,
        exported_at_unix: u64,
    ) -> Result<WorkspaceBackup> {
        snapshot::workspace_backup_snapshot(repository, application_version, exported_at_unix)
    }
}
