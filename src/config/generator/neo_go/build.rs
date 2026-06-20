use anyhow::Result;

use crate::types::{NodeConfig, StorageEngine};

use super::{
    super::super::format::{
        effective_committee_public_keys, effective_network_magic, effective_seed_nodes,
        effective_validators_count, max_transactions_per_block, RuntimeConfigProfile,
    },
    model::{
        NeoGoApplicationConfiguration, NeoGoConfig, NeoGoDbConfiguration, NeoGoLevelDbOptions,
        NeoGoNodeConfiguration, NeoGoP2pConfiguration, NeoGoPprofConfiguration,
        NeoGoProtocolConfiguration, NeoGoRpcConfiguration,
    },
};

pub(super) fn neo_go_config(
    node: &NodeConfig,
    profile: Option<&RuntimeConfigProfile>,
) -> Result<NeoGoConfig> {
    if node.storage_engine != StorageEngine::LevelDb {
        anyhow::bail!("neo-go supports LevelDB storage in NeoNexus");
    }

    Ok(NeoGoConfig {
        protocol_configuration: protocol_configuration(node, profile),
        application_configuration: application_configuration(node),
    })
}

fn protocol_configuration(
    node: &NodeConfig,
    profile: Option<&RuntimeConfigProfile>,
) -> NeoGoProtocolConfiguration {
    NeoGoProtocolConfiguration {
        magic: effective_network_magic(node.network, profile),
        seed_list: effective_seed_nodes(node.network, profile),
        standby_committee: effective_committee_public_keys(profile),
        time_per_block: "15s".to_string(),
        max_transactions_per_block: max_transactions_per_block(node.network),
        validators_count: effective_validators_count(node.network, profile),
    }
}

fn application_configuration(node: &NodeConfig) -> NeoGoApplicationConfiguration {
    NeoGoApplicationConfiguration {
        db_configuration: db_configuration(node),
        p2p: p2p_configuration(node),
        rpc: rpc_configuration(node),
        pprof: pprof_configuration(node),
        node: node_configuration(),
    }
}

fn db_configuration(node: &NodeConfig) -> NeoGoDbConfiguration {
    NeoGoDbConfiguration {
        db_type: "leveldb".to_string(),
        leveldb_options: NeoGoLevelDbOptions {
            data_directory_path: format!("data/{}", node.network),
        },
    }
}

fn p2p_configuration(node: &NodeConfig) -> NeoGoP2pConfiguration {
    NeoGoP2pConfiguration {
        address: "0.0.0.0".to_string(),
        port: node.p2p_port,
        dial_timeout: "3s".to_string(),
        proto_tick_interval: "2s".to_string(),
        ping_interval: "30s".to_string(),
        ping_timeout: "90s".to_string(),
    }
}

fn rpc_configuration(node: &NodeConfig) -> NeoGoRpcConfiguration {
    NeoGoRpcConfiguration {
        enabled: true,
        enable_cors_workaround: false,
        max_gas_invoke: 20,
        session_enabled: true,
        session_expiration_time: 300_000,
        address: "127.0.0.1".to_string(),
        port: node.rpc_port,
    }
}

fn pprof_configuration(node: &NodeConfig) -> NeoGoPprofConfiguration {
    NeoGoPprofConfiguration {
        enabled: false,
        address: "127.0.0.1".to_string(),
        port: node.ws_port.unwrap_or(node.rpc_port.saturating_add(1)),
    }
}

fn node_configuration() -> NeoGoNodeConfiguration {
    NeoGoNodeConfiguration {
        relay: true,
        user_agent: "/NeoNexus:neo-go/".to_string(),
    }
}
