use super::*;

impl Repository {
    pub fn load_private_network_allow_external_sidecars(&self) -> Result<bool> {
        let connection = self.connection()?;
        Ok(
            load_setting(&connection, SETTING_PRIVATE_NETWORK_SIDECARS_ALLOW_EXTERNAL)?
                .as_deref()
                .is_some_and(parse_bool_setting),
        )
    }

    pub fn save_private_network_allow_external_sidecars(&self, allow_external: bool) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(
            &transaction,
            SETTING_PRIVATE_NETWORK_SIDECARS_ALLOW_EXTERNAL,
            if allow_external { "true" } else { "false" },
        )?;
        transaction.commit()?;
        Ok(())
    }
}
