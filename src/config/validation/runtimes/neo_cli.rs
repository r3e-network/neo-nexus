use serde_json::Value;

use crate::types::NodeConfig;

use super::super::{
    super::format::{
        effective_committee_public_keys, effective_network_magic, effective_validators_count,
        neo_cli_storage_engine, GenerationContext, RuntimeConfigProfile,
    },
    checks::*,
    model::ConfigValidationReport,
};

pub(in crate::config::validation) fn validate_neo_cli_config(
    node: &NodeConfig,
    text: &str,
    profile: Option<&RuntimeConfigProfile>,
    context: &GenerationContext,
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
            effective_committee_public_keys(node.network, profile).len(),
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
    // No `ApplicationConfiguration.RPC` check: neo-cli has no such section. The
    // listener is configured in Plugins/RpcServer/RpcServer.json, which the
    // sidecar generator emits and its own tests cover.
    // `IsActive` is expected to match the wallet the config was generated with,
    // not hard-coded false. neo-cli opens one wallet for the whole node and its
    // signing plugins use it, so a duty-bearing export with a supplied password
    // is *meant* to set this true — asserting false rejected the generator's own
    // output and wrote no file for any neo-cli duty.
    check_json_bool(
        report,
        &value,
        &["ApplicationConfiguration", "UnlockWallet", "IsActive"],
        context
            .wallet
            .as_ref()
            .is_some_and(crate::config::ServiceWallet::can_unlock),
        "Wallet unlock",
    );
    // Without an active logger neo-cli installs no log sink at all and writes
    // nothing to file or console, leaving the Logs surface reading a file that
    // is never created.
    check_json_bool(
        report,
        &value,
        &["ApplicationConfiguration", "Logger", "Active"],
        true,
        "Logging",
    );
    check_neo_cli_plugin_source(report, &value);
}
