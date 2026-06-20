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
        "Seed list",
    );
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
    check_yaml_u16(
        report,
        &value,
        &["ApplicationConfiguration", "P2P", "Port"],
        node.p2p_port,
        "P2P port",
    );
    check_yaml_u16(
        report,
        &value,
        &["ApplicationConfiguration", "RPC", "Port"],
        node.rpc_port,
        "RPC port",
    );
}
