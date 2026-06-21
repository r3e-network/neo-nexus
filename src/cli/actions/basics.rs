use std::{env, fs, path::PathBuf, process, time::SystemTime};

use anyhow::{Context, Result};

use crate::core::workspace::Repository;

mod help;

pub(in crate::cli) use help::help_text;

pub(in crate::cli) fn version_text() -> String {
    format!("NeoNexus {}", env!("CARGO_PKG_VERSION"))
}

pub(in crate::cli) fn self_check_text() -> Result<String> {
    let check_dir = unique_self_check_dir();
    fs::create_dir_all(&check_dir).with_context(|| {
        format!(
            "failed to create self-check directory {}",
            check_dir.display()
        )
    })?;
    let db_path = check_dir.join("neonexus-self-check.db");
    let repository = Repository::open(db_path.clone()).with_context(|| {
        format!(
            "failed to open self-check SQLite workspace {}",
            db_path.display()
        )
    })?;
    let node_count = repository
        .list_nodes()
        .context("failed to query self-check workspace")?
        .len();
    drop(repository);
    fs::remove_dir_all(&check_dir).with_context(|| {
        format!(
            "failed to remove self-check directory {}",
            check_dir.display()
        )
    })?;

    Ok(format!(
        "{version}\ntarget: {os}/{arch}\nworkspace-db: ok ({node_count} nodes)\nnative-mode: eframe/egui\n",
        version = version_text(),
        os = env::consts::OS,
        arch = env::consts::ARCH,
    ))
}

fn unique_self_check_dir() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    env::temp_dir().join(format!("neonexus-self-check-{}-{timestamp}", process::id()))
}
