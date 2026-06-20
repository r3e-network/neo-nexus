use serde_json::Value;

use crate::types::NodeConfig;

use super::super::{
    super::format::{
        effective_committee_public_keys, effective_network_magic, effective_validators_count,
        neo_cli_storage_engine, RuntimeConfigProfile,
    },
    checks::*,
    model::ConfigValidationReport,
};

pub(in crate::config::validation) fn validate_neo_cli_config(
    node: &NodeConfig,
    text: &str,
    profile: Option<&RuntimeConfigProfile>,
    report: &mut ConfigValidationReport,
) {
    let value: Value = match serde_json::from_str(text) {
        Ok(value) => {
            report.pass("Parse", "JSON parsed successfully.");
            value
        }
        Err(error) => {
            report.critical("Parse", format!("JSON parse failed: {error}"));
            return;
        }
    };

    check_json_u32(
        report,
        &value,
        &["ProtocolConfiguration", "Network"],
        effective_network_magic(node.network, profile),
        "Network magic",
    );
    if profile.is_some() {
        check_json_u8(
            report,
            &value,
            &["ProtocolConfiguration", "ValidatorsCount"],
            effective_validators_count(node.network, profile),
            "Validators count",
        );
        check_json_array_len_at_least(
            report,
            &value,
            &["ProtocolConfiguration", "StandbyCommittee"],
            effective_committee_public_keys(profile).len(),
            "Standby committee",
        );
    }
    check_json_string(
        report,
        &value,
        &["ApplicationConfiguration", "Storage", "Engine"],
        neo_cli_storage_engine(node.storage_engine),
        "Storage engine",
    );
    check_json_u16(
        report,
        &value,
        &["ApplicationConfiguration", "P2P", "Port"],
        node.p2p_port,
        "P2P port",
    );
    check_json_u16(
        report,
        &value,
        &["ApplicationConfiguration", "RPC", "Port"],
        node.rpc_port,
        "RPC port",
    );
    check_json_bool(
        report,
        &value,
        &["ApplicationConfiguration", "UnlockWallet", "IsActive"],
        false,
        "Wallet unlock",
    );
    check_neo_cli_plugins(report, &value);
}
