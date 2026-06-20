mod blockchain;
mod consensus;
mod network;
mod p2p;
mod rpc;
mod storage;

use crate::types::NodeConfig;

use super::super::{super::format::RuntimeConfigProfile, model::ConfigValidationReport};

pub(in crate::config::validation) fn validate_neo_rs_config(
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

    network::check(node, profile, report, &value);
    storage::check(node, report, &value);
    p2p::check(node, profile, report, &value);
    rpc::check(node, report, &value);
    consensus::check(node, profile, report, &value);
    blockchain::check(node, report, &value);
}
