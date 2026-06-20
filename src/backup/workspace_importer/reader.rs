use std::{fs, path::Path};

use anyhow::{Context, Result};

use super::super::{schema::WorkspaceBackup, validation::validate_backup_header};

pub(in crate::backup) fn read_backup(path: &Path) -> Result<WorkspaceBackup> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read backup {}", path.display()))?;
    let backup: WorkspaceBackup = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse backup {}", path.display()))?;
    validate_backup_header(&backup)?;
    Ok(backup)
}
