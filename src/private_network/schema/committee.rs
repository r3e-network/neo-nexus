use serde::{Deserialize, Serialize};

use crate::private_network::SignerCommandPlan;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(in crate::private_network) struct DeploymentCommitteeManifest {
    pub(in crate::private_network) signer_count: usize,
    pub(in crate::private_network) wallet_reference_count: usize,
    pub(in crate::private_network) endpoint_reference_count: usize,
    #[serde(default)]
    pub(in crate::private_network) sidecar_command_count: usize,
    pub(in crate::private_network) public_keys: Vec<String>,
    pub(in crate::private_network) secret_material_policy: String,
    pub(in crate::private_network) preflight_policy: String,
    pub(in crate::private_network) signers: Vec<DeploymentCommitteeSignerManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(in crate::private_network) struct DeploymentCommitteeSignerManifest {
    pub(in crate::private_network) label: String,
    pub(in crate::private_network) public_key: String,
    pub(in crate::private_network) wallet_path: Option<String>,
    pub(in crate::private_network) signer_endpoint: Option<String>,
    #[serde(default)]
    pub(in crate::private_network) signer_command_template: Option<String>,
    #[serde(default)]
    pub(in crate::private_network) signer_command: Option<String>,
    #[serde(default)]
    pub(in crate::private_network) signer_command_plan: Option<SignerCommandPlan>,
}
