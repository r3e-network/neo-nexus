use std::path::{Path, PathBuf};

use crate::app::domain::{
    NodeConfig, WorkspaceBackupImporter, WorkspaceBackupValidation, WorkspaceIntegrityReport,
};

mod backup_validation;
mod integrity;

use backup_validation::BackupValidationSummary;
use integrity::IntegritySummary;

pub(super) struct WorkspaceSafetySummary {
    pub(super) node_count: usize,
    pub(super) has_running_nodes: bool,
    pub(super) latest_backup: Option<PathBuf>,
    pub(super) backup_validation: BackupValidationSummary,
    pub(super) integrity: IntegritySummary,
}

impl WorkspaceSafetySummary {
    pub(super) fn new(
        nodes: &[NodeConfig],
        backup_export_dir: &Path,
        integrity_report: Option<&WorkspaceIntegrityReport>,
        backup_validation: Option<&WorkspaceBackupValidation>,
    ) -> Self {
        let latest_backup = WorkspaceBackupImporter::latest_backup_path(backup_export_dir)
            .ok()
            .flatten();
        Self {
            node_count: nodes.len(),
            has_running_nodes: nodes.iter().any(|node| node.status.is_running()),
            backup_validation: BackupValidationSummary::from_validation(
                latest_backup.as_deref(),
                backup_validation,
            ),
            latest_backup,
            integrity: IntegritySummary::from_report(integrity_report),
        }
    }

    pub(super) fn can_export(&self) -> bool {
        self.node_count > 0
    }

    pub(super) fn can_import(&self) -> bool {
        self.latest_backup.is_some() && !self.has_running_nodes && self.backup_validation.is_current
    }

    pub(super) fn can_validate_backup(&self) -> bool {
        self.latest_backup.is_some()
    }
}
