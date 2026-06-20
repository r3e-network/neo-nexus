use anyhow::Result;

use super::{
    counts::BackupValidationCounts, events::validate_backup_events, nodes::validate_backup_nodes,
    profiles::validate_backup_profiles,
};
use crate::backup::schema::WorkspaceBackup;

pub(in crate::backup) fn validate_backup_collections(
    backup: &WorkspaceBackup,
) -> Result<BackupValidationCounts> {
    validate_backup_profiles(backup)?;
    validate_backup_events(backup)?;
    validate_backup_nodes(backup)
}
