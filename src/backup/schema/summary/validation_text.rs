use super::types::WorkspaceBackupValidation;

impl WorkspaceBackupValidation {
    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            "backup-validation: ok".to_string(),
            format!("schema-version: {}", self.schema_version),
            format!("application-version: {}", self.application_version),
            format!("exported-at-unix: {}", self.exported_at_unix),
            format!("nodes: {}", self.node_count),
            format!("plugin-states: {}", self.plugin_state_count),
            format!("plugin-installations: {}", self.plugin_installation_count),
            format!("workspace-settings: {}", self.workspace_setting_count),
            format!("remote-servers: {}", self.remote_server_count),
            format!(
                "runtime-catalog-profiles: {}",
                self.runtime_catalog_profile_count
            ),
            format!(
                "runtime-signer-profiles: {}",
                self.runtime_signer_profile_count
            ),
            format!("neo-wallet-profiles: {}", self.neo_wallet_profile_count),
            format!("fast-sync-snapshots: {}", self.fast_sync_snapshot_count),
            format!("events: {}", self.event_count),
        ];
        if let Some(path) = &self.source_path {
            lines.insert(1, format!("source: {}", path.display()));
        }
        lines.push(String::new());
        lines.join("\n")
    }
}
