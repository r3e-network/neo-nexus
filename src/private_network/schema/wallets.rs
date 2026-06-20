use serde::{Deserialize, Serialize};

use crate::private_network::SignerCommandPlan;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(in crate::private_network) struct DeploymentSecretProvisioningManifest {
    pub(in crate::private_network) schema_version: u32,
    pub(in crate::private_network) policy: String,
    pub(in crate::private_network) wallet_provisioning_file: String,
    pub(in crate::private_network) wallet_instructions_file: String,
    pub(in crate::private_network) recommended_wallet_root: String,
    pub(in crate::private_network) required_wallet_count: usize,
    pub(in crate::private_network) wallet_reference_count: usize,
    pub(in crate::private_network) missing_wallet_reference_count: usize,
    pub(in crate::private_network) generated_secret_count: usize,
}

impl Default for DeploymentSecretProvisioningManifest {
    fn default() -> Self {
        Self {
            schema_version: 0,
            policy: String::new(),
            wallet_provisioning_file: String::new(),
            wallet_instructions_file: String::new(),
            recommended_wallet_root: String::new(),
            required_wallet_count: 0,
            wallet_reference_count: 0,
            missing_wallet_reference_count: 0,
            generated_secret_count: usize::MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(in crate::private_network) struct WalletProvisioningDocument {
    pub(in crate::private_network) schema_version: u32,
    pub(in crate::private_network) generated_at_unix: u64,
    pub(in crate::private_network) runtime: String,
    pub(in crate::private_network) template: String,
    pub(in crate::private_network) network_magic: u32,
    pub(in crate::private_network) secret_material_policy: String,
    pub(in crate::private_network) generated_secret_count: usize,
    pub(in crate::private_network) required_wallet_count: usize,
    pub(in crate::private_network) wallet_reference_count: usize,
    pub(in crate::private_network) missing_wallet_reference_count: usize,
    pub(in crate::private_network) entries: Vec<WalletProvisioningEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(in crate::private_network) struct WalletProvisioningEntry {
    pub(in crate::private_network) label: String,
    pub(in crate::private_network) public_key: String,
    pub(in crate::private_network) wallet_path: Option<String>,
    pub(in crate::private_network) recommended_wallet_path: String,
    pub(in crate::private_network) path_scope: String,
    pub(in crate::private_network) action: String,
    pub(in crate::private_network) signer_endpoint: Option<String>,
    pub(in crate::private_network) signer_command_template: Option<String>,
    pub(in crate::private_network) signer_command_plan: Option<SignerCommandPlan>,
}
