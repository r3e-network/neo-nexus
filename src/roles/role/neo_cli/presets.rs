use crate::{
    catalog::PluginId,
    roles::{NodeRole, RolePluginChange},
};

use super::changes::{disable, enable};

pub(super) fn role_plugin_changes(role: NodeRole) -> Vec<RolePluginChange> {
    match role {
        NodeRole::RpcApi => vec![
            enable(PluginId::RpcServer, "JSON-RPC is the primary API surface."),
            disable(
                PluginId::RestServer,
                "REST stays off unless explicitly needed.",
            ),
            disable(
                PluginId::DBFTPlugin,
                "API nodes should not perform consensus duties.",
            ),
        ],
        NodeRole::State => vec![
            enable(PluginId::RpcServer, "State workflows need RPC access."),
            enable(PluginId::StateService, "State proofs require StateService."),
            disable(
                PluginId::DBFTPlugin,
                "State service nodes do not validate by default.",
            ),
        ],
        NodeRole::Indexer => vec![
            enable(
                PluginId::RpcServer,
                "Indexers need RPC access for clients and probes.",
            ),
            enable(
                PluginId::ApplicationLogs,
                "Contract application logs are required for indexing.",
            ),
            enable(
                PluginId::TokensTracker,
                "NEP-11 and NEP-17 transfer tracking is indexer work.",
            ),
            enable(
                PluginId::StateService,
                "State roots improve index verification workflows.",
            ),
            disable(
                PluginId::DBFTPlugin,
                "Indexer nodes do not validate by default.",
            ),
        ],
        NodeRole::Consensus => vec![
            enable(PluginId::DBFTPlugin, "Consensus role requires dBFT duties."),
            disable(
                PluginId::RpcServer,
                "Consensus nodes keep public API off by default.",
            ),
            disable(
                PluginId::RestServer,
                "Consensus nodes keep REST API off by default.",
            ),
            disable(
                PluginId::ApplicationLogs,
                "Consensus role avoids indexer workload by default.",
            ),
            disable(
                PluginId::TokensTracker,
                "Consensus role avoids token index workload by default.",
            ),
        ],
        NodeRole::Observer => vec![
            enable(
                PluginId::RpcServer,
                "Observer nodes expose read-only RPC access.",
            ),
            disable(
                PluginId::RestServer,
                "REST stays off unless explicitly needed.",
            ),
            disable(
                PluginId::DBFTPlugin,
                "Observers must not participate in consensus.",
            ),
        ],
    }
}
