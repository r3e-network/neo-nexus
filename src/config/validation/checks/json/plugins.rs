use serde_json::Value;

use super::super::{super::model::ConfigValidationReport, paths::json_path};

pub(in crate::config::validation) fn check_neo_cli_plugins(
    report: &mut ConfigValidationReport,
    value: &Value,
) {
    let Some(plugins) = json_path(value, &["Plugins"]).and_then(Value::as_array) else {
        report.critical("Plugin list", "Plugins must be an array.");
        return;
    };

    let has_rpc = plugins.iter().any(|plugin| {
        plugin
            .get("Name")
            .and_then(Value::as_str)
            .is_some_and(|name| name == "RpcServer")
    });
    if has_rpc {
        report.pass("RPC plugin", "RpcServer plugin is enabled for neo-cli RPC.");
    } else {
        report.warning(
            "RPC plugin",
            "RpcServer plugin is not enabled; JSON-RPC will not be available.",
        );
    }
}
