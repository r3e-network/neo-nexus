mod contamination;
mod geth;
mod reth;

use crate::types::{NodeConfig, NodeType};

use super::super::{super::format::RuntimeConfigProfile, model::ConfigValidationReport};

pub(in crate::config::validation) fn validate_neox_config(
    node: &NodeConfig,
    text: &str,
    profile: Option<&RuntimeConfigProfile>,
    report: &mut ConfigValidationReport,
) {
    let value: toml::Value = match toml::from_str(text) {
        Ok(value) => {
            report.pass("Parse", "TOML parsed successfully.");
            value
        }
        Err(error) => {
            report.critical("Parse", format!("TOML parse failed: {error}"));
            return;
        }
    };

    contamination::check(report, &value);

    match node.node_type {
        NodeType::NeoXGeth => geth::check(node, profile, report, &value),
        NodeType::NeoXReth => reth::check(node, report, &value, text),
        other => report.critical("Client", format!("{other} is not a Neo X client.")),
    }
}
