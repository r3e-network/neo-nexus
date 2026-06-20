use crate::types::NodeType;

use super::{PluginCategory, PluginDefinition, PluginId};

const NEO_CLI_ONLY: &[NodeType] = &[NodeType::NeoCli];
const BUILT_IN_RPC: &[NodeType] = &[NodeType::NeoCli, NodeType::NeoGo];

pub(in crate::catalog) static PLUGIN_DEFINITIONS: [PluginDefinition; 8] = [
    PluginDefinition {
        id: PluginId::RpcServer,
        name: "JSON-RPC API",
        category: PluginCategory::Api,
        description: "Expose the standard Neo JSON-RPC interface for wallets, dApps, and tools.",
        node_types: BUILT_IN_RPC,
        requires_restart: true,
    },
    PluginDefinition {
        id: PluginId::RestServer,
        name: "REST API",
        category: PluginCategory::Api,
        description: "Expose a RESTful HTTP API for monitoring and service integrations.",
        node_types: NEO_CLI_ONLY,
        requires_restart: true,
    },
    PluginDefinition {
        id: PluginId::ApplicationLogs,
        name: "Application Logs",
        category: PluginCategory::Indexing,
        description: "Capture contract application logs for diagnostics and indexing.",
        node_types: NEO_CLI_ONLY,
        requires_restart: true,
    },
    PluginDefinition {
        id: PluginId::StateService,
        name: "State Service",
        category: PluginCategory::Core,
        description: "Track state roots and state proofs for verification workflows.",
        node_types: NEO_CLI_ONLY,
        requires_restart: true,
    },
    PluginDefinition {
        id: PluginId::DBFTPlugin,
        name: "dBFT Consensus",
        category: PluginCategory::Core,
        description: "Enable consensus node duties for private or committee deployments.",
        node_types: NEO_CLI_ONLY,
        requires_restart: true,
    },
    PluginDefinition {
        id: PluginId::TokensTracker,
        name: "Tokens Tracker",
        category: PluginCategory::Indexing,
        description: "Index NEP-11 and NEP-17 transfer state for wallet-facing APIs.",
        node_types: NEO_CLI_ONLY,
        requires_restart: true,
    },
    PluginDefinition {
        id: PluginId::LevelDbStore,
        name: "LevelDB Store",
        category: PluginCategory::Storage,
        description: "Use LevelDB for chain storage.",
        node_types: NEO_CLI_ONLY,
        requires_restart: true,
    },
    PluginDefinition {
        id: PluginId::RocksDbStore,
        name: "RocksDB Store",
        category: PluginCategory::Storage,
        description: "Use RocksDB for high-throughput chain storage.",
        node_types: NEO_CLI_ONLY,
        requires_restart: true,
    },
];
