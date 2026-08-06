use anyhow::Result;

use crate::types::{NodeConfig, StorageEngine};

use super::{
    super::super::format::{
        effective_committee_public_keys, effective_network_magic, effective_seed_nodes,
        effective_validators_count, max_transactions_per_block, RuntimeConfigProfile,
    },
    model::{
        bind_address, GoDuration, NeoGoApplicationConfiguration, NeoGoBasicService, NeoGoConfig,
        NeoGoDbConfiguration, NeoGoLevelDbOptions, NeoGoP2pConfiguration, NeoGoPprofConfiguration,
        NeoGoProtocolConfiguration, NeoGoRpcConfiguration,
    },
};

/// Bind host for the services a node exposes. P2P must be reachable by peers;
/// the operator interfaces stay on loopback until an operator opts out.
const ANY_HOST: &str = "0.0.0.0";
const LOOPBACK_HOST: &str = "127.0.0.1";

/// Offsets from the RPC port for the diagnostic endpoints, so a fleet on one
/// host does not collide. Both are disabled by default; the addresses only
/// matter once an operator turns them on.
const PROMETHEUS_PORT_OFFSET: u16 = 2;
const PPROF_PORT_OFFSET: u16 = 3;

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
        standby_committee: effective_committee_public_keys(node.network, profile),
        time_per_block: GoDuration::seconds(15),
        max_transactions_per_block: max_transactions_per_block(node.network),
        validators_count: effective_validators_count(node.network, profile),
        verify_transactions: true,
    }
}

fn application_configuration(node: &NodeConfig) -> NeoGoApplicationConfiguration {
    NeoGoApplicationConfiguration {
        // neo-go spells the warning level `warn`; `warning` is rejected.
        log_level: "info".to_string(),
        log_encoding: "console".to_string(),
        // Left unset so the node logs to stdout, which is what the supervisor
        // captures. A path here would silently divert the log away from it.
        log_path: None,
        db_configuration: db_configuration(node),
        p2p: p2p_configuration(node),
        relay: true,
        rpc: rpc_configuration(node),
        prometheus: prometheus_configuration(node),
        pprof: pprof_configuration(node),
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
        addresses: vec![bind_address(ANY_HOST, node.p2p_port)],
        dial_timeout: GoDuration::seconds(3),
        proto_tick_interval: GoDuration::seconds(2),
        ping_interval: GoDuration::seconds(30),
        ping_timeout: GoDuration::seconds(90),
        min_peers: 5,
        max_peers: 100,
        attempt_conn_peers: 20,
    }
}

fn rpc_configuration(node: &NodeConfig) -> NeoGoRpcConfiguration {
    NeoGoRpcConfiguration {
        enabled: true,
        addresses: vec![bind_address(LOOPBACK_HOST, node.rpc_port)],
        enable_cors_workaround: false,
        max_gas_invoke: 20,
        session_enabled: true,
        session_lifetime: GoDuration::seconds(300),
    }
}

fn prometheus_configuration(node: &NodeConfig) -> NeoGoBasicService {
    NeoGoBasicService {
        enabled: false,
        addresses: vec![bind_address(
            LOOPBACK_HOST,
            node.rpc_port.saturating_add(PROMETHEUS_PORT_OFFSET),
        )],
    }
}

fn pprof_configuration(node: &NodeConfig) -> NeoGoPprofConfiguration {
    NeoGoPprofConfiguration {
        enabled: false,
        addresses: vec![bind_address(
            LOOPBACK_HOST,
            node.rpc_port.saturating_add(PPROF_PORT_OFFSET),
        )],
        enable_block: false,
        enable_mutex: false,
    }
}
