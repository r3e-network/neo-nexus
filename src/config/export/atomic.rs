use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result};

static TEMPORARY_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A completely written file waiting to replace its destination.
///
/// The temporary file always lives beside the destination, so the final rename
/// cannot cross filesystems. Dropping an uncommitted stage removes it.
pub(super) struct StagedWrite {
    target: PathBuf,
    temporary: PathBuf,
    committed: bool,
}

impl StagedWrite {
    pub(super) fn new(target: impl AsRef<Path>, contents: &[u8], owner_only: bool) -> Result<Self> {
        let target = target.as_ref().to_path_buf();
        let parent = target
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;

        let (temporary, mut file) = create_temporary_file(parent, &target, owner_only)?;
        let staged = Self {
            target,
            temporary,
            committed: false,
        };
        file.write_all(contents)
            .with_context(|| format!("failed to stage config file {}", staged.target.display()))?;
        file.sync_all().with_context(|| {
            format!(
                "failed to persist staged config file {}",
                staged.target.display()
            )
        })?;
        if owner_only {
            restrict_permissions(&file, &staged.temporary)?;
        }
        drop(file);
        Ok(staged)
    }

    pub(super) fn commit(mut self) -> Result<()> {
        replace_file(&self.temporary, &self.target).with_context(|| {
            format!(
                "failed to atomically replace config file {}",
                self.target.display()
            )
        })?;
        self.committed = true;
        sync_parent(&self.target)?;
        Ok(())
    }
}

impl Drop for StagedWrite {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.temporary);
        }
    }
}

fn create_temporary_file(
    parent: &Path,
    target: &Path,
    owner_only: bool,
) -> Result<(PathBuf, File)> {
    let file_name = target
        .file_name()
        .context("config target must name a file")?
        .to_string_lossy();

    for _ in 0..64 {
        let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{file_name}.neonexus-tmp-{}-{sequence}",
            std::process::id()
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        configure_creation_permissions(&mut options, owner_only);
        match options.open(&temporary) {
            Ok(file) => return Ok((temporary, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to stage config file {}", target.display()))
            }
        }
    }

    anyhow::bail!(
        "failed to allocate a temporary file beside config {}",
        target.display()
    )
}

#[cfg(unix)]
fn configure_creation_permissions(options: &mut OpenOptions, owner_only: bool) {
    use std::os::unix::fs::OpenOptionsExt;

    if owner_only {
        options.mode(0o600);
    }
}

#[cfg(not(unix))]
fn configure_creation_permissions(_options: &mut OpenOptions, _owner_only: bool) {}

#[cfg(unix)]
fn restrict_permissions(file: &File, path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    file.set_permissions(fs::Permissions::from_mode(0o600))
        .with_context(|| {
            format!(
                "failed to restrict config permissions for {}",
                path.display()
            )
        })
}

#[cfg(not(unix))]
fn restrict_permissions(_file: &File, _path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    use std::{iter, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source: Vec<u16> = source
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect();
    // SAFETY: both buffers are NUL-terminated and remain alive for the call.
    let moved = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(unix)]
fn sync_parent(target: &Path) -> Result<()> {
    let parent = target
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .with_context(|| {
            format!(
                "failed to persist config directory metadata {}",
                parent.display()
            )
        })
}

#[cfg(not(unix))]
fn sync_parent(_target: &Path) -> Result<()> {
    Ok(())
}
