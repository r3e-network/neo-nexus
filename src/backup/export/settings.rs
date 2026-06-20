use crate::repository::WorkspaceSetting;

use super::super::schema::WorkspaceSettingBackup;

pub(in crate::backup) fn workspace_setting_backup(
    setting: WorkspaceSetting,
) -> WorkspaceSettingBackup {
    WorkspaceSettingBackup {
        key: setting.key,
        value: setting.value,
    }
}
