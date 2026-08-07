//! The service sections a node's duty switches on.
//!
//! `Consensus`, `StateRoot` and `P2PNotary` are all `config.InternalService` in
//! neo-go — `Enabled` plus an `UnlockWallet`, nothing more. `Oracle` has its own
//! shape. Verified against `pkg/config/{internal_service,oracle_config,
//! notary_config,state_root,wallet_config}.go`.

use serde::Serialize;

use super::GoDuration;

/// The four service sections a node's duty can switch on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub(in crate::config::generator::neo_go) struct NeoGoServices {
    #[serde(rename = "Consensus", skip_serializing_if = "Option::is_none")]
    pub(in crate::config::generator::neo_go) consensus: Option<NeoGoInternalService>,
    #[serde(rename = "Oracle", skip_serializing_if = "Option::is_none")]
    pub(in crate::config::generator::neo_go) oracle: Option<NeoGoOracle>,
    #[serde(rename = "StateRoot", skip_serializing_if = "Option::is_none")]
    pub(in crate::config::generator::neo_go) state_root: Option<NeoGoInternalService>,
    #[serde(rename = "P2PNotary", skip_serializing_if = "Option::is_none")]
    pub(in crate::config::generator::neo_go) p2p_notary: Option<NeoGoInternalService>,
}

/// `Consensus`, `StateRoot` and `P2PNotary` are all `config.InternalService`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(in crate::config::generator::neo_go) struct NeoGoInternalService {
    #[serde(rename = "Enabled")]
    pub(in crate::config::generator::neo_go) enabled: bool,
    #[serde(rename = "UnlockWallet", skip_serializing_if = "Option::is_none")]
    pub(in crate::config::generator::neo_go) unlock_wallet: Option<NeoGoWallet>,
}

/// Stored in plaintext on disk, which is why a wallet is only ever written when
/// an operator has supplied one for this node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(in crate::config::generator::neo_go) struct NeoGoWallet {
    #[serde(rename = "Path")]
    pub(in crate::config::generator::neo_go) path: String,
    #[serde(rename = "Password")]
    pub(in crate::config::generator::neo_go) password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(in crate::config::generator::neo_go) struct NeoGoOracle {
    #[serde(rename = "Enabled")]
    pub(in crate::config::generator::neo_go) enabled: bool,
    #[serde(rename = "AllowPrivateHost")]
    pub(in crate::config::generator::neo_go) allow_private_host: bool,
    #[serde(rename = "AllowedContentTypes")]
    pub(in crate::config::generator::neo_go) allowed_content_types: Vec<String>,
    #[serde(rename = "Nodes")]
    pub(in crate::config::generator::neo_go) nodes: Vec<String>,
    #[serde(rename = "NeoFS")]
    pub(in crate::config::generator::neo_go) neofs: NeoGoOracleNeoFs,
    #[serde(rename = "MaxTaskTimeout")]
    pub(in crate::config::generator::neo_go) max_task_timeout: GoDuration,
    #[serde(rename = "RefreshInterval")]
    pub(in crate::config::generator::neo_go) refresh_interval: GoDuration,
    #[serde(rename = "MaxConcurrentRequests")]
    pub(in crate::config::generator::neo_go) max_concurrent_requests: u32,
    #[serde(rename = "RequestTimeout")]
    pub(in crate::config::generator::neo_go) request_timeout: GoDuration,
    #[serde(rename = "ResponseTimeout")]
    pub(in crate::config::generator::neo_go) response_timeout: GoDuration,
    #[serde(rename = "UnlockWallet", skip_serializing_if = "Option::is_none")]
    pub(in crate::config::generator::neo_go) unlock_wallet: Option<NeoGoWallet>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(in crate::config::generator::neo_go) struct NeoGoOracleNeoFs {
    #[serde(rename = "Nodes")]
    pub(in crate::config::generator::neo_go) nodes: Vec<String>,
    #[serde(rename = "Timeout")]
    pub(in crate::config::generator::neo_go) timeout: GoDuration,
}
