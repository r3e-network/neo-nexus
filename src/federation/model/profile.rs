use super::super::endpoints::{
    public_endpoint_url, PUBLIC_NODES_PATH, PUBLIC_STATUS_PATH, PUBLIC_SYSTEM_METRICS_PATH,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRemoteServerProfile {
    pub name: String,
    pub base_url: String,
    pub description: String,
    pub enabled: bool,
}

impl Default for NewRemoteServerProfile {
    fn default() -> Self {
        Self {
            name: "Remote NeoNexus".to_string(),
            base_url: "https://".to_string(),
            description: String::new(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteServerProfile {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub description: String,
    pub enabled: bool,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
}

impl RemoteServerProfile {
    pub fn public_status_url(&self) -> String {
        public_endpoint_url(&self.base_url, PUBLIC_STATUS_PATH)
    }

    pub fn public_nodes_url(&self) -> String {
        public_endpoint_url(&self.base_url, PUBLIC_NODES_PATH)
    }

    pub fn public_system_metrics_url(&self) -> String {
        public_endpoint_url(&self.base_url, PUBLIC_SYSTEM_METRICS_PATH)
    }
}
