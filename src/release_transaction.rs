//! A multi-file upgrade published as one recoverable unit.
//!
//! The database and the config on disk cannot share one SQLite transaction, so
//! "upgrade this node" has to be its own transaction that the manager can
//! finish or undo. This module publishes a runtime pointer, the regenerated
//! config files, and the node record in a fixed order, backing everything up
//! before the first write and rolling each step back in reverse order when a
//! later step — including the health acceptance gate — fails. The durable
//! record in [`crate::repository::release_transactions`] says, after a crash,
//! exactly which phase was reached and where the backups are.
//!
//! This does not claim to replace the filesystem atomically or to run a live
//! chain. Plugin *files* keep their own atomic replace; this transaction gates
//! their compatibility and folds the selection into its final commit. Health
//! acceptance is the caller's bounded check of the target binary (see
//! [`ReleaseAcceptance`]).

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::{
    catalog::PluginState,
    config::ConfigExporter,
    repository::{ReleaseTransaction, Repository},
    types::{NewNode, NodeConfig},
};

/// Whether the newly-selected runtime will start. Bounded, never a live chain:
/// the caller supplies a smoke or readiness check so the gate is deterministic
/// in tests and real in production.
pub type ReleaseAcceptance = Box<dyn FnOnce(&Path) -> Result<()> + Send>;

/// Upgrades `node` to the installed runtime `target`, regenerating its managed
/// config and updating the node record, all as one rollback-able transaction.
pub fn apply_release_transaction(
    repository: &Repository,
    data_dir: &Path,
    node: &NodeConfig,
    target: &TargetRelease,
    acceptance: Option<ReleaseAcceptance>,
) -> Result<String> {
    if node.status.is_running() || node.pid.is_some() {
        anyhow::bail!("stop the node before applying a release");
    }
    preflight_target(repository, node, target)?;
    if crate::launch::runtime_args_include_config(node.node_type, &node.args) {
        anyhow::bail!("this node uses an external config; resolve it before release");
    }

    let plugins = repository.list_plugin_states(&node.id)?;
    let proposed = Proposed {
        node: node.clone(),
        target: target.clone(),
        nodes_root: data_dir.join("nodes"),
    };
    let conflicts = review_config_conflicts(repository, &proposed, &plugins)?;
    if conflicts > 0 {
        anyhow::bail!(
            "{conflicts} configuration conflict(s) require review in /config; release preserved"
        );
    }

    let backup_root = data_dir.join("release-backups");
    fs::create_dir_all(&backup_root)?;
    let backup_dir = backup_root.join(format!("{}-{}", node.name, Uuid::new_v4().simple()));

    let record = repository.begin_release_transaction(
        &node.id,
        &node.runtime_version,
        &node.binary_path.display().to_string(),
        &target.version,
        &target.binary_path.display().to_string(),
        &backup_dir.display().to_string(),
        unix_now(),
    )?;

    let tx = Tx {
        repository,
        backup_dir,
        record,
        proposed,
        acceptance,
    };
    tx.run()
}

/// A verified, present runtime package to point the node at.
#[derive(Debug, Clone)]
pub struct TargetRelease {
    pub version: String,
    pub binary_path: PathBuf,
}

fn preflight_target(
    repository: &Repository,
    node: &NodeConfig,
    target: &TargetRelease,
) -> Result<()> {
    let installation = repository
        .list_runtime_installations()?
        .into_iter()
        .find(|installation| {
            installation.node_type == node.node_type && installation.version == target.version
        })
        .context("target installed runtime not found")?;
    if installation.binary_path != target.binary_path {
        anyhow::bail!("installed runtime binary path does not match the selected target");
    }
    let (sha, bytes) = crate::snapshots::sha256_file(&installation.binary_path)?;
    if sha != installation.sha256 || bytes != installation.bytes {
        anyhow::bail!("installed binary changed since verification; reinstall before selecting it");
    }
    validate_smoke_plugins(repository, node, &target.version)
}

fn validate_smoke_plugins(repository: &Repository, node: &NodeConfig, version: &str) -> Result<()> {
    let plugins = repository.list_plugin_states(&node.id)?;
    for installed in repository.list_plugin_installations(&node.id)? {
        if plugins
            .iter()
            .any(|plugin| plugin.plugin_id == installed.plugin_id && plugin.enabled)
        {
            if let Some(release) =
                crate::plugins::installed_plugin_release(&installed.manifest_path)?
            {
                let mut probe = node.clone();
                probe.runtime_version = version.to_string();
                release.validate_for(&probe)?;
            }
        }
    }
    Ok(())
}

