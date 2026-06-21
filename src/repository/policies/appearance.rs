use super::*;

impl Repository {
    /// Loads the persisted dark-mode preference, defaulting to light (`false`)
    /// for a fresh workspace or unreadable value.
    pub fn load_app_dark_mode(&self) -> Result<bool> {
        let connection = self.connection()?;
        Ok(load_setting(&connection, SETTING_APPEARANCE_DARK_MODE)?
            .as_deref()
            .is_some_and(parse_bool_setting))
    }

    pub fn save_app_dark_mode(&self, dark: bool) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(
            &transaction,
            SETTING_APPEARANCE_DARK_MODE,
            if dark { "true" } else { "false" },
        )?;
        transaction.commit()?;
        Ok(())
    }
}
