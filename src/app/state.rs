use super::*;

mod async_bus;
mod fleet;
mod operations_ui;
mod sections;
mod session;
mod toasts;

pub(in crate::app) use async_bus::AsyncProbeBus;
pub(in crate::app) use fleet::FleetUi;
pub(in crate::app) use operations_ui::OperationsUi;
pub(in crate::app) use sections::WorkspaceSections;
pub(in crate::app) use session::SessionUi;
pub(in crate::app) use toasts::render_toast_strip;

pub struct NeoNexusApp {
    pub(in crate::app) repository: Repository,
    pub(in crate::app) session: SessionUi,
    pub(in crate::app) fleet: FleetUi,
    pub(in crate::app) operations_ui: OperationsUi,
    pub(in crate::app) sections: WorkspaceSections,
    pub(in crate::app) async_bus: AsyncProbeBus,
    pub(in crate::app) supervisor: ProcessSupervisor,
    pub(in crate::app) watchdog: Watchdog,
    pub(in crate::app) watchdog_policy_draft: WatchdogPolicyDraft,
    pub(in crate::app) runtime_upgrade_policy: RuntimeUpgradePolicy,
    pub(in crate::app) runtime_upgrade_policy_draft: RuntimeUpgradePolicyDraft,
    pub(in crate::app) metrics: MetricsCollector,
    pub(in crate::app) metrics_snapshot: MetricsSnapshot,
    pub(in crate::app) snapshot_draft: SnapshotDraft,
    pub(in crate::app) snapshot_catalog_source: String,
    pub(in crate::app) snapshot_catalog_signature_source: String,
    pub(in crate::app) snapshot_catalog_public_key: String,
    pub(in crate::app) snapshot_catalog: Option<FastSyncSnapshotCatalog>,
    pub(in crate::app) snapshot_catalog_signature_verified: Option<bool>,
    pub(in crate::app) snapshot_catalog_bytes: u64,
    pub(in crate::app) runtime_package_draft: RuntimePackageDraft,
    pub(in crate::app) runtime_catalog_profile_label: String,
    pub(in crate::app) runtime_catalog_source: String,
    pub(in crate::app) runtime_catalog_signature_source: String,
    pub(in crate::app) runtime_catalog_public_key: String,
    pub(in crate::app) runtime_catalog_profiles: Vec<RuntimeCatalogProfile>,
    pub(in crate::app) runtime_signer_profile_label: String,
    pub(in crate::app) runtime_signer_public_key: String,
    pub(in crate::app) runtime_signer_profiles: Vec<RuntimeSignerProfile>,
    pub(in crate::app) neo_wallet_profiles: Vec<NeoWalletProfile>,
    pub(in crate::app) wallet_profile_source: String,
    pub(in crate::app) wallet_profile_id: String,
    pub(in crate::app) wallet_profile_label: String,
    pub(in crate::app) runtime_catalog: Option<RuntimeReleaseCatalog>,
    pub(in crate::app) runtime_catalog_signature_verified: Option<bool>,
    pub(in crate::app) runtime_catalog_bytes: u64,
    pub(in crate::app) remote_servers: Vec<RemoteServerProfile>,
    pub(in crate::app) remote_server_name: String,
    pub(in crate::app) remote_server_base_url: String,
    pub(in crate::app) remote_server_description: String,
    pub(in crate::app) remote_server_enabled: bool,
    pub(in crate::app) last_remote_server_probe: Option<RemoteServerProbeRecord>,
    pub(in crate::app) selected_plugin: Option<PluginId>,
    pub(in crate::app) plugin_package_source: String,
    pub(in crate::app) plugin_package_expected_sha256: String,
    pub(in crate::app) selected_snapshot: Option<String>,
    pub(in crate::app) selected_snapshot_catalog_entry: Option<String>,
    pub(in crate::app) selected_runtime_catalog_profile: Option<String>,
    pub(in crate::app) selected_runtime_signer_profile: Option<String>,
    pub(in crate::app) selected_neo_wallet_profile: Option<String>,
    pub(in crate::app) selected_runtime_installation: Option<String>,
    pub(in crate::app) selected_runtime_release: Option<String>,
    pub(in crate::app) selected_remote_server: Option<String>,
    pub(in crate::app) selected_role: NodeRole,
    pub(in crate::app) private_network_template: PrivateNetworkTemplate,
    pub(in crate::app) private_network_runtime: NodeType,
    pub(in crate::app) private_network_committee_keys: String,
    pub(in crate::app) private_network_signer_refs: String,
    pub(in crate::app) private_network_last_export_root: Option<PathBuf>,
    pub(in crate::app) private_network_last_validation: Option<PrivateNetworkLaunchPackValidation>,
    pub(in crate::app) private_network_sidecar_report:
        Option<PrivateNetworkLaunchPackSidecarReport>,
    pub(in crate::app) private_network_sidecar_health_report: Option<SidecarEndpointHealthReport>,
    pub(in crate::app) private_network_sidecar_pids: BTreeMap<String, u32>,
    pub(in crate::app) private_network_allow_external_sidecars: bool,
    pub(in crate::app) plugin_catalog: PluginCatalog,
    pub(in crate::app) workspace_integrity_report: Option<WorkspaceIntegrityReport>,
    pub(in crate::app) last_backup_validation: Option<WorkspaceBackupValidation>,
    pub(in crate::app) last_release_package: Option<ReleasePackage>,
    pub(in crate::app) last_release_verification: Option<ReleasePackageVerification>,
    pub(in crate::app) plugin_page: usize,
    pub(in crate::app) plugin_query: String,
    pub(in crate::app) plugin_enabled_filter: Option<bool>,
    pub(in crate::app) plugin_category_filter: Option<PluginCategory>,
    pub(in crate::app) config_page: usize,
    pub(in crate::app) log_page: usize,
    pub(in crate::app) log_query: String,
    pub(in crate::app) log_follow_tail: bool,
    pub(in crate::app) snapshot_page: usize,
    pub(in crate::app) snapshot_query: String,
    pub(in crate::app) snapshot_network_filter: Option<Network>,
    pub(in crate::app) snapshot_type_filter: Option<NodeType>,
    pub(in crate::app) snapshot_verified_filter: Option<bool>,
    pub(in crate::app) snapshot_cached_filter: Option<bool>,
    pub(in crate::app) snapshot_catalog_page: usize,
    pub(in crate::app) snapshot_catalog_query: String,
    pub(in crate::app) snapshot_catalog_network_filter: Option<Network>,
    pub(in crate::app) snapshot_catalog_type_filter: Option<NodeType>,
    pub(in crate::app) runtime_page: usize,
    pub(in crate::app) runtime_inventory_query: String,
    pub(in crate::app) runtime_inventory_type_filter: Option<NodeType>,
    pub(in crate::app) runtime_inventory_signed_filter: Option<bool>,
    pub(in crate::app) runtime_inventory_platform_filter: Option<bool>,
    pub(in crate::app) runtime_catalog_page: usize,
    pub(in crate::app) runtime_catalog_query: String,
    pub(in crate::app) runtime_catalog_type_filter: Option<NodeType>,
    pub(in crate::app) runtime_catalog_platform_filter: Option<bool>,
    pub(in crate::app) monitor_process_page: usize,
    pub(in crate::app) monitor_process_query: String,
    pub(in crate::app) monitor_process_state_filter: Option<ProcessStateFilter>,
    pub(in crate::app) monitor_process_high_cpu_filter: bool,
    pub(in crate::app) monitor_process_high_memory_filter: bool,
    pub(in crate::app) selected_monitor_process: Option<String>,
    pub(in crate::app) wallet_profile_page: usize,
    pub(in crate::app) wallet_profile_query: String,
    pub(in crate::app) wallet_profile_used_filter: Option<bool>,
    pub(in crate::app) remote_server_page: usize,
    pub(in crate::app) remote_server_query: String,
    pub(in crate::app) remote_server_enabled_filter: Option<bool>,
    pub(in crate::app) remote_probe_history_page: usize,
    pub(in crate::app) remote_probe_history_query: String,
    pub(in crate::app) remote_probe_history_status_filter: Option<RemoteProbeStatus>,
}

impl NeoNexusApp {
    /// Reset fleet list paging when a fleet-wide filter changes.
    pub(in crate::app) fn reset_fleet_paging(&mut self) {
        self.fleet.reset_paging();
    }

    pub(in crate::app) fn set_fleet_status_filter(&mut self, status: Option<NodeStatus>) {
        self.fleet.set_status_filter(status);
    }

    pub(in crate::app) fn select_fleet_node(&mut self, id: Option<String>) {
        if self.fleet.selected_node == id {
            return;
        }
        self.fleet.select_node(id);
        self.selected_plugin = None;
        self.plugin_page = 0;
        self.config_page = 0;
        self.log_page = 0;
    }

    pub(in crate::app) fn running_node_count(&self) -> usize {
        self.fleet.running_count()
    }
}
