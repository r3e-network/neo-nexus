use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::WorkspaceBackup;

pub(super) struct WrittenBackup {
    pub(super) path: PathBuf,
    pub(super) bytes_written: usize,
}

pub(super) fn write_backup_file(
    backup: &WorkspaceBackup,
    output_dir: &Path,
    exported_at_unix: u64,
) -> Result<WrittenBackup> {
    let text = serde_json::to_string_pretty(backup).context("failed to render workspace backup")?;
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create backup directory {}", output_dir.display()))?;
    let path = output_dir.join(format!("neonexus-backup-{exported_at_unix}.json"));
    fs::write(&path, text.as_bytes())
        .with_context(|| format!("failed to write backup {}", path.display()))?;
    // The archive carries the whole fleet: every node's command line, signer
    // lease, wallet binding and duty. It was landing at the umask default,
    // which on most hosts means any local user could read it.
    restrict_to_owner(&path)?;

    Ok(WrittenBackup {
        path,
        bytes_written: text.len(),
    })
}

#[cfg(unix)]
fn restrict_to_owner(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).with_context(|| {
        format!(
            "failed to restrict permissions on backup {}",
            path.display()
        )
    })
}

/// Windows inherits the containing directory's ACL, which the operator chooses
/// when they pick an export directory; there is no mode to set here.
#[cfg(not(unix))]
fn restrict_to_owner(_path: &Path) -> Result<()> {
    Ok(())
}
