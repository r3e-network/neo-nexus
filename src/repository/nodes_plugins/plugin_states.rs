use super::*;

impl Repository {
    pub fn set_plugin_enabled(
        &self,
        node_id: &str,
        plugin_id: PluginId,
        enabled: bool,
    ) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO plugin_states (node_id, plugin_id, enabled)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(node_id, plugin_id)
             DO UPDATE SET enabled = excluded.enabled",
            params![node_id, plugin_id.to_string(), enabled],
        )?;
        Ok(())
    }

    pub fn list_plugin_states(&self, node_id: &str) -> Result<Vec<PluginState>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT plugin_id, enabled
             FROM plugin_states
             WHERE node_id = ?1
             ORDER BY plugin_id ASC",
        )?;

        let rows = statement.query_map(params![node_id], |row| {
            let plugin_id_raw: String = row.get(0)?;
            Ok(PluginState {
                plugin_id: PluginId::from_str(&plugin_id_raw)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?,
                enabled: row.get(1)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load plugin state")
    }
}
