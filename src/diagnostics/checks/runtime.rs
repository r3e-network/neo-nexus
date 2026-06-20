use crate::{
    catalog::{PluginId, PluginState},
    diagnostics::{CheckSeverity, DiagnosticCheck},
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
    let mut checks = vec![DiagnosticCheck {
        severity: CheckSeverity::Pass,
        title: "RPC",
        detail: "neo-go exposes RPC through generated YAML configuration.".to_string(),
    }];

    if node.storage_engine == StorageEngine::LevelDb {
        checks.push(DiagnosticCheck {
            severity: CheckSeverity::Pass,
            title: "Storage",
            detail: "neo-go LevelDB backend matches the generated DBConfiguration.".to_string(),
        });
    } else {
        checks.push(DiagnosticCheck {
            severity: CheckSeverity::Warning,
            title: "Storage",
            detail: "neo-go support expects LevelDB in NeoNexus; switch storage before export."
                .to_string(),
        });
    }

    checks
}

fn neo_rs_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    let mut checks = vec![DiagnosticCheck {
        severity: CheckSeverity::Pass,
        title: "RPC",
        detail: "neo-rs exposes JSON-RPC through its TOML [rpc] section.".to_string(),
    }];

    if node.storage_engine == StorageEngine::RocksDb {
        checks.push(DiagnosticCheck {
            severity: CheckSeverity::Pass,
            title: "Storage",
            detail: "neo-rs RocksDB backend matches the selected storage.".to_string(),
        });
    } else {
        checks.push(DiagnosticCheck {
            severity: CheckSeverity::Warning,
            title: "Storage",
            detail: "neo-rs support expects RocksDB; switch storage before export.".to_string(),
        });
    }

    checks
}

fn neo_cli_plugin_checks(node: &NodeConfig, states: &[PluginState]) -> Vec<DiagnosticCheck> {
    let mut checks = Vec::new();

    if plugin_enabled(states, PluginId::RpcServer) {
        checks.push(DiagnosticCheck {
            severity: CheckSeverity::Pass,
            title: "RPC",
            detail: "RpcServer plugin is enabled.".to_string(),
        });
    } else {
        checks.push(DiagnosticCheck {
            severity: CheckSeverity::Warning,
            title: "RPC",
            detail: "RpcServer plugin is disabled, so API access may be unavailable.".to_string(),
        });
    }

    let expected_storage_plugin = match node.storage_engine {
        StorageEngine::LevelDb => PluginId::LevelDbStore,
        StorageEngine::RocksDb => PluginId::RocksDbStore,
    };

    if plugin_enabled(states, expected_storage_plugin) {
        checks.push(DiagnosticCheck {
            severity: CheckSeverity::Pass,
            title: "Storage",
            detail: format!("{expected_storage_plugin} plugin matches selected storage."),
        });
    } else {
        checks.push(DiagnosticCheck {
            severity: CheckSeverity::Warning,
            title: "Storage",
            detail: format!(
                "{expected_storage_plugin} plugin should be enabled for {}.",
                node.storage_engine
            ),
        });
    }

    checks
}

fn plugin_enabled(states: &[PluginState], plugin_id: PluginId) -> bool {
    states
        .iter()
        .any(|state| state.plugin_id == plugin_id && state.enabled)
}
