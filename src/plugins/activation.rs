use super::{fs_utils::ensure_real_directory_exists, PluginPackageManager};
use crate::{
    catalog::PluginId,
    repository::Repository,
    types::{NodeConfig, NodeType},
};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn disabled_path(work: &Path, plugin: PluginId) -> PathBuf {
    work.join(".neonexus-disabled-plugins")
        .join(plugin.to_string())
}

impl PluginPackageManager {
    pub fn validate_enabled_layout(
        repository: &Repository,
        work: &Path,
        node: &NodeConfig,
    ) -> Result<()> {
        if node.node_type != NodeType::NeoCli {
            return Ok(());
        }
        for state in repository.list_plugin_states(&node.id)? {
            let unexpected = if state.enabled {
                disabled_path(work, state.plugin_id)
            } else {
                work.join("Plugins").join(state.plugin_id.to_string())
            };
            if unexpected.exists() {
                anyhow::bail!(
                    "plugin {} activation changed; stop the node before applying plugin state",
                    state.plugin_id
                );
            }
        }
        Ok(())
    }

    /// neo-cli loads assemblies by filesystem presence. Disabled packages live
    /// outside Plugins, preserving binaries, configuration and version metadata.
    pub fn set_enabled(work: &Path, plugin: PluginId, enabled: bool) -> Result<()> {
        let active = work.join("Plugins").join(plugin.to_string());
        let disabled = disabled_path(work, plugin);
        let (source, target) = if enabled {
            (&disabled, &active)
        } else {
            (&active, &disabled)
        };
        if !source.exists() {
            return Ok(());
        }
        for path in [source, target] {
            for ancestor in path.ancestors() {
                if fs::symlink_metadata(ancestor)
                    .is_ok_and(|metadata| metadata.file_type().is_symlink())
                {
                    anyhow::bail!("plugin activation path contains a symbolic link");
                }
            }
        }
        ensure_real_directory_exists(source, "plugin package")?;
        if target.exists() {
            anyhow::bail!("both active and disabled copies of {plugin} exist; reconcile the package directories before toggling");
        }
        ensure_real_directory_exists(
            target
                .parent()
                .context("plugin activation directory has no parent")?,
            "plugin activation directory",
        )?;
        fs::rename(source, target)?;
        Ok(())
    }

    /// Reconcile desired state before a managed launch, including state changes
    /// made by role planning or headless repository clients.
    pub fn synchronize_enabled(
        repository: &Repository,
        work: &Path,
        node: &NodeConfig,
    ) -> Result<()> {
        if node.node_type != NodeType::NeoCli {
            return Ok(());
        }
        for state in repository.list_plugin_states(&node.id)? {
            Self::set_enabled(work, state.plugin_id, state.enabled)?;
        }
        Self::refresh_installation_paths(repository, work, &node.id)
    }

    pub fn refresh_installation_paths(
        repository: &Repository,
        work: &Path,
        node_id: &str,
    ) -> Result<()> {
        for mut installation in repository.list_plugin_installations(node_id)? {
            let disabled = disabled_path(work, installation.plugin_id);
            let target = if disabled.is_dir() {
                disabled
            } else {
                work.join("Plugins")
                    .join(installation.plugin_id.to_string())
            };
            if installation.installed_path != target {
                installation.installed_path = target.clone();
                installation.manifest_path = target.join(".neonexus/manifest.json");
                repository.upsert_plugin_installation(&installation)?;
            }
        }
        Ok(())
    }
}
