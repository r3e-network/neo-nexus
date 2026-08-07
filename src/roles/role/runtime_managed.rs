use crate::types::{NodeType, StorageEngine};

use super::{availability::role_availability, NodeRole, RolePlan};

/// A plan for a client that configures its services in one generated file
/// rather than through plugin assemblies.
///
/// neo-go and neo-rs used to share this note verbatim, which quietly implied
/// that anything true of one was true of the other. They are now given separate
/// notes, and an unavailable duty says so instead of producing an empty plan
/// that reads as success.
pub(super) fn runtime_managed_plan(
    node_type: NodeType,
    storage_engine: StorageEngine,
    role: NodeRole,
    runtime_note: &'static str,
) -> RolePlan {
    let availability = role_availability(node_type, role);
    let mut notes = vec![role.description()];
    match availability.reason() {
        Some(reason) => notes.push(reason),
        None => {
            notes.push(runtime_note);
            notes.push(service_note(role));
        }
    }

    RolePlan {
        role,
        node_type,
        storage_engine,
        plugin_changes: Vec::new(),
        notes,
    }
}

/// How the duty is expressed for a single-file client: which section of the
/// generated configuration carries it.
fn service_note(role: NodeRole) -> &'static str {
    match role {
        NodeRole::RpcApi | NodeRole::Observer => {
            "RPC posture is carried by the generated configuration, not by plugin state."
        }
        NodeRole::State => "State retention is a ledger setting in the generated configuration.",
        NodeRole::Indexer => {
            "Application log and token index queries are served by the built-in RPC server."
        }
        NodeRole::Consensus => {
            "Enabled through the Consensus service, which needs an unlocked wallet holding \
             the validator key."
        }
        NodeRole::Oracle => {
            "Enabled through the Oracle service, which needs an unlocked wallet and an \
             on-chain Oracle designation."
        }
        NodeRole::StateValidator => {
            "Enabled through the StateRoot service, which needs an unlocked wallet and an \
             on-chain StateValidator designation."
        }
        NodeRole::Notary => {
            "Enabled through the P2PNotary service, which needs P2PSigExtensions on the \
             network and an on-chain P2PNotary designation."
        }
    }
}
