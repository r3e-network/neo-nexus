use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{redaction::redact_sensitive_text, types::NodeConfig};

use super::{model::unix_timestamp, ManagedProcessSpec};

pub fn log_path_for(log_dir: impl AsRef<Path>, node: &NodeConfig) -> PathBuf {
    log_dir
        .as_ref()
        .join(format!("{}-{}.log", node.node_type, node.id))
}

pub(super) fn open_launch_log(spec: &ManagedProcessSpec, log_path: &Path) -> Result<File> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log directory {}", parent.display()))?;
    }

    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("failed to open process log {}", log_path.display()))?;

    writeln!(log_file, "\n== NeoNexus launch {} ==", unix_timestamp())
        .with_context(|| format!("failed to write launch header to {}", log_path.display()))?;
    writeln!(log_file, "process-id: {}", spec.id)
        .with_context(|| format!("failed to write process id to {}", log_path.display()))?;
    writeln!(log_file, "process-kind: {}", spec.kind)
        .with_context(|| format!("failed to write process kind to {}", log_path.display()))?;
    writeln!(log_file, "{}: {}", spec.kind, spec.label)
        .with_context(|| format!("failed to write process label to {}", log_path.display()))?;
    writeln!(
        log_file,
        "command: {}",
        redact_sensitive_text(&spec.display_command)
    )
    .with_context(|| format!("failed to write launch command to {}", log_path.display()))?;
    fs::create_dir_all(&spec.working_dir).with_context(|| {
        format!(
            "failed to create working directory {}",
            spec.working_dir.display()
        )
    })?;
    writeln!(log_file, "working-dir: {}", spec.working_dir.display()).with_context(|| {
        format!(
            "failed to write working directory to {}",
            log_path.display()
        )
    })?;

    Ok(log_file)
}

pub(super) fn append_pid(log_file: &mut File, log_path: &Path, pid: u32) -> Result<()> {
    writeln!(log_file, "pid: {pid}")
        .with_context(|| format!("failed to write process id to {}", log_path.display()))
}
