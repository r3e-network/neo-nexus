use crate::types::NodeConfig;

use super::super::{
    super::format::{
        effective_network_magic, effective_seed_nodes, effective_validators_count,
        RuntimeConfigProfile,
    },
    checks::*,
    model::ConfigValidationReport,
};

pub(in crate::config::validation) fn validate_neo_go_config(
    node: &NodeConfig,
    text: &str,
    profile: Option<&RuntimeConfigProfile>,
    report: &mut ConfigValidationReport,
) {
    let value: serde_yaml::Value = match serde_yaml::from_str(text) {
        Ok(value) => {
            report.pass("Parse", "YAML parsed successfully.");
            value
        }
        Err(error) => {
            report.critical("Parse", format!("YAML parse failed: {error}"));
            return;
        }
    };

    check_yaml_u32(
        report,
        &value,
        &["ProtocolConfiguration", "Magic"],
        effective_network_magic(node.network, profile),
        "Network magic",
    );
    check_yaml_u8(
        report,
        &value,
        &["ProtocolConfiguration", "ValidatorsCount"],
        effective_validators_count(node.network, profile),
        "Validators count",
    );
    check_yaml_array_len_at_least(
        report,
        &value,
        &["ProtocolConfiguration", "SeedList"],
        effective_seed_nodes(node.network, profile).len(),
        "Seed list matches",
    );
    // Every check above compares the file against what this workspace would
    // generate, which cannot catch a workspace that generates an unusable file:
    // `effective_seed_nodes(Private, None)` is empty and `len >= 0` passes.
    check_chain_identity(report, node, &chain_identity(&value));
    check_yaml_string(
        report,
        &value,
        &["ApplicationConfiguration", "DBConfiguration", "Type"],
        "leveldb",
        "Storage engine",
    );
    check_yaml_string(
        report,
        &value,
        &[
            "ApplicationConfiguration",
            "DBConfiguration",
            "LevelDBOptions",
            "DataDirectoryPath",
        ],
        &format!("data/{}", node.network),
        "Data directory",
    );
    // neo-go binds through `Addresses: ["host:port"]`. It has no Address/Port
    // pair, and `KnownFields(true)` makes emitting one a fatal startup error —
    // so the check has to look where the node actually reads.
    check_yaml_address_port(
        report,
        &value,
        &["ApplicationConfiguration", "P2P", "Addresses"],
        node.p2p_port,
        "P2P port",
    );
    check_yaml_address_port(
        report,
        &value,
        &["ApplicationConfiguration", "RPC", "Addresses"],
        node.rpc_port,
        "RPC port",
    );
    check_yaml_string(
        report,
        &value,
        &["ApplicationConfiguration", "LogEncoding"],
        "console",
        "Log encoding",
    );
}

/// Read the identity keys out of a neo-go config.
///
/// `None` means the key is absent, which is materially different from present
/// and empty: an absent key means the client uses whatever it was compiled
/// with, and for a node carrying a private magic those are the public values.
fn chain_identity(value: &serde_yaml::Value) -> ChainIdentity {
    let array_len = |path: &[&str]| {
        yaml_path(value, path).and_then(|found| match found {
            serde_yaml::Value::Sequence(items) => Some(items.len()),
            _ => None,
        })
    };
    ChainIdentity {
        seed_count: array_len(&["ProtocolConfiguration", "SeedList"]),
        committee_count: array_len(&["ProtocolConfiguration", "StandbyCommittee"]),
        validators_count: yaml_path(value, &["ProtocolConfiguration", "ValidatorsCount"])
            .and_then(serde_yaml::Value::as_u64),
        committee_is_expressible: true,
    }
}
