use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{
    catalog::PluginState,
    types::{NodeConfig, NodeType},
};

use super::{managed, model::ConfigExport};
use crate::config::{
    format::{config_filename, GenerationContext, RuntimeConfigProfile},
    generator::ConfigGenerator,
    validation::ConfigValidator,
};

pub struct ConfigExporter;

impl ConfigExporter {
    /// Stage candidates for operator review without changing any active file.
    pub fn review_node_config(
        path: &Path,
        node: &NodeConfig,
        plugins: &[PluginState],
        context: &GenerationContext,
    ) -> Result<usize> {
        let rendered = ConfigGenerator::render_for_node_with_context(node, plugins, None, context)?;
        let mut count = usize::from(!managed::prepare(
            path,
            rendered.text.as_bytes(),
            &node.runtime_version,
        )?);
        if let Some(directory) = path.parent() {
            for sidecar in ConfigGenerator::sidecars_for_node(node, plugins) {
                count += usize::from(!managed::prepare(
                    &directory.join(sidecar.relative_path),
                    sidecar.text.as_bytes(),
                    &node.runtime_version,
                )?);
            }
        }
        Ok(count)
    }

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
        if node.node_type == NodeType::NeoCli {
            if let Some(directory) = path.parent() {
                for plugin in plugins.iter().filter(|plugin| plugin.enabled) {
                    let manifest = directory
                        .join("Plugins")
                        .join(plugin.plugin_id.to_string())
                        .join(".neonexus/manifest.json");
                    if manifest.is_file() {
                        if let Some(release) = crate::plugins::installed_plugin_release(&manifest)?
                        {
                            release.validate_for(node)?;
                        }
                    }
                }
            }
        }
        let mut files = vec![(path.clone(), rendered.text)];
        if let Some(node_dir) = path.parent() {
            files.extend(
                ConfigGenerator::sidecars_for_node(node, plugins)
                    .into_iter()
                    .map(|sidecar| (node_dir.join(sidecar.relative_path), sidecar.text)),
            );
        }
        let mut conflicts = Vec::new();
        for (file, text) in &files {
            if !managed::prepare(file, text.as_bytes(), &node.runtime_version)? {
                conflicts.push(file.display().to_string());
            }
        }
        if !conflicts.is_empty() {
            anyhow::bail!("configuration conflicts: {}. Active files were preserved. Review local and candidate files in /config before retrying", conflicts.join(", "));
        }
        for (file, text) in &files {
            managed::publish(file, text.as_bytes(), &node.runtime_version)?;
        }

        Ok(ConfigExport {
            bytes_written: files.iter().map(|(_, text)| text.len()).sum(),
            sidecar_paths: files.into_iter().skip(1).map(|(path, _)| path).collect(),
            path,
        })
    }
}
