use super::*;

impl Repository {
    pub fn load_setting(&self, key: &str) -> Result<Option<String>> {
        let connection = self.connection()?;
        load_setting(&connection, key)
    }

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

    /// Restore workspace settings from a backup archive with legacy compatibility.
    /// Old backups created before jitter_enabled exist may lack that key; we
    /// therefore ensure it is false on restore rather than inheriting the target
    /// current value. All other keys are upserted as-is; the number of rows
    /// written equals the length of the provided input slice.
    pub fn restore_workspace_settings(&self, settings: &[WorkspaceSetting]) -> Result<usize> {
        use std::collections::HashSet;
        let keys: HashSet<_> = settings.iter().map(|s| s.key.as_str()).collect();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        for setting in settings {
            validate_backup_setting_key(&setting.key)?;
            save_setting(&transaction, &setting.key, &setting.value)?;
        }
        // Ensure legacy default if missing from old backup
        if !keys.contains(SETTING_WATCHDOG_JITTER_ENABLED) {
            save_setting(&transaction, SETTING_WATCHDOG_JITTER_ENABLED, "false")?;
        }
        transaction.commit()?;
        Ok(settings.len())
    }
}
