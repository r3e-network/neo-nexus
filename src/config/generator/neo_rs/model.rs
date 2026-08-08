//! The neo-rs TOML config shape.
//!
//! Section and field names are taken from `NodeConfig` in
//! `neo-node/src/node/config.rs` on r3e-network/neo-rs, cross-checked against
//! the configs it ships in `config/`.
//!
//! Unlike neo-go, neo-rs does not reject unknown keys — serde ignores them — so
//! a wrong name here does not stop the node, it just silently does nothing.
//! That is the more dangerous failure of the two, because the config looks
//! applied and is not.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsConfig {
    pub(super) network: NeoRsNetworkConfig,
    pub(super) storage: NeoRsStorageConfig,
    pub(super) p2p: NeoRsP2pConfig,
    pub(super) rpc: NeoRsRpcConfig,
    pub(super) consensus: NeoRsConsensusConfig,
    pub(super) state_service: NeoRsStateServiceConfig,
    pub(super) indexer: NeoRsIndexerConfig,
    pub(super) application_logs: NeoRsApplicationLogsConfig,
    pub(super) tokens_tracker: NeoRsTokensTrackerConfig,
    pub(super) blockchain: NeoRsBlockchainConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsNetworkConfig {
    pub(super) network_type: String,
    pub(super) network_magic: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsStorageConfig {
    pub(super) backend: String,
    pub(super) data_dir: String,
    pub(super) read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsP2pConfig {
    pub(super) port: u16,
    pub(super) bind_address: String,
    pub(super) max_connections: i64,
    pub(super) min_desired_connections: usize,
    pub(super) max_connections_per_address: usize,
    pub(super) max_known_hashes: usize,
    pub(super) seed_nodes: Vec<String>,
    pub(super) enable_compression: bool,
    pub(super) broadcast_history_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsRpcConfig {
    pub(super) enabled: bool,
    pub(super) port: u16,
    pub(super) bind_address: String,
}

/// `[consensus]` (the daemon also accepts it as `[dbft]`).
///
/// There is deliberately no `validators` key: the section takes `enabled`,
/// `auto_start`, `private_key_hex` and an optional HSM block, and nothing else.
/// A committee list here was silently discarded by the node while leaking the
/// keys into a file that never read them.
///
/// `private_key_hex` is not emitted either. It is a raw secp256r1 private key
/// in plaintext, and NeoNexus does not hold or write private keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsConsensusConfig {
    pub(super) enabled: bool,
    pub(super) auto_start: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsBlockchainConfig {
    pub(super) block_time: u32,
    pub(super) max_transactions_per_block: u32,
}

/// `[state_service]`: state-root tracking and proof serving. neo-rs serves
/// state roots but has no key with which to sign them, which is why the
/// StateValidator duty is unavailable on this client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsStateServiceConfig {
    pub(super) enabled: bool,
    pub(super) full_state: bool,
    pub(super) track_during_catchup: bool,
    pub(super) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsIndexerConfig {
    pub(super) enabled: bool,
    pub(super) backfill_on_startup: bool,
    pub(super) store_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsApplicationLogsConfig {
    pub(super) enabled: bool,
    pub(super) path: String,
    pub(super) max_stack_size: u32,
    pub(super) debug: bool,
    pub(super) exception_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoRsTokensTrackerConfig {
    pub(super) enabled: bool,
    pub(super) db_path: String,
    pub(super) track_history: bool,
    pub(super) max_results: u32,
    pub(super) enabled_trackers: Vec<String>,
    pub(super) exception_policy: String,
}
