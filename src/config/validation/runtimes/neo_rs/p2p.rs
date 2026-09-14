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
        "Seed nodes match",
    );
    check_chain_identity(report, node, &chain_identity(value));
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

/// Read the identity keys out of a neo-rs config.
fn chain_identity(value: &toml::Value) -> ChainIdentity {
    let array_len =
        |path: &[&str]| toml_path(value, path).and_then(|found| found.as_array().map(Vec::len));
    ChainIdentity {
        seed_count: array_len(&["p2p", "seed_nodes"]),
        committee_count: array_len(&["consensus", "standby_committee"])
            .or_else(|| array_len(&["network", "standby_committee"])),
        validators_count: toml_path(value, &["consensus", "validators_count"])
            .or_else(|| toml_path(value, &["network", "validators_count"]))
            .and_then(toml::Value::as_integer)
            .map(|count| count.max(0) as u64),
        // neo-rs config has no committee key at all; the client carries its own.
        committee_is_expressible: false,
    }
}
