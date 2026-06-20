use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::repository::Repository;

use super::{
    discovery,
    schema::{WorkspaceBackup, WorkspaceBackupImport, WorkspaceBackupValidation},
};

mod apply;
mod reader;
mod summary;

pub struct WorkspaceBackupImporter;

impl WorkspaceBackupImporter {
    pub fn validate_path(path: impl AsRef<Path>) -> Result<WorkspaceBackupValidation> {
        let path = path.as_ref();
        let backup = Self::read(path)?;
        let mut validation = Self::validate(&backup)?;
        validation.source_path = Some(path.to_path_buf());
        Ok(validation)
    }

    pub fn import_path(
        repository: &Repository,
        path: impl AsRef<Path>,
    ) -> Result<WorkspaceBackupImport> {
        let path = path.as_ref();
        let backup = Self::read(path)?;
        let mut result = Self::import(repository, &backup)?;
        result.source_path = Some(path.to_path_buf());
        Ok(result)
    }

    pub fn read(path: impl AsRef<Path>) -> Result<WorkspaceBackup> {
        reader::read_backup(path.as_ref())
    }

    pub fn validate(backup: &WorkspaceBackup) -> Result<WorkspaceBackupValidation> {
        summary::validate_backup_summary(backup)
    }

    pub fn import(
        repository: &Repository,
        backup: &WorkspaceBackup,
    ) -> Result<WorkspaceBackupImport> {
        apply::import_backup(repository, backup)
    }

    pub fn latest_backup_path(input_dir: impl AsRef<Path>) -> Result<Option<PathBuf>> {
        discovery::latest_backup_path(input_dir)
    }
}
