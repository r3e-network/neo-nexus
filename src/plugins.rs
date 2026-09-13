mod archive;
mod fs_utils;
mod manager;
mod model;
mod validation;

pub use manager::PluginPackageManager;
pub use model::{PluginInstallation, PluginPackageManifest};
pub use validation::validate_plugin_package_manifest;

use crate::{
    catalog::{PluginCatalog, PluginId},
    types::{NodeConfig, NodeType, NodeTypeTraits},
};

/// Operator guidance for packages versus built-in runtime capabilities.
pub fn plugin_support_guidance(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::NeoCli => "Install a verified C# DLL ZIP package on a stopped neo-cli node using the Plugins page or PluginPackageManager. Save its launch configuration separately, then start the node. Managed catalog entries require restart; installation does not confirm runtime loading.",
        NodeType::NeoGo => "NeoGo has built-in services, including JSON-RPC, configured for the next launch. Use the catalog's RPC configuration control or the runtime's supported YAML settings. Adding Go modules requires source integration, rebuilding and restarting; uploading a C# DLL cannot add them.",
        NodeType::NeoRs => "NeoRs uses compiled-in functionality and Cargo features. Select features supported by your runtime version, rebuild that runtime in its source workspace, deploy the binary and restart. NeoNexus does not build features through the plugin installer.",
        NodeType::NeoXGeth => "NeoX-Geth uses built-in extensions. Configure options supported by your runtime version or use external RPC tooling; source-level changes require rebuilding and restarting. NeoNexus does not install C# DLL packages for this runtime.",
        NodeType::NeoXReth => "NeoX-Reth (neox-rs) uses Reth extensions integrated in the runtime build. Configure supported runtime options or build and deploy a compatible extension-enabled binary, then restart. NeoNexus does not dynamically install Reth extensions.",
    }
}

/// Reject unsupported package operations before inspecting or writing files.
pub fn ensure_plugin_package_support(node_type: NodeType) -> anyhow::Result<()> {
    if !node_type.supports_plugins() {
        anyhow::bail!(
            "Plugin packages are not supported for {node_type} nodes. Only neo-cli supports C# DLL plugin packages. {} See docs/PLUGIN_SUPPORT_MATRIX.md.",
            plugin_support_guidance(node_type)
        );
    }
    Ok(())
}

/// Shared precondition for the UI, upload handler and package installer.
pub fn ensure_plugin_installable(node: &NodeConfig) -> anyhow::Result<()> {
    ensure_plugin_package_support(node.node_type)?;
    if node.status.is_active() || node.pid.is_some() {
        anyhow::bail!(
            "stop and settle {} before installing a plugin; package changes apply on the next launch, not by hot reload",
            node.name
        );
    }
    Ok(())
}

/// Configuration controls include built-in services, not just DLL packages.
pub fn ensure_plugin_configuration_supported(
    node_type: NodeType,
    plugin_id: PluginId,
) -> anyhow::Result<()> {
    if !PluginCatalog
        .for_node_type(node_type)
        .iter()
        .any(|definition| definition.id == plugin_id)
    {
        anyhow::bail!(
            "{plugin_id} does not apply to a {node_type} node's managed launch configuration. {} See docs/PLUGIN_SUPPORT_MATRIX.md.",
            plugin_support_guidance(node_type)
        );
    }
    Ok(())
}

pub(crate) fn plugin_adapter_unavailable(node_type: Option<NodeType>) -> anyhow::Error {
    match node_type {
        Some(node_type) => anyhow::anyhow!(
            "No plugin adapter is registered for {node_type}; no plugin operation was performed. {} See docs/PLUGIN_SUPPORT_MATRIX.md.",
            plugin_support_guidance(node_type)
        ),
        None => anyhow::anyhow!(
            "No concrete plugin adapter or target node type is available; no plugin operation was performed. Only neo-cli supports C# DLL plugin packages through the verified package installer. NeoGo uses built-in services/Go modules, NeoRs uses Cargo features, NeoX-Geth uses built-in extensions and NeoX-Reth uses Reth extensions. Use the Plugins page for supported package or launch-configuration controls. See docs/PLUGIN_SUPPORT_MATRIX.md."
        ),
    }
}

pub(super) const PLUGIN_PACKAGE_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub(super) const PLUGIN_PACKAGE_MAX_EXPANDED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub(super) const PLUGIN_PACKAGE_MAX_FILES: usize = 20_000;
pub(super) const PLUGIN_CONTROL_DIR: &str = ".neonexus";
