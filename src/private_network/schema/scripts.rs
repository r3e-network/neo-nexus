use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(in crate::private_network) struct DeploymentScriptsManifest {
    pub(in crate::private_network) runbook: String,
    pub(in crate::private_network) preflight_unix: String,
    pub(in crate::private_network) preflight_windows: String,
    pub(in crate::private_network) health_unix: String,
    pub(in crate::private_network) health_windows: String,
    pub(in crate::private_network) start_unix: String,
    pub(in crate::private_network) stop_unix: String,
    pub(in crate::private_network) start_windows: String,
    pub(in crate::private_network) stop_windows: String,
}
