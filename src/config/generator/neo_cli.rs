use anyhow::Result;
use serde_json::{json, Value};

use crate::{catalog::PluginState, types::NodeConfig};

use super::super::format::{
    effective_network_magic, max_transactions_per_block, neo_cli_storage_engine,
    RuntimeConfigProfile,
};
use super::ConfigGenerator;

impl ConfigGenerator {
    pub fn neo_cli(node: &NodeConfig, plugins: &[PluginState]) -> Result<Value> {
        Self::neo_cli_with_profile(node, plugins, None)
    }

    pub fn neo_cli_with_profile(
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<Value> {
        let enabled_plugins: Vec<Value> = plugins
            .iter()
            .filter(|plugin| plugin.enabled)
            .map(|plugin| json!({ "Name": plugin.plugin_id.to_string() }))
            .collect();
        let mut protocol = json!({
            "Network": effective_network_magic(node.network, profile)
        });
        if let Some(profile) = profile {
            protocol["SeedList"] = json!(profile.seed_nodes);
            protocol["ValidatorsCount"] = json!(profile.validators_count);
            protocol["StandbyCommittee"] = json!(profile.committee_public_keys);
            protocol["MillisecondsPerBlock"] = json!(15_000);
            protocol["MaxTransactionsPerBlock"] = json!(max_transactions_per_block(node.network));
        }

        Ok(json!({
            "ProtocolConfiguration": protocol,
            "ApplicationConfiguration": {
                "Storage": {
                    "Engine": neo_cli_storage_engine(node.storage_engine)
                },
                "P2P": {
                    "Port": node.p2p_port
                },
                "RPC": {
                    "Port": node.rpc_port
                },
                "UnlockWallet": {
                    "IsActive": false
                }
            },
            "Plugins": enabled_plugins
        }))
    }
}
