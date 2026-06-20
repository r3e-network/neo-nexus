use anyhow::Result;

use crate::repository::{validate_backup_setting_key, WorkspaceSetting};

use super::super::schema::WorkspaceSettingBackup;

pub(in crate::backup) fn restored_workspace_setting(
    backup: &WorkspaceSettingBackup,
) -> Result<WorkspaceSetting> {
    if backup.key.trim().is_empty() {
        anyhow::bail!("backup workspace setting key is required");
    }
    validate_backup_setting_key(&backup.key)?;
    Ok(WorkspaceSetting {
        key: backup.key.clone(),
        value: backup.value.clone(),
    })
}
