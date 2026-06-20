use crate::types::NodeConfig;

use super::super::super::{
    super::format::{effective_network_magic, RuntimeConfigProfile},
    checks::*,
    model::ConfigValidationReport,
};

pub(super) fn check(
    node: &NodeConfig,
    profile: Option<&RuntimeConfigProfile>,
    report: &mut ConfigValidationReport,
    value: &toml::Value,
) {
    check_toml_string(
        report,
        value,
        &["network", "network_type"],
        &node.network.to_string(),
        "Network type",
    );
    check_toml_u32(
        report,
        value,
        &["network", "network_magic"],
        effective_network_magic(node.network, profile),
        "Network magic",
    );
}
