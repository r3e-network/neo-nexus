use super::*;

impl Repository {
    pub fn upsert_runtime_installation(&self, installation: &RuntimeInstallation) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO runtime_installations (
                package_id, label, node_type, version, os, arch, binary_path,
                sha256, signature_verified, signer_public_key, bytes, installed_at_unix
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(package_id) DO UPDATE SET
                label = excluded.label,
                node_type = excluded.node_type,
                version = excluded.version,
                os = excluded.os,
                arch = excluded.arch,
                binary_path = excluded.binary_path,
                sha256 = excluded.sha256,
                signature_verified = excluded.signature_verified,
                signer_public_key = excluded.signer_public_key,
                bytes = excluded.bytes,
                installed_at_unix = excluded.installed_at_unix",
            params![
                &installation.package_id,
                &installation.label,
                installation.node_type.to_string(),
                &installation.version,
                &installation.platform.os,
                &installation.platform.arch,
                installation.binary_path.to_string_lossy(),
                &installation.sha256,
                installation.signature_verified,
                &installation.signer_public_key,
                installation.bytes,
                installation.installed_at_unix,
            ],
        )?;
        Ok(())
    }

    pub fn list_runtime_installations(&self) -> Result<Vec<RuntimeInstallation>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT package_id, label, node_type, version, os, arch, binary_path,
                    sha256, signature_verified, signer_public_key, bytes, installed_at_unix
             FROM runtime_installations
             ORDER BY node_type ASC, version DESC, label COLLATE NOCASE ASC",
        )?;
        let rows = statement.query_map([], runtime_installation_from_row)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load runtime installations")
    }
}
