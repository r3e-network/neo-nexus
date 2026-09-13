//! Chain-state reads, exposed to the GUI and CLI through the core facade.
//!
//! Read-only by construction: designating a role or registering a candidate are
//! signed transactions the application deliberately does not perform.

pub use crate::roles::ChainRole;

pub use crate::chain_state::{
    designation_status, governance_snapshot, mempool_telemetry, peer_telemetry, CandidateStanding,
    ChainQueryError, DesignationStatus, GovernanceSnapshot, MempoolCongestion, MempoolTelemetry,
    PeerConnectivity, PeerEndpoint, PeerTelemetry, RoleDesignation,
};
