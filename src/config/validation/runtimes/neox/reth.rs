use crate::types::NodeConfig;

use super::super::super::{super::format::neox_reth_chain, model::ConfigValidationReport};

pub(super) fn check(
    node: &NodeConfig,
    report: &mut ConfigValidationReport,
    value: &toml::Value,
    text: &str,
) {
    match value.get("peers").and_then(toml::Value::as_table) {
        Some(_) => report.pass("Peering", "[peers] is present."),
        None => report.critical("Peering", "[peers] is missing from the neox-rs config."),
    }

    // Reth's config file has no key for any of these. If a future edit ever
    // adds one it is a mistake — the client would ignore it silently, and the
    // operator would believe the port in the file.
    for key in ["port", "rpc", "http", "chain", "datadir"] {
        if value.get(key).is_some() {
            report.critical(
                "Runtime flags",
                format!("`{key}` is not a Reth config key; it must be a launch flag."),
            );
            return;
        }
    }
    report.pass(
        "Runtime flags",
        "Chain, data directory and ports are left to the launch flags.",
    );

    match neox_reth_chain(node.network) {
        Some(chain) if text.contains(chain) => report.pass(
            "Chain",
            format!("The launch chain `{chain}` is recorded in the header."),
        ),
        Some(chain) => report.warning(
            "Chain",
            format!("The header does not name the `{chain}` chain spec."),
        ),
        None => report.warning(
            "Chain",
            "A private Neo X network needs its own chain spec passed to --chain.".to_string(),
        ),
    }
}
