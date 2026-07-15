use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn export_workspace_backup(&mut self) {
        match WorkspaceBackupExporter::write(
            &self.repository,
            self.backup_export_dir(),
            env!("CARGO_PKG_VERSION"),
        ) {
            Ok(export) => {
                self.last_backup_validation = None;
                self.session.notice = Some(format!(
                    "Backup exported: {} nodes, {} plugin states, {} plugin packages, {} settings, {} runtime catalogs, {} signers, {} snapshots, {} events, {}",
                    export.node_count,
                    export.plugin_state_count,
                    export.plugin_installation_count,
                    export.workspace_setting_count,
                    export.runtime_catalog_profile_count,
                    export.runtime_signer_profile_count,
                    export.fast_sync_snapshot_count,
                    export.event_count,
                    short_path(&export.path, 46)
                ));
                self.record_event(
                    None,
                    None,
                    EventKind::BackupExported,
                    EventSeverity::Info,
                    format!("Workspace backup exported to {}", export.path.display()),
                );
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn validate_latest_workspace_backup(&mut self) {
        let latest_path =
            match WorkspaceBackupImporter::latest_backup_path(self.backup_export_dir()) {
                Ok(Some(path)) => path,
                Ok(None) => {
                    self.session.notice =
                        Some("No NeoNexus backup found in the backup directory".to_string());
                    return;
                }
                Err(error) => {
                    self.session.notice = Some(error.to_string());
                    return;
                }
            };

        match WorkspaceBackupImporter::validate_path(&latest_path) {
            Ok(validation) => {
                self.last_backup_validation = Some(validation.clone());
                self.session.notice = Some(format!(
                    "Backup validated: {} nodes, {} plugin states, {} plugin packages, {} settings, {} runtime catalogs, {} signers, {} snapshots, {} events, {}",
                    validation.node_count,
                    validation.plugin_state_count,
                    validation.plugin_installation_count,
                    validation.workspace_setting_count,
                    validation.runtime_catalog_profile_count,
                    validation.runtime_signer_profile_count,
                    validation.fast_sync_snapshot_count,
                    validation.event_count,
                    short_path(&latest_path, 46)
                ));
                self.record_event(
                    None,
                    None,
                    EventKind::BackupValidated,
                    EventSeverity::Info,
                    format!(
                        "Workspace backup validated from {}; {} nodes, {} events",
                        latest_path.display(),
                        validation.node_count,
                        validation.event_count
                    ),
                );
            }
            Err(error) => {
                self.last_backup_validation = None;
                self.session.notice = Some(error.to_string());
            }
        }
    }

    pub(in crate::app) fn import_latest_workspace_backup(&mut self) {
        if self.fleet.nodes.iter().any(|node| node.status.is_running()) {
            self.session.notice =
                Some("Stop running nodes before importing a workspace backup".to_string());
            return;
        }

        let latest_path =
            match WorkspaceBackupImporter::latest_backup_path(self.backup_export_dir()) {
                Ok(Some(path)) => path,
                Ok(None) => {
                    self.session.notice =
                        Some("No NeoNexus backup found in the backup directory".to_string());
                    return;
                }
                Err(error) => {
                    self.session.notice = Some(error.to_string());
                    return;
                }
            };

        if !self.latest_backup_validation_matches(&latest_path) {
            self.session.notice = Some("Validate latest workspace backup before importing".to_string());
            return;
        }

        match WorkspaceBackupImporter::import_path(&self.repository, &latest_path) {
            Ok(import) => {
                self.last_backup_validation = None;
                self.session.notice = Some(format!(
                    "Backup imported: {} created, {} updated, {} plugin states, {} plugin packages, {} settings, {} runtime catalogs, {} signers, {} snapshots, {} events",
                    import.created_nodes,
                    import.updated_nodes,
                    import.plugin_state_count,
                    import.plugin_installation_count,
                    import.workspace_setting_count,
                    import.runtime_catalog_profile_count,
                    import.runtime_signer_profile_count,
                    import.fast_sync_snapshot_count,
                    import.event_count
                ));
                self.record_event(
                    None,
                    None,
                    EventKind::BackupImported,
                    EventSeverity::Info,
                    format!(
                        "Workspace backup imported from {}; {} created, {} updated",
                        latest_path.display(),
                        import.created_nodes,
                        import.updated_nodes
                    ),
                );
                self.reload_nodes();
                self.reload_workspace_policies();
                self.reload_runtime_catalog_profiles();
                self.reload_runtime_signer_profiles();
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    fn latest_backup_validation_matches(&self, latest_path: &std::path::Path) -> bool {
        self.last_backup_validation
            .as_ref()
            .and_then(|validation| validation.source_path.as_deref())
            .is_some_and(|source_path| source_path == latest_path)
    }
}
