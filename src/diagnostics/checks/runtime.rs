use crate::{
    catalog::{PluginId, PluginState},
    diagnostics::{CheckSeverity, DiagnosticCheck, DiagnosticResolution},
    types::{NodeConfig, NodeType, StorageEngine},
};

pub(in crate::diagnostics) fn plugin_checks(
    node: &NodeConfig,
    states: &[PluginState],
) -> Vec<DiagnosticCheck> {
    match node.node_type {
        NodeType::NeoGo => neo_go_checks(node),
        NodeType::NeoRs => neo_rs_checks(node),
        NodeType::NeoCli => neo_cli_plugin_checks(node, states),
    }
}

fn neo_go_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    let mut checks = vec![DiagnosticCheck::new(
        CheckSeverity::Pass,
        "RPC",
        "neo-go exposes RPC through generated YAML configuration.",
        DiagnosticResolution::ConfigWorkspace,
    )];

    if node.storage_engine == StorageEngine::LevelDb {
        checks.push(DiagnosticCheck::new(
            CheckSeverity::Pass,
            "Storage",
            "neo-go LevelDB backend matches the generated DBConfiguration.",
            DiagnosticResolution::NodeStudio,
        ));
    } else {
        checks.push(DiagnosticCheck::new(
            CheckSeverity::Warning,
            "Storage",
            "neo-go support expects LevelDB in NeoNexus; switch storage before export.",
            DiagnosticResolution::NodeStudio,
        ));
    }

    checks
}

fn neo_rs_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    let mut checks = vec![DiagnosticCheck::new(
        CheckSeverity::Pass,
        "RPC",
        "neo-rs exposes JSON-RPC through its TOML [rpc] section.",
        DiagnosticResolution::ConfigWorkspace,
    )];

    if node.storage_engine == StorageEngine::RocksDb {
        checks.push(DiagnosticCheck::new(
            CheckSeverity::Pass,
            "Storage",
            "neo-rs RocksDB backend matches the selected storage.",
            DiagnosticResolution::NodeStudio,
        ));
    } else {
        checks.push(DiagnosticCheck::new(
            CheckSeverity::Warning,
            "Storage",
            "neo-rs support expects RocksDB; switch storage before export.",
            DiagnosticResolution::NodeStudio,
        ));
    }

    checks
}

fn neo_cli_plugin_checks(node: &NodeConfig, states: &[PluginState]) -> Vec<DiagnosticCheck> {
    let mut checks = Vec::new();

    if plugin_enabled(states, PluginId::RpcServer) {
        checks.push(DiagnosticCheck::new(
            CheckSeverity::Pass,
            "RPC",
            "RpcServer plugin is enabled.",
            DiagnosticResolution::PluginManager,
        ));
    } else {
        checks.push(DiagnosticCheck::new(
            CheckSeverity::Warning,
            "RPC",
            "RpcServer plugin is disabled, so API access may be unavailable.",
            DiagnosticResolution::PluginManager,
        ));
    }

    let expected_storage_plugin = match node.storage_engine {
        StorageEngine::LevelDb => PluginId::LevelDbStore,
        StorageEngine::RocksDb => PluginId::RocksDbStore,
    };

    if plugin_enabled(states, expected_storage_plugin) {
        checks.push(DiagnosticCheck::new(
            CheckSeverity::Pass,
            "Storage",
            format!("{expected_storage_plugin} plugin matches selected storage."),
            DiagnosticResolution::PluginManager,
        ));
    } else {
        checks.push(DiagnosticCheck::new(
            CheckSeverity::Warning,
            "Storage",
            format!(
                "{expected_storage_plugin} plugin should be enabled for {}.",
                node.storage_engine
            ),
            DiagnosticResolution::PluginManager,
        ));
    }

    checks
}

fn plugin_enabled(states: &[PluginState], plugin_id: PluginId) -> bool {
    states
        .iter()
        .any(|state| state.plugin_id == plugin_id && state.enabled)
}
