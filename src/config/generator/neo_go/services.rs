//! The neo-go service sections a node's duty turns on.
//!
//! `Consensus`, `StateRoot` and `P2PNotary` are all `config.InternalService`:
//! `Enabled` plus an `UnlockWallet`. `Oracle` has its own richer shape. All
//! four are keys of `ApplicationConfiguration`, verified against
//! `pkg/config/*.go` on nspcc-dev/neo-go.
//!
//! Every one of them signs with a key, so none can be switched on without a
//! wallet: neo-go reads `UnlockWallet` as a value, not a pointer, and an
//! enabled service with an empty path fails at startup. A node with a duty but
//! no wallet therefore gets the section written **disabled**, which is honest —
//! the configuration records the intent, and validation says what is missing —
//! rather than a file that will not boot.

use crate::{
    config::format::{GenerationContext, ServiceWallet},
    roles::NodeRole,
};

use super::model::{
    GoDuration, NeoGoInternalService, NeoGoOracle, NeoGoOracleNeoFs, NeoGoServices, NeoGoWallet,
};

/// Builds the service sections for a duty. Sections a node does not perform are
/// omitted entirely rather than written disabled, so a config carries only what
/// is relevant to it.
pub(super) fn services_for(context: &GenerationContext) -> NeoGoServices {
    let wallet = context.wallet.as_ref();
    let mut services = NeoGoServices::default();
    let Some(role) = context.role else {
        return services;
    };
    match role {
        NodeRole::Consensus => services.consensus = Some(signing_service(wallet)),
        NodeRole::StateValidator => services.state_root = Some(signing_service(wallet)),
        NodeRole::Notary => services.p2p_notary = Some(signing_service(wallet)),
        NodeRole::Oracle => services.oracle = Some(oracle(wallet)),
        NodeRole::RpcApi | NodeRole::State | NodeRole::Indexer | NodeRole::Observer => {}
    }
    services
}

fn signing_service(wallet: Option<&ServiceWallet>) -> NeoGoInternalService {
    NeoGoInternalService {
        enabled: wallet.is_some(),
        unlock_wallet: wallet.map(unlock),
    }
}

fn unlock(wallet: &ServiceWallet) -> NeoGoWallet {
    NeoGoWallet {
        path: wallet.path.clone(),
        password: wallet.password.clone(),
    }
}

/// The oracle service. `Nodes` is the set of oracle peers responses are
/// broadcast to and is filled in by the operator; an empty list is valid and
/// simply means this node broadcasts to none yet.
fn oracle(wallet: Option<&ServiceWallet>) -> NeoGoOracle {
    NeoGoOracle {
        enabled: wallet.is_some(),
        allow_private_host: false,
        allowed_content_types: vec!["application/json".to_string()],
        nodes: Vec::new(),
        neofs: NeoGoOracleNeoFs {
            nodes: Vec::new(),
            timeout: GoDuration::seconds(15),
        },
        max_task_timeout: GoDuration::seconds(3_600),
        refresh_interval: GoDuration::seconds(180),
        max_concurrent_requests: 10,
        request_timeout: GoDuration::seconds(5),
        response_timeout: GoDuration::seconds(4),
        unlock_wallet: wallet.map(unlock),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/config/generator/neo_go/services/tests.rs"]
mod tests;
