use anyhow::Result;

use crate::backup::schema::WorkspaceBackup;

pub(in crate::backup) fn validate_backup_header(backup: &WorkspaceBackup) -> Result<()> {
    if !matches!(backup.schema_version, 2..=7) {
        anyhow::bail!(
            "unsupported backup schema version {}; expected 2, 3, 4, 5, 6, or 7",
            backup.schema_version
        );
    }
    if backup.application != "NeoNexus" {
        anyhow::bail!("unsupported backup application {}", backup.application);
    }
    Ok(())
}
