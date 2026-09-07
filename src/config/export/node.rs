use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{bail, Context, Result};

use crate::{
    catalog::{PluginCatalog, PluginState},
    types::{NodeConfig, NodeType},
};

use super::{atomic::StagedWrite, model::ConfigExport};
use crate::config::{
    format::{config_filename, GenerationContext, RuntimeConfigProfile},
    generator::ConfigGenerator,
    validation::ConfigValidator,
};
use crate::private_network::magic_override::{ConsumedMagicOverride, MagicOverrideRequest};

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
        Self::write_node_config_to_path_with_profile_and_context(
            path,
            node,
            plugins,
            profile,
            None, // No magic override verification by default
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
        // Validated against the SAME context it was generated with. Checking a
        // duty-bearing config against a duty-free expectation rejects the
        // generator's own output and writes nothing.
        let validation = ConfigValidator::validate_text_with_context(
            node,
            rendered.format,
            &rendered.text,
            profile,
            context,
        );
        if !validation.is_success() {
            anyhow::bail!(
                "generated {} config failed validation: {}",
                node.node_type,
                validation.operator_summary()
            );
        }

        let path = path.as_ref().to_path_buf();
        let sidecars =
            Self::write_config_set(&path, rendered.text.as_bytes(), node, plugins, context)?;

        Ok(ConfigExport {
            bytes_written: rendered.text.len() + sidecars.bytes_written,
            sidecar_paths: sidecars.paths,
            path,
        })
    }

    /// Write config with optional magic override token verification
    ///
    /// If `override_token` is provided, this function will verify that:
    /// 1. The token was created specifically for this node ID
    /// 2. The token hasn't been consumed before (prevents replay)
    /// 3. The token hasn't expired
    ///
    /// This prevents cross-node magic number replay attacks where a profile
    /// intended for one node could be mistakenly applied to another node.
    pub fn write_node_config_to_path_with_profile_and_context(
        path: impl AsRef<Path>,
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
        override_token: Option<&MagicOverrideRequest>,
        context: &GenerationContext,
    ) -> Result<ConfigExport> {
        // Validate magic override if provided
        if let (Some(token), Some(profile)) = (override_token, profile) {
            // Verify this token is bound to this specific node
            let consumption = token.consume_for_node(&node.id)?;

            // Cross-check that consumption matches input profile
            if consumption.network_magic != profile.network_magic {
                anyhow::bail!(
                    "magic override token network_magic {} does not match requested profile {}",
                    consumption.network_magic,
                    profile.network_magic
                );
            }

            // Use the validated token data to ensure node binding
            return Self::write_node_config_with_validated_override(
                path,
                node,
                plugins,
                &consumption,
                context,
            );
        }

        // Profile without token: use standard flow (backward compatible)
        Self::write_node_config_to_path_with_context(path, node, plugins, profile, context)
    }

    /// Write configuration using a validated magic override
    fn write_node_config_with_validated_override(
        path: impl AsRef<Path>,
        node: &NodeConfig,
        plugins: &[PluginState],
        consumption: &ConsumedMagicOverride,
        context: &GenerationContext,
    ) -> Result<ConfigExport> {
        // Create a fresh profile from validated data
        let profile = consumption.into_runtime_config_profile();
        Self::write_node_config_to_path_with_context(path, node, plugins, Some(&profile), context)
    }

    /// Writes each enabled plugin's own configuration file beside the primary
    /// service and dBFT in `Plugins/<Name>/<Name>.json`, not in `config.json`,
    /// so an export that skips these configures none of them.
    fn write_config_set(
        primary: &Path,
        primary_text: &[u8],
        node: &NodeConfig,
        plugins: &[PluginState],
        context: &GenerationContext,
    ) -> Result<WrittenSidecars> {
        let Some(node_dir) = primary.parent() else {
            bail!(
                "config target {} has no parent directory",
                primary.display()
            );
        };

        let generated = ConfigGenerator::sidecars_for_node_with_context(node, plugins, context);
        let mut written = WrittenSidecars::default();
        let mut staged_sidecars = Vec::with_capacity(generated.len());
        for sidecar in &generated {
            let path = checked_sidecar_path(node_dir, &sidecar.relative_path)?;
            staged_sidecars.push(StagedWrite::new(&path, sidecar.text.as_bytes(), true)?);
            written.bytes_written += sidecar.text.len();
            written.paths.push(path);
        }

        // Stage every desired file before replacing any live config. A failure
        // while writing a temporary file therefore leaves the complete old set
        // intact rather than mixing a new primary with missing sidecars.
        let staged_primary = StagedWrite::new(primary, primary_text, true)?;
        remove_stale_sidecars(node_dir, node, &written.paths)?;
        for staged in staged_sidecars {
            staged.commit()?;
        }
        // The primary is the commit marker: a successful return always means
        // it and every desired sidecar were already replaced atomically.
        staged_primary.commit()?;
        Ok(written)
    }
}

#[derive(Default)]
struct WrittenSidecars {
    paths: Vec<PathBuf>,
    bytes_written: usize,
}

fn checked_sidecar_path(node_dir: &Path, relative: &str) -> Result<PathBuf> {
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("plugin config path {relative:?} must be a contained relative path");
    }
    Ok(node_dir.join(relative))
}

fn remove_stale_sidecars(node_dir: &Path, node: &NodeConfig, desired: &[PathBuf]) -> Result<()> {
    let every_plugin_enabled = PluginCatalog
        .all()
        .iter()
        .map(|plugin| PluginState {
            plugin_id: plugin.id,
            enabled: true,
        })
        .collect::<Vec<_>>();
    for sidecar in ConfigGenerator::sidecars_for_node(node, &every_plugin_enabled) {
        let path = checked_sidecar_path(node_dir, &sidecar.relative_path)?;
        if desired.contains(&path) {
            continue;
        }
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_file() || metadata.file_type().is_symlink() => {
                fs::remove_file(&path).with_context(|| {
                    format!("failed to remove stale plugin config {}", path.display())
                })?;
            }
            Ok(_) => bail!(
                "stale plugin config path {} is not a regular file",
                path.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to inspect stale plugin config {}", path.display())
                })
            }
        }
    }
    if node.node_type == NodeType::NeoCli {
        for relative in [
            "Plugins/SignClient/SignClient.json",
            "Plugins/NeoNexus.SignerBootstrap/SignerBootstrap.json",
        ] {
            let path = checked_sidecar_path(node_dir, relative)?;
            if !desired.contains(&path) {
                match fs::symlink_metadata(&path) {
                    Ok(metadata) if metadata.is_file() || metadata.file_type().is_symlink() => {
                        fs::remove_file(&path).with_context(|| {
                            format!("failed to remove stale plugin config {}", path.display())
                        })?;
                    }
                    Ok(_) => bail!(
                        "stale plugin config path {} is not a regular file",
                        path.display()
                    ),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(error).with_context(|| {
                            format!("failed to inspect stale plugin config {}", path.display())
                        })
                    }
                }
            }
        }
    }
    Ok(())
}