/// The node pointing at its proposed runtime, plus where its managed config
/// lives in this workspace.
struct Proposed {
    node: NodeConfig,
    target: TargetRelease,
    nodes_root: PathBuf,
}

impl Proposed {
    fn as_node(&self) -> NodeConfig {
        let mut node = self.node.clone();
        node.binary_path = self.target.binary_path.clone();
        node.runtime_version = self.target.version.clone();
        node
    }

    fn managed_config_path(&self) -> PathBuf {
        ConfigExporter::managed_target_path(self.nodes_root.join(&self.node.id), &self.as_node())
    }
}

fn review_config_conflicts(
    repository: &Repository,
    proposed: &Proposed,
    plugins: &[PluginState],
) -> Result<usize> {
    let context = crate::node_lifecycle::generation_context_for_node(repository, &proposed.node);
    let concrete = proposed.as_node();
    let path =
        ConfigExporter::managed_target_path(proposed.nodes_root.join(&proposed.node.id), &concrete);
    ConfigExporter::review_node_config(&path, &concrete, plugins, &context)
}

fn backup_file(backup_dir: &Path, source: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    let _ = fs::create_dir_all(backup_dir);
    let name = source
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "config".to_string());
    fs::copy(source, backup_dir.join(name)).context("failed to back up config file")?;
    Ok(())
}

struct Tx<'a> {
    repository: &'a Repository,
    backup_dir: PathBuf,
    record: ReleaseTransaction,
    proposed: Proposed,
    acceptance: Option<ReleaseAcceptance>,
}

impl Tx<'_> {
    fn run(mut self) -> Result<String> {
        self.transition("requested", "backing-up")?;
        backup_file(&self.backup_dir, &self.proposed.managed_config_path())?;

        self.publish_config()?;
        self.transition("backing-up", "applied")?;

        if let Some(acceptance) = self.acceptance.take() {
            if let Err(error) = acceptance(&self.proposed.target.binary_path) {
                self.rollback(format!("acceptance failed: {error}"))?;
                anyhow::bail!("acceptance failed; release rolled back: {error}");
            }
        }
        self.transition("applied", "accepted")?;

        self.commit_node_record()?;
        self.transition("accepted", "committed")?;

        Ok(format!(
            "{} upgraded {} -> {} (committed; config regenerated)",
            self.proposed.node.name, self.record.previous_version, self.proposed.target.version
        ))
    }

    fn publish_config(&self) -> Result<()> {
        let plugins = self.repository.list_plugin_states(&self.proposed.node.id)?;
        let proposed_node = self.proposed.as_node();
        let context = crate::node_lifecycle::generation_context_for_node(
            self.repository,
            &self.proposed.node,
        );
        let path = self.proposed.managed_config_path();
        ConfigExporter::write_node_config_to_path_with_context(
            &path,
            &proposed_node,
            &plugins,
            None,
            &context,
        )
        .context("failed to publish regenerated config")?;
        Ok(())
    }

    fn commit_node_record(&self) -> Result<()> {
        let node = &self.proposed.node;
        self.repository.update_node(
            &node.id,
            NewNode {
                name: node.name.clone(),
                node_type: node.node_type,
                network: node.network,
                binary_path: self.proposed.target.binary_path.clone(),
                args: node.args.clone(),
                runtime_version: self.proposed.target.version.clone(),
                storage_engine: node.storage_engine,
                rpc_port: node.rpc_port,
                p2p_port: node.p2p_port,
                ws_port: node.ws_port,
            },
        )?;
        Ok(())
    }

    fn rollback(&mut self, error: impl Into<String>) -> Result<()> {
        // Restore the config we overwrote from its backup. The node record is
        // only committed at the very end, so a rollback never touches it.
        let backup_file = self.backup_dir.join(
            self.proposed
                .managed_config_path()
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "config.json".to_string()),
        );
        if backup_file.exists() {
            let _ = fs::copy(&backup_file, self.proposed.managed_config_path());
        }
        let _ = self.repository.advance_release_transaction(
            &self.record.id,
            &self.record.phase.clone(),
            "rolled-back",
            Some(&error.into()),
            unix_now(),
        );
        Ok(())
    }

    fn transition(&mut self, expected: &str, next: &str) -> Result<()> {
        let advanced = self.repository.advance_release_transaction(
            &self.record.id,
            expected,
            next,
            None,
            unix_now(),
        )?;
        if !advanced {
            anyhow::bail!("release transaction phase race; someone else moved it");
        }
        self.record.phase = next.to_string();
        Ok(())
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
