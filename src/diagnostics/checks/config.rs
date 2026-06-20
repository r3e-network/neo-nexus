use crate::{
    catalog::PluginState,
    config::{
        ConfigGenerator, ConfigValidationCheck, ConfigValidationSeverity, ConfigValidator,
        RenderedConfig,
    },
    diagnostics::{CheckSeverity, DiagnosticCheck},
    launch::{runtime_args_include_config, LaunchPlan},
    types::{NodeConfig, NodeType},
};

pub(in crate::diagnostics) fn managed_config_checks(
    node: &NodeConfig,
    launch_plan: Option<&LaunchPlan>,
) -> Vec<DiagnosticCheck> {
    let configured_external_config = runtime_args_include_config(node.node_type, &node.args);

    if configured_external_config {
        return vec![DiagnosticCheck {
            severity: CheckSeverity::Warning,
            title: "Managed config",
            detail: format!(
                "{} runtime args already include a config flag; NeoNexus will preserve it and will not inject the generated managed config.",
                node.node_type
            ),
        }];
    }

    let detail = match launch_plan.and_then(|plan| plan.managed_config_path.as_ref()) {
        Some(path) => format!(
            "NeoNexus will launch with generated managed config {}.",
            path.display()
        ),
        None => match node.node_type {
            NodeType::NeoCli => {
                "NeoNexus manages neo-cli config.json in the process working directory.".to_string()
            }
            NodeType::NeoGo => {
                "NeoNexus will inject generated YAML config with --config-file at launch."
                    .to_string()
            }
            NodeType::NeoRs => {
                "NeoNexus will inject generated TOML config with --config at launch.".to_string()
            }
        },
    };

    vec![DiagnosticCheck {
        severity: CheckSeverity::Pass,
        title: "Managed config",
        detail,
    }]
}

pub(in crate::diagnostics) fn config_checks(
    node: &NodeConfig,
    plugin_states: &[PluginState],
) -> Vec<DiagnosticCheck> {
    let rendered = match ConfigGenerator::render_for_node(node, plugin_states) {
        Ok(rendered) => rendered,
        Err(error) => {
            return vec![DiagnosticCheck {
                severity: CheckSeverity::Critical,
                title: "Config",
                detail: format!("Managed config rendering failed: {error}"),
            }];
        }
    };

    config_validation_checks(
        &rendered,
        ConfigValidator::validate_rendered(node, &rendered).checks,
    )
}

fn config_validation_checks(
    rendered: &RenderedConfig,
    checks: Vec<ConfigValidationCheck>,
) -> Vec<DiagnosticCheck> {
    std::iter::once(DiagnosticCheck {
        severity: CheckSeverity::Pass,
        title: "Config format",
        detail: format!("Managed config renders as {}.", rendered.format.label()),
    })
    .chain(checks.into_iter().map(|check| DiagnosticCheck {
        severity: severity_from_config_validation(check.severity),
        title: config_check_title(check.title),
        detail: check.detail,
    }))
    .collect()
}

fn severity_from_config_validation(severity: ConfigValidationSeverity) -> CheckSeverity {
    match severity {
        ConfigValidationSeverity::Pass => CheckSeverity::Pass,
        ConfigValidationSeverity::Warning => CheckSeverity::Warning,
        ConfigValidationSeverity::Critical => CheckSeverity::Critical,
    }
}

fn config_check_title(title: &'static str) -> &'static str {
    match title {
        "Parse" => "Config parse",
        "Format" => "Config format",
        "Network magic" => "Config network",
        "Network type" => "Config network",
        "Storage engine" | "Storage backend" => "Config storage",
        "Data directory" => "Config storage",
        "Read-only storage" => "Config storage",
        "P2P port" | "P2P bind" => "Config P2P",
        "Seed list" | "Seed nodes" => "Config seeds",
        "RPC port" | "RPC bind" | "RPC enabled" | "RPC plugin" => "Config RPC",
        "Consensus" | "Consensus auto start" | "Consensus validators" => "Config consensus",
        "Block time" | "Max transactions" => "Config chain",
        "Wallet unlock" => "Config wallet",
        "Plugin list" => "Config plugins",
        "Standby committee" | "Validators count" => "Config consensus",
        _ => "Config",
    }
}
