//! Protect operator edits across renders and runtime upgrades. Metadata stores
//! hashes only; the candidate and explicit replacement backups remain local.
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Default, Serialize, Deserialize)]
struct Baseline {
    generated: String,
    accepted: String,
    version: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigConflict {
    pub path: PathBuf,
    pub candidate_path: PathBuf,
    pub from_version: String,
    pub to_version: String,
    pub token: String,
    current: String,
    generated: String,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn control(path: &Path, name: &str) -> PathBuf {
    let file = path.file_name().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!(".{file}.neonexus-{name}"))
}

fn read(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn real_path(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => anyhow::bail!(
                "config path contains a symbolic link: {}",
                ancestor.display()
            ),
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    real_path(path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Set permissions when creating, before secret content reaches disk.
    let temporary = control(path, &format!("writing-{}", uuid::Uuid::new_v4()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(&temporary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    let result = (|| -> Result<()> {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

/// Stage a conflict without modifying the active file. Returns whether writing
/// the generated file is safe. All files are checked before the exporter writes.
pub(super) fn prepare(path: &Path, generated: &[u8], version: &str) -> Result<bool> {
    real_path(path)?;
    let Some(current) = read(path)? else {
        return Ok(true);
    };
    let generated_hash = hash(generated);
    let current_hash = hash(&current);
    let baseline: Option<Baseline> = read(&control(path, "baseline.json"))?
        .map(|bytes| serde_json::from_slice(&bytes))
        .transpose()?;
    let safe = current_hash == generated_hash
        || baseline.as_ref().is_some_and(|base| {
            current_hash == base.generated
                || (current_hash == base.accepted
                    && generated_hash == base.generated
                    && version == base.version)
        });
    if safe {
        clear(path);
        return Ok(true);
    }
    let candidate_path = control(path, "candidate");
    write(&candidate_path, generated)?;
    let conflict = ConfigConflict {
        path: path.to_path_buf(),
        candidate_path,
        from_version: baseline
            .map(|base| base.version)
            .unwrap_or_else(|| "untracked".into()),
        to_version: version.into(),
        token: uuid::Uuid::new_v4().to_string(),
        current: current_hash,
        generated: generated_hash,
    };
    write(
        &control(path, "conflict.json"),
        &serde_json::to_vec_pretty(&conflict)?,
    )?;
    Ok(false)
}

pub(super) fn publish(path: &Path, generated: &[u8], version: &str) -> Result<()> {
    if !prepare(path, generated, version)? {
        anyhow::bail!(
            "configuration changed before publication; review {} in /config",
            path.display()
        );
    }
    let baseline: Option<Baseline> = read(&control(path, "baseline.json"))?
        .map(|bytes| serde_json::from_slice(&bytes))
        .transpose()?;
    let generated_hash = hash(generated);
    let current = read(path)?;
    let keep = baseline.as_ref().is_some_and(|base| {
        base.generated == generated_hash
            && base.version == version
            && current.as_ref().is_some_and(|bytes| {
                hash(bytes) == base.accepted && base.accepted != base.generated
            })
    });
    if !keep {
        write(path, generated)?;
    }
    let accepted = if keep {
        hash(
            current
                .as_ref()
                .context("accepted configuration disappeared")?,
        )
    } else {
        generated_hash.clone()
    };
    write(
        &control(path, "baseline.json"),
        &serde_json::to_vec(&Baseline {
            generated: generated_hash,
            accepted,
            version: version.into(),
        })?,
    )?;
    clear(path);
    Ok(())
}

pub fn config_conflict(path: &Path) -> Result<Option<ConfigConflict>> {
    read(&control(path, "conflict.json"))?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(Into::into))
        .transpose()
}

/// Find staged reviews for both runtime files and package-specific plugin
/// configs. Paths come from directory entries, never from editable metadata.
pub fn list_config_conflicts(root: &Path) -> Result<Vec<ConfigConflict>> {
    let mut result = Vec::new();
    if !root.exists() {
        return Ok(result);
    }
    let mut pending = vec![root.to_path_buf()];
    let mut visited = 0usize;
    while let Some(directory) = pending.pop() {
        real_path(&directory)?;
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            visited += 1;
            if visited > 20_000 {
                anyhow::bail!("configuration inventory exceeds 20000 entries");
            }
            let metadata = entry.file_type()?;
            if metadata.is_symlink() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if metadata.is_dir() {
                // Runtime data and retained package backups are not active configs.
                if !matches!(name.as_str(), "data" | "logs" | ".neonexus") {
                    pending.push(entry.path());
                }
            } else if let Some(file) = name
                .strip_prefix('.')
                .and_then(|name| name.strip_suffix(".neonexus-conflict.json"))
            {
                let path = entry.path().with_file_name(file);
                if let Some(mut conflict) = config_conflict(&path)? {
                    conflict.path = path.clone();
                    conflict.candidate_path = control(&path, "candidate");
                    result.push(conflict);
                }
            }
        }
    }
    result.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(result)
}

pub(crate) fn prepare_plugin_config(path: &Path, generated: &[u8], version: &str) -> Result<bool> {
    prepare(path, generated, version)
}

pub(crate) fn stage_plugin_config(
    path: &Path,
    staged: &Path,
    generated: &[u8],
    version: &str,
) -> Result<()> {
    if let Some(current) = read(path)? {
        write(staged, &current)?;
    }
    if let Some(baseline) = read(&control(path, "baseline.json"))? {
        write(&control(staged, "baseline.json"), &baseline)?;
    }
    publish(staged, generated, version)
}

/// Resolve exactly the reviewed candidate. Both local and candidate content must
/// still match the review, so stale tabs cannot overwrite later operator edits.
pub fn resolve_config_conflict(
    path: &Path,
    token: &str,
    keep_local: bool,
) -> Result<Option<PathBuf>> {
    let conflict = config_conflict(path)?.context("no pending config conflict")?;
    if conflict.token != token {
        anyhow::bail!("configuration review expired; refresh and review again");
    }
    let current = read(path)?.context("local configuration is missing")?;
    let candidate =
        read(&control(path, "candidate"))?.context("configuration candidate is missing")?;
    if hash(&current) != conflict.current || hash(&candidate) != conflict.generated {
        anyhow::bail!("configuration changed after review; retry the operation and review again");
    }
    let backup = if keep_local {
        None
    } else {
        let backup = control(path, &format!("backup-{}", uuid::Uuid::new_v4()));
        write(&backup, &current)?;
        write(path, &candidate)?;
        Some(backup)
    };
    write(
        &control(path, "baseline.json"),
        &serde_json::to_vec(&Baseline {
            generated: conflict.generated.clone(),
            accepted: if keep_local {
                conflict.current
            } else {
                conflict.generated
            },
            version: conflict.to_version,
        })?,
    )?;
    clear(path);
    Ok(backup)
}

fn clear(path: &Path) {
    let _ = fs::remove_file(control(path, "conflict.json"));
    let _ = fs::remove_file(control(path, "candidate"));
}

#[cfg(test)]
#[path = "../../../tests/unit/config/managed.rs"]
mod tests;
