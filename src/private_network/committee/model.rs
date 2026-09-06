use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::supervisor::ManagedProcessSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitteeRoster {
    pub signers: Vec<CommitteeSigner>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitteeHandoffSummary {
    pub required_signer_count: usize,
    pub signer_count: usize,
    pub missing_required_signer_count: usize,
    pub wallet_reference_count: usize,
    pub missing_wallet_reference_count: usize,
    pub endpoint_reference_count: usize,
    pub sidecar_command_count: usize,
    pub sidecar_command_plan_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitteeSidecarProcess {
    pub signer_label: String,
    pub public_key: String,
    pub wallet_path: Option<PathBuf>,
    pub signer_endpoint: Option<String>,
    pub log_path: PathBuf,
    pub process: ManagedProcessSpec,
}

/// Deployment handoff metadata for one private-network committee key.
///
/// This is deliberately not an application-level
/// [`crate::signing::SignerBackendProfile`]. A wallet path, health endpoint and
/// sidecar command may coexist because together they describe artifacts the
/// launch-pack operator must provision. They never select, inherit or fall
/// back to NeoNexus's process-wide signer backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitteeSigner {
    pub label: String,
    pub public_key: String,
    pub wallet_path: Option<PathBuf>,
    pub signer_endpoint: Option<String>,
    pub signer_command_template: Option<String>,
    pub signer_command: Option<String>,
    pub signer_command_plan: Option<SignerCommandPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SignerCommandPlan {
    pub execution_policy: String,
    pub binary: String,
    pub arguments: Vec<String>,
}
