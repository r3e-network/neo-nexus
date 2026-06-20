use std::path::Path;

use super::types::WorkspaceBackupImport;

impl WorkspaceBackupImport {
    pub fn to_cli_text(&self) -> String {
        self.to_cli_text_with_target(None)
    }

    pub fn to_cli_text_with_target(&self, target_database: Option<&Path>) -> String {
        let mut lines = vec!["backup-import: ok".to_string()];
        if let Some(path) = &self.source_path {
            lines.push(format!("source: {}", path.display()));
        }
        if let Some(path) = target_database {
            lines.push(format!("target-database: {}", path.display()));
        }
        lines.extend([
            format!("schema-version: {}", self.schema_version),
            format!("exported-at-unix: {}", self.exported_at_unix),
            format!("created-nodes: {}", self.created_nodes),
            format!("updated-nodes: {}", self.updated_nodes),
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
            String::new(),
        ]);
        lines.join("\n")
    }
}
