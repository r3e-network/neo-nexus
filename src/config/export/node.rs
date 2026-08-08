use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    catalog::PluginState,
    types::{NodeConfig, NodeType},
};

use super::model::ConfigExport;
use crate::config::{
    format::{config_filename, GenerationContext, RuntimeConfigProfile},
    generator::ConfigGenerator,
    validation::ConfigValidator,
};

pub struct ConfigExporter;

impl ConfigExporter {
    pub fn target_path(base_dir: impl AsRef<Path>, node: &NodeConfig) -> PathBuf {
        base_dir.as_ref().join(config_filename(node))
    }

    pub fn managed_target_path(node_work_dir: impl AsRef<Path>, node: &NodeConfig) -> PathBuf {
        let node_work_dir = node_work_dir.as_ref();
        match node.node_type {
            NodeType::NeoCli => node_work_dir.join("config.json"),
            NodeType::NeoGo | NodeType::NeoRs | NodeType::NeoXGeth | NodeType::NeoXReth => {
                node_work_dir.join("config").join(config_filename(node))
            }
        }
    }

    pub fn write_node_config(
        base_dir: impl AsRef<Path>,
        node: &NodeConfig,
        plugins: &[PluginState],
    ) -> Result<ConfigExport> {
        let base_dir = base_dir.as_ref();
        Self::write_node_config_to_path(Self::target_path(base_dir, node), node, plugins)
    }

    pub fn write_node_config_to_path(
        path: impl AsRef<Path>,
        node: &NodeConfig,
        plugins: &[PluginState],
    ) -> Result<ConfigExport> {
        Self::write_node_config_to_path_with_profile(path, node, plugins, None)
    }

    pub fn write_node_config_to_path_with_profile(
        path: impl AsRef<Path>,
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<ConfigExport> {
        Self::write_node_config_to_path_with_context(
            path,
            node,
            plugins,
            profile,
            &GenerationContext::default(),
        )
    }

    /// Writes a node's configuration for the duty it is being operated for.
    /// Without a context the render is a plain relaying node — which is what
    /// every export produced before duties reached the generator.
    pub fn write_node_config_to_path_with_context(
        path: impl AsRef<Path>,
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
        context: &GenerationContext,
    ) -> Result<ConfigExport> {
        let rendered =
            ConfigGenerator::render_for_node_with_context(node, plugins, profile, context)?;
        let validation = ConfigValidator::validate_rendered_with_profile(node, &rendered, profile);
        if !validation.is_success() {
            anyhow::bail!(
                "generated {} config failed validation: {}",
                node.node_type,
                validation.operator_summary()
            );
        }

        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory {}", parent.display())
            })?;
        }
        fs::write(&path, rendered.text.as_bytes())
            .with_context(|| format!("failed to write config {}", path.display()))?;

        restrict_permissions(&path);

        let sidecars = Self::write_plugin_sidecars(&path, node, plugins)?;

        Ok(ConfigExport {
            bytes_written: rendered.text.len() + sidecars.bytes_written,
            sidecar_paths: sidecars.paths,
            path,
        })
    }

    /// Writes each enabled plugin's own configuration file beside the primary
    /// one. neo-cli configures the RPC listener, the oracle service, the state
    /// service and dBFT in `Plugins/<Name>/<Name>.json`, not in `config.json`,
    /// so an export that skips these configures none of them.
    fn write_plugin_sidecars(
        primary: &Path,
        node: &NodeConfig,
        plugins: &[PluginState],
    ) -> Result<WrittenSidecars> {
        let Some(node_dir) = primary.parent() else {
            return Ok(WrittenSidecars::default());
        };
        let mut written = WrittenSidecars::default();
        for sidecar in ConfigGenerator::sidecars_for_node(node, plugins) {
            let path = node_dir.join(&sidecar.relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create plugin directory {}", parent.display())
                })?;
            }
            fs::write(&path, sidecar.text.as_bytes())
                .with_context(|| format!("failed to write plugin config {}", path.display()))?;
            restrict_permissions(&path);
            written.bytes_written += sidecar.text.len();
            written.paths.push(path);
        }
        Ok(written)
    }
}

#[derive(Default)]
struct WrittenSidecars {
    paths: Vec<PathBuf>,
    bytes_written: usize,
}

/// Config files carry network magic, seed addresses and validator keys, and a
/// plugin file may carry an RPC password, so they stay owner-only on Unix.
fn restrict_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}
