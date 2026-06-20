use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(in crate::private_network) struct DeploymentArtifactManifest {
    pub(in crate::private_network) label: String,
    pub(in crate::private_network) path: String,
    pub(in crate::private_network) sha256: String,
    pub(in crate::private_network) bytes: u64,
}
