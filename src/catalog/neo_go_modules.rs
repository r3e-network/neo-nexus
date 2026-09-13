//! NeoGo module definitions
//!
//! NeoGo uses Go modules instead of traditional plugins.
//! These are compiled extensions that modify node behavior.

use serde::{Deserialize, Serialize};

/// NeoGo module definition (similar to plugin but Go-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoGoModuleDefinition {
    /// Unique module identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Version compatibility
    pub version: String,

    /// Module description
    pub description: String,

    /// Required NeoGo version constraint
    pub required_version: Option<String>,

    /// Whether this module modifies consensus rules
    pub consensus_critical: bool,

    /// Go package path for installation
    pub go_package: String,

    /// Enable/disable status
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl NeoGoModuleDefinition {
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        go_package: &str,
        consensus_critical: bool,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: description.to_string(),
            required_version: None,
            consensus_critical,
            go_package: go_package.to_string(),
            enabled: !consensus_critical,
        }
    }
}

/// Built-in NeoGo modules available for installation
pub fn get_available_modules() -> Vec<NeoGoModuleDefinition> {
    vec![
        // Consensus-related modules
        NeoGoModuleDefinition::new(
            "echo",
            "Echo Module",
            "Sends echo responses during consensus for testing",
            "github.com/n3ll/echo-module",
            false,
        ),
        NeoGoModuleDefinition::new(
            "stateroot",
            "State Root Module",
            "Maintains merkle roots of contract states",
            "github.com/nspcc-dev/neogo/pkg/service/stateroot",
            false,
        ),
        NeoGoModuleDefinition::new(
            "txindex",
            "Transaction Index Module",
            "Maintains transaction history index for RPC queries",
            "github.com/nspcc-dev/neogo/pkg/service/txindex",
            false,
        ),
        // Performance optimization modules
        NeoGoModuleDefinition::new(
            "pprof",
            "Performance Profiling Module",
            "Enables pprof profiling endpoint for performance analysis",
            "net/http/pprof",
            false,
        ),
        // Security modules
        NeoGoModuleDefinition::new(
            "metrics",
            "Prometheus Metrics Module",
            "Exports metrics in Prometheus format",
            "github.com/prometheus/client_golang",
            false,
        ),
    ]
}

/// Module installation needs a source build; this catalog is not an installer.
pub fn install_module(module_id: &str, _working_dir: &std::path::Path) -> anyhow::Result<()> {
    anyhow::bail!(
        "NeoGo module '{module_id}' was not installed: automated source integration and rebuilding are not implemented. No files were changed. Verify the module against your NeoGo version, integrate it in the runtime's source workspace, rebuild, deploy and restart. Built-in services can instead use supported YAML settings or the Plugins page's RPC configuration control."
    )
}

/// Generic module ids have no verified mapping to NeoGo service configuration.
pub fn toggle_module(
    module_id: &str,
    _enabled: bool,
    _working_dir: &std::path::Path,
) -> anyhow::Result<()> {
    anyhow::bail!(
        "NeoGo module '{module_id}' was not changed: generic module configuration is not implemented. No files were changed. Configure the built-in service using settings supported by your NeoGo version; use the Plugins page's RPC control for managed JSON-RPC configuration on a stopped node, then launch it again. Missing configuration must be generated before editing it."
    )
}

/// Check if NeoGo supports modules
pub fn supports_modules(node_type: crate::types::NodeType) -> bool {
    matches!(node_type, crate::types::NodeType::NeoGo)
}
