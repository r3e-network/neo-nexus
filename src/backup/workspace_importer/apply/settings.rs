use anyhow::Result;

use crate::{
    backup::{restore::restored_workspace_setting, schema::WorkspaceBackup},
    repository::Repository,
};

pub(super) fn restore_workspace_settings(
    repository: &Repository,
    backup: &WorkspaceBackup,
) -> Result<usize> {
    let workspace_settings = backup
        .workspace_settings
        .iter()
        .map(restored_workspace_setting)
        .collect::<Result<Vec<_>>>()?;
    repository.restore_workspace_settings(&workspace_settings)
}
