use super::*;

impl Repository {
    pub fn upsert_plugin_installation(&self, installation: &PluginInstallation) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO plugin_installations (
                node_id, plugin_id, installed_path, manifest_path, source_path,
                sha256, package_bytes, installed_files, expanded_bytes, installed_at_unix
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(node_id, plugin_id) DO UPDATE SET
                installed_path = excluded.installed_path,
                manifest_path = excluded.manifest_path,
                source_path = excluded.source_path,
                sha256 = excluded.sha256,
                package_bytes = excluded.package_bytes,
                installed_files = excluded.installed_files,
                expanded_bytes = excluded.expanded_bytes,
                installed_at_unix = excluded.installed_at_unix",
            params![
                &installation.node_id,
                installation.plugin_id.to_string(),
                installation.installed_path.to_string_lossy(),
                installation.manifest_path.to_string_lossy(),
                installation.source_path.to_string_lossy(),
                installation.sha256,
                installation.package_bytes,
                installation.installed_files as u64,
                installation.expanded_bytes,
                installation.installed_at_unix,
            ],
        )?;
        Ok(())
    }

    pub fn list_plugin_installations(&self, node_id: &str) -> Result<Vec<PluginInstallation>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT node_id, plugin_id, installed_path, manifest_path, source_path,
                    sha256, package_bytes, installed_files, expanded_bytes, installed_at_unix
             FROM plugin_installations
             WHERE node_id = ?1
             ORDER BY plugin_id ASC",
        )?;
        let rows = statement.query_map(params![node_id], plugin_installation_from_row)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load plugin installations")
    }

    pub fn replace_plugin_installations(
        &self,
        node_id: &str,
        installations: &[PluginInstallation],
    ) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "DELETE FROM plugin_installations WHERE node_id = ?1",
            params![node_id],
        )?;
        for installation in installations {
            transaction.execute(
                "INSERT INTO plugin_installations (
                    node_id, plugin_id, installed_path, manifest_path, source_path,
                    sha256, package_bytes, installed_files, expanded_bytes, installed_at_unix
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    node_id,
                    installation.plugin_id.to_string(),
                    installation.installed_path.to_string_lossy(),
                    installation.manifest_path.to_string_lossy(),
                    installation.source_path.to_string_lossy(),
                    installation.sha256,
                    installation.package_bytes,
                    installation.installed_files as u64,
                    installation.expanded_bytes,
                    installation.installed_at_unix,
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }
}
