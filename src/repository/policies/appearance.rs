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

    /// Loads whether the right-hand inspector panel was left open, defaulting to
    /// hidden (`false`) for a fresh workspace or unreadable value.
    pub fn load_app_inspector_visible(&self) -> Result<bool> {
        let connection = self.connection()?;
        Ok(
            load_setting(&connection, SETTING_WORKSPACE_INSPECTOR_VISIBLE)?
                .as_deref()
                .is_some_and(parse_bool_setting),
        )
    }

    pub fn save_app_inspector_visible(&self, visible: bool) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(
            &transaction,
            SETTING_WORKSPACE_INSPECTOR_VISIBLE,
            if visible { "true" } else { "false" },
        )?;
        transaction.commit()?;
        Ok(())
    }

    /// Loads the persist key of the last-active workspace view, or `None` for a
    /// fresh workspace so the caller can fall back to the default landing page.
    pub fn load_workspace_last_view(&self) -> Result<Option<String>> {
        let connection = self.connection()?;
        load_setting(&connection, SETTING_WORKSPACE_LAST_VIEW)
    }

    pub fn save_workspace_last_view(&self, view_key: &str) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(&transaction, SETTING_WORKSPACE_LAST_VIEW, view_key)?;
        transaction.commit()?;
        Ok(())
    }
}
