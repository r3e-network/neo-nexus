use serde::{Deserialize, Serialize};

/// The identifier reported by the RPC server. Matching an identifier does not
/// prove a common genesis or protect against an RPC server lying about its chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RpcIdentityKind {
    N3NetworkMagic,
    EvmChainId,
}

impl RpcIdentityKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::N3NetworkMagic => "N3 network magic",
            Self::EvmChainId => "EVM chain ID",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RpcIdentityStatus {
    Unknown,
    Unverified,
    Matched,
    Mismatch,
}

/// Missing values remain unknown, including observations imported from an old
/// workspace. A private network has no inferred public identity or peer minimum.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RpcNetworkObservation {
    pub identity_kind: Option<RpcIdentityKind>,
    pub actual_identity: Option<u64>,
    pub expected_identity: Option<u64>,
    pub peer_count: Option<u64>,
    pub peers_expected: bool,
}

impl RpcNetworkObservation {
    pub fn identity_status(&self) -> RpcIdentityStatus {
        match (self.actual_identity, self.expected_identity) {
            (None, _) => RpcIdentityStatus::Unknown,
            (Some(_), None) => RpcIdentityStatus::Unverified,
            (Some(actual), Some(expected)) if actual == expected => RpcIdentityStatus::Matched,
            _ => RpcIdentityStatus::Mismatch,
        }
    }

    pub fn identity_summary(&self) -> String {
        let kind = self
            .identity_kind
            .map_or("network identity", RpcIdentityKind::label);
        match (self.actual_identity, self.expected_identity) {
            (None, Some(expected)) => format!("{kind} unavailable; expected {expected}"),
            (None, None) => format!("{kind} unavailable; not verified"),
            (Some(actual), None) => {
                format!("{kind} {actual}; observed, no expected identity configured")
            }
            (Some(actual), Some(expected)) if actual == expected => {
                format!("{kind} {actual}; matches expected")
            }
            (Some(actual), Some(expected)) => {
                format!("wrong network: {kind} {actual}; expected {expected}")
            }
        }
    }

    pub fn peer_summary(&self) -> String {
        match self.peer_count {
            None => "peer count unavailable".into(),
            Some(0) if self.peers_expected => "no connected peers on a public network".into(),
            Some(0) => "0 peers; an isolated private/development node may be intentional".into(),
            Some(count) => format!("{count} peers"),
        }
    }

    pub fn requires_attention(&self) -> bool {
        matches!(
            self.identity_status(),
            RpcIdentityStatus::Unknown | RpcIdentityStatus::Mismatch
        ) || self.peer_count.is_none()
            || self.peers_expected && self.peer_count == Some(0)
    }
}
