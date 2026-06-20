use serde::{Deserialize, Serialize};

use super::{
    DeploymentArtifactManifest, DeploymentCommitteeManifest, DeploymentNodeManifest,
    DeploymentScriptsManifest, DeploymentSecretProvisioningManifest,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(in crate::private_network) struct DeploymentManifest {
    pub(in crate::private_network) schema_version: u32,
    pub(in crate::private_network) generated_at_unix: u64,
    pub(in crate::private_network) template: String,
    pub(in crate::private_network) runtime: String,
    pub(in crate::private_network) network: String,
    pub(in crate::private_network) network_magic: u32,
    pub(in crate::private_network) validators_count: u8,
    pub(in crate::private_network) seed_nodes: Vec<String>,
    pub(in crate::private_network) committee: DeploymentCommitteeManifest,
    #[serde(default)]
    pub(in crate::private_network) secret_provisioning: DeploymentSecretProvisioningManifest,
    pub(in crate::private_network) scripts: DeploymentScriptsManifest,
    #[serde(default)]
    pub(in crate::private_network) artifacts: Vec<DeploymentArtifactManifest>,
    pub(in crate::private_network) nodes: Vec<DeploymentNodeManifest>,
}
