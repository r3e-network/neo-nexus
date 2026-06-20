use super::*;

impl Repository {
    pub fn list_workspace_settings_for_backup(&self) -> Result<Vec<WorkspaceSetting>> {
        let connection = self.connection()?;
        let mut settings = Vec::new();
        for key in WORKSPACE_BACKUP_SETTING_KEYS {
            if let Some(value) = load_setting(&connection, key)? {
                settings.push(WorkspaceSetting {
                    key: (*key).to_string(),
                    value,
                });
            }
        }
        Ok(settings)
    }

    pub fn restore_workspace_settings(&self, settings: &[WorkspaceSetting]) -> Result<usize> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        for setting in settings {
            validate_backup_setting_key(&setting.key)?;
            save_setting(&transaction, &setting.key, &setting.value)?;
        }
        transaction.commit()?;
        Ok(settings.len())
    }
}
