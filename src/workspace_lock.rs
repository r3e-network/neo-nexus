//! Cross-process ownership for the long-running web supervision engine.
//!
//! SQLite safely coordinates database writes, but it cannot prevent two web
//! servers on different ports from supervising the same node rows. The lock is
//! therefore held for the entire server lifetime. One-shot inspection commands
//! remain usable; only a second long-running supervisor is refused.

use std::{
    fs::{self, File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use fs2::FileExt;

const LOCK_FILE_NAME: &str = ".neonexus-supervisor.lock";

#[derive(Debug)]
pub(crate) struct WorkspaceSupervisorLock {
    file: File,
    path: PathBuf,
}

impl WorkspaceSupervisorLock {
    pub(crate) fn acquire(workspace: &Path) -> Result<Self> {
        fs::create_dir_all(workspace).with_context(|| {
            format!(
                "failed to create NeoNexus workspace {}",
                workspace.display()
            )
        })?;
        let path = workspace.join(LOCK_FILE_NAME);
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("failed to open supervisor lock {}", path.display()))?;
        file.try_lock_exclusive().with_context(|| {
            format!(
                "workspace {} already has an active NeoNexus supervision server",
                workspace.display()
            )
        })?;

        file.set_len(0)
            .with_context(|| format!("failed to reset supervisor lock {}", path.display()))?;
        file.seek(SeekFrom::Start(0))
            .with_context(|| format!("failed to seek supervisor lock {}", path.display()))?;
        writeln!(file, "pid={}", std::process::id())
            .with_context(|| format!("failed to write supervisor lock {}", path.display()))?;
        file.sync_data()
            .with_context(|| format!("failed to persist supervisor lock {}", path.display()))?;

        Ok(Self { file, path })
    }
}

impl Drop for WorkspaceSupervisorLock {
    fn drop(&mut self) {
        if let Err(error) = FileExt::unlock(&self.file) {
            eprintln!(
                "NeoNexus could not release supervisor lock {}: {error}",
                self.path.display()
            );
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/workspace_lock/tests.rs"]
mod tests;
