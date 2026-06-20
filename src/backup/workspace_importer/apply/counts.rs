pub(super) struct ProfileImportCounts {
    pub(super) remote_server_count: usize,
    pub(super) runtime_catalog_profile_count: usize,
    pub(super) runtime_signer_profile_count: usize,
    pub(super) neo_wallet_profile_count: usize,
    pub(super) fast_sync_snapshot_count: usize,
}

pub(super) struct NodeImportCounts {
    pub(super) created_nodes: usize,
    pub(super) updated_nodes: usize,
    pub(super) plugin_state_count: usize,
    pub(super) plugin_installation_count: usize,
}

impl NodeImportCounts {
    pub(super) fn empty() -> Self {
        Self {
            created_nodes: 0,
            updated_nodes: 0,
            plugin_state_count: 0,
            plugin_installation_count: 0,
        }
    }
}
