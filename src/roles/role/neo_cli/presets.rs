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
        NodeRole::Oracle => vec![
            enable(
                PluginId::OracleService,
                "The oracle duty is carried by OracleService.",
            ),
            enable(
                PluginId::RpcServer,
                "OracleService declares a hard dependency on RpcServer.",
            ),
            disable(
                PluginId::DBFTPlugin,
                "An oracle node does not produce blocks.",
            ),
        ],
        NodeRole::StateValidator => vec![
            enable(
                PluginId::StateService,
                "Signing state roots is StateService work.",
            ),
            enable(
                PluginId::RpcServer,
                "StateService declares a hard dependency on RpcServer.",
            ),
            disable(
                PluginId::DBFTPlugin,
                "A state validator does not produce blocks.",
            ),
        ],
        // Unreachable in practice: neo-cli has no notary service, so the
        // availability matrix rejects this pairing before a plan is built.
        NodeRole::Notary => Vec::new(),
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
