use crate::types::{NodeConfig, NodeType, StorageEngine};

use super::{neo_cli::neo_cli_plan, runtime_managed::runtime_managed_plan, NodeRole, RolePlan};

pub struct RolePlanner;

impl RolePlanner {
    pub fn plan(node: &NodeConfig, role: NodeRole) -> RolePlan {
        Self::plan_for(node.node_type, node.storage_engine, role)
    }

    pub fn plan_for(
        node_type: NodeType,
        storage_engine: StorageEngine,
        role: NodeRole,
    ) -> RolePlan {
        let mut plan = match node_type {
            NodeType::NeoCli => neo_cli_plan(storage_engine, role),
            NodeType::NeoGo => runtime_managed_plan(
                node_type,
                storage_engine,
                role,
                "neo-go configures every service in its generated YAML; there are no plugin assemblies to install.",
            ),
            NodeType::NeoRs => runtime_managed_plan(
                node_type,
                storage_engine,
                role,
                "neo-rs configures RPC, storage and consensus in its generated TOML.",
            ),
            NodeType::NeoXGeth => runtime_managed_plan(
                node_type,
                storage_engine,
                role,
                "Neo X Geth configures its RPC namespaces and peering in the generated TOML; consensus membership is decided on-chain.",
            ),
            NodeType::NeoXReth => runtime_managed_plan(
                node_type,
                storage_engine,
                role,
                "neox-rs takes its chain and ports as launch flags; validator mode also needs --validator.experimental on the public networks.",
            ),
        };
        if matches!(
            role,
            NodeRole::Consensus | NodeRole::Oracle | NodeRole::StateValidator | NodeRole::Notary
        ) {
            plan.notes.push("A wallet import validates metadata; a signer endpoint is a reference. Neither proves that a native signing key is attached to this node.");
            plan.notes.push(signer_note(node_type));
        }
        plan
    }
}

fn signer_note(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::NeoCli => "neo-cli requires its native wallet or compatible SignClient plugin configuration. NeoOS has an N3 SignClient gRPC bridge, but NeoNexus does not install or bind that plugin automatically.",
        NodeType::NeoGo => "neo-go signs through each service's UnlockWallet configuration. An encrypted wallet path alone leaves the service disabled; a NeoOS REST endpoint is not a supported UnlockWallet value.",
        NodeType::NeoRs => "neo-rs uses its native wallet/HSM signer interface. NeoNexus does not inject a NeoOS HTTP or gRPC consensus adapter; configure and verify the runtime signer separately.",
        NodeType::NeoXGeth => "geth-neox uses native wallet callbacks and the Clef external-signer protocol. A NeoOS REST transaction signer is not a Clef or NeoX dBFT signer; Anti-MEV/DKG keys require separate provisioning.",
        NodeType::NeoXReth => "neox-rs validator startup requires --validator.ecdsa-key and the matching Anti-MEV/DKG material. NeoNexus does not supply a remote validator adapter or export custody private keys into those files.",
    }
}
