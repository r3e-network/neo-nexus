use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::{NodeConfig, StorageEngine};

use super::super::format::{
    broadcast_history_limit, effective_committee_public_keys, effective_network_magic,
    effective_seed_nodes, max_transactions_per_block, RuntimeConfigProfile,
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
        if node.storage_engine != StorageEngine::RocksDb {
            anyhow::bail!("neo-rs requires RocksDB storage in NeoNexus");
        }

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
                enabled: profile.is_some_and(|profile| profile.consensus_enabled),
                auto_start: profile.is_some_and(|profile| profile.consensus_enabled),
                validators: effective_committee_public_keys(profile),
            },
            blockchain: NeoRsBlockchainConfig {
                block_time: 15_000,
                max_transactions_per_block: max_transactions_per_block(node.network),
            },
        };

        toml::to_string_pretty(&config).context("failed to render neo-rs TOML")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NeoRsConfig {
    network: NeoRsNetworkConfig,
    storage: NeoRsStorageConfig,
    p2p: NeoRsP2pConfig,
    rpc: NeoRsRpcConfig,
    consensus: NeoRsConsensusConfig,
    blockchain: NeoRsBlockchainConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NeoRsNetworkConfig {
    network_type: String,
    network_magic: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NeoRsStorageConfig {
    backend: String,
    data_dir: String,
    read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NeoRsP2pConfig {
    port: u16,
    bind_address: String,
    max_connections: i64,
    min_desired_connections: usize,
    max_connections_per_address: usize,
    max_known_hashes: usize,
    seed_nodes: Vec<String>,
    enable_compression: bool,
    broadcast_history_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NeoRsRpcConfig {
    enabled: bool,
    port: u16,
    bind_address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NeoRsConsensusConfig {
    enabled: bool,
    auto_start: bool,
    validators: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NeoRsBlockchainConfig {
    block_time: u32,
    max_transactions_per_block: u32,
}
