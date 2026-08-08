use anyhow::{Context, Result};

use crate::types::{NodeConfig, StorageEngine};

use crate::roles::NodeRole;

use super::super::format::{
    broadcast_history_limit, effective_network_magic, effective_seed_nodes,
    max_transactions_per_block, GenerationContext, RuntimeConfigProfile,
};
use super::ConfigGenerator;

impl ConfigGenerator {
    pub fn neo_rs_toml(node: &NodeConfig) -> Result<String> {
        Self::neo_rs_toml_with_profile(node, None)
    }

    pub fn neo_rs_toml_with_profile(
        node: &NodeConfig,
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<String> {
        Self::neo_rs_toml_with_context(node, profile, &GenerationContext::default())
    }

    pub fn neo_rs_toml_with_context(
        node: &NodeConfig,
        profile: Option<&RuntimeConfigProfile>,
        context: &GenerationContext,
    ) -> Result<String> {
        if node.storage_engine != StorageEngine::RocksDb {
            anyhow::bail!("neo-rs requires RocksDB storage in NeoNexus");
        }

        // A duty decides which service sections the node runs. Consensus can
        // also come from a private-network profile, which predates duties.
        let consensus = matches!(context.role, Some(NodeRole::Consensus))
            || profile.is_some_and(|profile| profile.consensus_enabled);
        let indexing = matches!(context.role, Some(NodeRole::Indexer));

        let config = NeoRsConfig {
            network: NeoRsNetworkConfig {
                network_type: node.network.to_string(),
                network_magic: effective_network_magic(node.network, profile),
            },
            storage: NeoRsStorageConfig {
                backend: "rocksdb".to_string(),
                data_dir: format!("./data/{}", node.network),
                read_only: false,
            },
            p2p: NeoRsP2pConfig {
                port: node.p2p_port,
                bind_address: "0.0.0.0".to_string(),
                max_connections: 40,
                min_desired_connections: 10,
                max_connections_per_address: 3,
                max_known_hashes: 1000,
                seed_nodes: effective_seed_nodes(node.network, profile),
                enable_compression: true,
                broadcast_history_limit: broadcast_history_limit(node.network),
            },
            rpc: NeoRsRpcConfig {
                enabled: true,
                port: node.rpc_port,
                bind_address: "127.0.0.1".to_string(),
            },
            consensus: NeoRsConsensusConfig {
                enabled: consensus,
                auto_start: consensus,
            },
            state_service: NeoRsStateServiceConfig {
                enabled: matches!(context.role, Some(NodeRole::State)),
                full_state: matches!(context.role, Some(NodeRole::State)),
                track_during_catchup: false,
                path: format!("./data/{}/state-root", node.network),
            },
            indexer: NeoRsIndexerConfig {
                enabled: indexing,
                backfill_on_startup: indexing,
                store_path: format!("./data/{}/indexer", node.network),
            },
            application_logs: NeoRsApplicationLogsConfig {
                enabled: indexing,
                path: format!("./data/{}/application-logs", node.network),
                max_stack_size: 65_535,
                debug: false,
                exception_policy: "Ignore".to_string(),
            },
            tokens_tracker: NeoRsTokensTrackerConfig {
                enabled: indexing,
                db_path: format!("./data/{}/tokens", node.network),
                track_history: true,
                max_results: 1000,
                enabled_trackers: vec!["NEP-17".to_string(), "NEP-11".to_string()],
                exception_policy: "StopPlugin".to_string(),
            },
            blockchain: NeoRsBlockchainConfig {
                block_time: 15_000,
                max_transactions_per_block: max_transactions_per_block(node.network),
            },
        };

        toml::to_string_pretty(&config).context("failed to render neo-rs TOML")
    }
}

mod model;

use model::*;
