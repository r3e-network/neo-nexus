use crate::types::NodeConfig;

use super::super::super::{
    super::format::{broadcast_history_limit, effective_seed_nodes, RuntimeConfigProfile},
    checks::*,
    model::ConfigValidationReport,
};

pub(super) fn check(
    node: &NodeConfig,
    profile: Option<&RuntimeConfigProfile>,
    report: &mut ConfigValidationReport,
    value: &toml::Value,
) {
    check_toml_u16(report, value, &["p2p", "port"], node.p2p_port, "P2P port");
    check_toml_string(
        report,
        value,
        &["p2p", "bind_address"],
        "0.0.0.0",
        "P2P bind",
    );
    check_toml_u32(
        report,
        value,
        &["p2p", "max_connections"],
        40,
        "P2P max connections",
    );
    check_toml_u32(
        report,
        value,
        &["p2p", "min_desired_connections"],
        10,
        "P2P desired connections",
    );
    check_toml_u32(
        report,
        value,
        &["p2p", "max_connections_per_address"],
        3,
        "P2P per-address limit",
    );
    check_toml_u32(
        report,
        value,
        &["p2p", "max_known_hashes"],
        1000,
        "P2P known hash cache",
    );
    check_toml_array_len_at_least(
        report,
        value,
        &["p2p", "seed_nodes"],
        effective_seed_nodes(node.network, profile).len(),
        "Seed nodes",
    );
    check_toml_bool(
        report,
        value,
        &["p2p", "enable_compression"],
        true,
        "P2P compression",
    );
    check_toml_u32(
        report,
        value,
        &["p2p", "broadcast_history_limit"],
        broadcast_history_limit(node.network) as u32,
        "P2P broadcast history",
    );
}
