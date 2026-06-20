use crate::types::NodeConfig;

use super::super::super::{checks::*, model::ConfigValidationReport};

pub(super) fn check(node: &NodeConfig, report: &mut ConfigValidationReport, value: &toml::Value) {
    check_toml_bool(report, value, &["rpc", "enabled"], true, "RPC enabled");
    check_toml_u16(report, value, &["rpc", "port"], node.rpc_port, "RPC port");
    check_toml_string(
        report,
        value,
        &["rpc", "bind_address"],
        "127.0.0.1",
        "RPC bind",
    );
}
