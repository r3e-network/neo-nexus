//! High-level workspace data queries for frontends. Frontends read workspace
//! collections (snapshots, event journal) through these operations instead of
//! reaching into the repository's row API, so a view does not query SQLite
//! directly during paint and the persistence layer stays behind the core facade.

use anyhow::Result;

use crate::{
    events::RuntimeEvent, repository::Repository, rpc_health::RpcHealthRecord,
    snapshots::FastSyncSnapshot, types::NodeConfig,
};

use super::operations::RuntimeEventFilter;

/// Read-only workspace service shared by the web and CLI adapters. Keeping
/// these queries here prevents presentation handlers from accumulating SQL
/// knowledge and gives future pagination/caching one boundary.
///
/// This service cannot change the workspace: mutations live in
/// [`super::workspace_commands::WorkspaceCommands`].
#[derive(Clone)]
pub struct WorkspaceQueries {
    repository: Repository,
}

impl WorkspaceQueries {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    // ── Reads ────────────────────────────────────────────────────────────────

    pub fn list_nodes(&self) -> Result<Vec<NodeConfig>> {
        self.repository.list_nodes()
    }

    pub fn list_plugin_states(&self, node_id: &str) -> Result<Vec<crate::catalog::PluginState>> {
        self.repository.list_plugin_states(node_id)
    }

    pub fn load_node_role(&self, node_id: &str) -> Result<Option<crate::roles::NodeRole>> {
        self.repository.load_node_role(node_id)
    }

    /// Alias for [`Self::list_fast_sync_snapshots`].
    #[inline]
    pub fn list_snapshots(&self) -> Result<Vec<FastSyncSnapshot>> {
        self.list_fast_sync_snapshots()
    }

    pub fn list_recent_events(&self, limit: usize) -> Result<Vec<RuntimeEvent>> {
        self.repository.list_recent_events(limit)
    }

    pub fn list_events(&self, filter: RuntimeEventFilter) -> Result<Vec<RuntimeEvent>> {
        self.repository.list_events(filter)
    }

    /// Nodes the watchdog stopped retrying and that have not come back.
    ///
    /// Read from the journal rather than from the engine's in-memory ledger: a
    /// page is served by a different thread, and a surface that reports "the
    /// watchdog is fine" needs to survive a restart of the process that knows
    /// otherwise. A node that has since been started by hand is `Running` and
    /// drops out, which is the question an operator is actually asking.
    pub fn nodes_the_watchdog_gave_up_on(&self) -> Result<Vec<NodeConfig>> {
        let exhausted: std::collections::BTreeSet<String> = self
            .repository
            .list_events(RuntimeEventFilter::of_kind(
                crate::events::EventKind::WatchdogExhausted,
                200,
            ))?
            .into_iter()
            .filter_map(|event| event.node_id)
            .collect();
        Ok(self
            .repository
            .list_nodes()?
            .into_iter()
            .filter(|node| exhausted.contains(&node.id) && !node.status.is_active())
            .collect())
    }

    pub fn count_events(&self, filter: &RuntimeEventFilter) -> Result<usize> {
        self.repository.count_events(filter)
    }

    pub fn latest_node_rpc_health(&self, node_id: &str) -> Result<Option<RpcHealthRecord>> {
        crate::core::node_health::latest_node_rpc_health(&self.repository, node_id)
    }

    /// Every node's chain state: the stored verdict plus the numbers behind it.
    ///
    /// Fleet-wide because a node's lag is measured against the highest height
    /// among the nodes sharing its chain, so one node's answer needs the group.
    pub fn fleet_chain_view(
        &self,
        nodes: &[NodeConfig],
        now_unix: u64,
    ) -> Result<Vec<crate::core::node_health::NodeChainView>> {
        crate::core::node_health::fleet_chain_view(&self.repository, nodes, now_unix)
    }

    pub fn node_chain_view(
        &self,
        nodes: &[NodeConfig],
        node_id: &str,
        now_unix: u64,
    ) -> Result<Option<crate::core::node_health::NodeChainView>> {
        crate::core::node_health::node_chain_view(&self.repository, nodes, node_id, now_unix)
    }

    pub fn node_health_timeline(
        &self,
        node_id: &str,
        limit: usize,
    ) -> Result<Vec<crate::observe::HealthTransition>> {
        crate::core::node_health::node_health_timeline(&self.repository, node_id, limit)
    }

    pub fn node_sample_history(
        &self,
        node_id: &str,
        limit: usize,
    ) -> Result<Vec<crate::observe::NodeSample>> {
        crate::core::node_health::node_sample_history(&self.repository, node_id, limit)
    }

    pub fn dashboard_summary(&self) -> Result<crate::dashboard::DashboardSummary> {
        crate::dashboard::DashboardSummary::load(&self.repository)
    }

    pub fn node_rpc_health_history(
        &self,
        node_id: &str,
        limit: usize,
    ) -> Result<Vec<RpcHealthRecord>> {
        crate::core::node_health::node_rpc_health_history(&self.repository, node_id, limit)
    }

    pub fn export_backup(
        &self,
        output_dir: impl AsRef<std::path::Path>,
        version: &str,
    ) -> Result<crate::backup::WorkspaceBackupExport> {
        crate::backup::WorkspaceBackupExporter::write(&self.repository, output_dir, version)
    }

    pub fn list_remote_servers(&self) -> Result<Vec<crate::federation::RemoteServerProfile>> {
        self.repository.list_remote_servers()
    }

    pub fn latest_remote_server_probe(
        &self,
        profile_id: &str,
    ) -> Result<Option<crate::federation::RemoteServerProbeRecord>> {
        self.repository.latest_remote_server_probe(profile_id)
    }

    pub fn list_neo_wallet_profiles(&self) -> Result<Vec<crate::wallet::NeoWalletProfile>> {
        self.repository.list_neo_wallet_profiles()
    }

    pub fn list_runtime_catalog_profiles(
        &self,
    ) -> Result<Vec<crate::runtime::RuntimeCatalogProfile>> {
        self.repository.list_runtime_catalog_profiles()
    }

    pub fn list_runtime_installations(&self) -> Result<Vec<crate::runtime::RuntimeInstallation>> {
        self.repository.list_runtime_installations()
    }

    pub fn list_fast_sync_snapshots(&self) -> Result<Vec<FastSyncSnapshot>> {
        self.repository.list_fast_sync_snapshots()
    }

    pub fn load_alert_routing_policy(&self) -> Result<crate::alerts::AlertRoutingPolicy> {
        self.repository.load_alert_routing_policy()
    }

    pub fn load_runtime_upgrade_policy(&self) -> Result<crate::runtime::RuntimeUpgradePolicy> {
        self.repository.load_runtime_upgrade_policy()
    }

    pub fn load_node_signer_key(
        &self,
        node_id: &str,
    ) -> Result<Option<crate::signing::SignerKeyRef>> {
        self.repository.load_node_signer_key(node_id)
    }

    pub fn list_all_signer_bindings(&self) -> Result<Vec<(String, crate::signing::SignerKeyRef)>> {
        self.repository.list_all_signer_bindings()
    }

    pub fn find_node_by_signer_key(
        &self,
        backend_id: &str,
        key_id: &str,
    ) -> Result<Option<String>> {
        self.repository.find_node_by_signer_key(backend_id, key_id)
    }

    /// Resolve a presented API-token secret, or `None` when it matches nothing.
    /// Reading, not writing: a failed lookup leaves the workspace untouched.
    pub fn verify_token_secret(
        &self,
        provided_secret: &str,
    ) -> Result<Option<crate::wallet::ApiToken>> {
        self.repository.verify_token_secret(provided_secret)
    }

    pub fn list_api_tokens(&self) -> Result<Vec<crate::wallet::ApiToken>> {
        self.repository.list_api_tokens()
    }

    pub fn load_watchdog_policy(&self) -> Result<crate::watchdog::RestartPolicy> {
        self.repository.load_watchdog_policy()
    }

    pub fn load_rpc_health_monitor_policy(
        &self,
    ) -> Result<crate::rpc_health::RpcHealthMonitorPolicy> {
        self.repository.load_rpc_health_monitor_policy()
    }

    pub fn load_remote_federation_monitor_policy(
        &self,
    ) -> Result<crate::federation::RemoteFederationMonitorPolicy> {
        self.repository.load_remote_federation_monitor_policy()
    }

    pub fn load_app_ui_density(&self) -> Result<Option<String>> {
        self.repository.load_app_ui_density()
    }

    pub fn list_remote_server_probes(
        &self,
        remote_server_id: &str,
        limit: usize,
    ) -> Result<Vec<crate::federation::RemoteServerProbeRecord>> {
        self.repository
            .list_remote_server_probes(remote_server_id, limit)
    }

    pub fn list_alert_deliveries(&self, limit: usize) -> Result<Vec<crate::alerts::AlertDelivery>> {
        self.repository.list_alert_deliveries(limit)
    }

    pub fn load_hermes_agent(
        &self,
        node_id: &str,
    ) -> Result<Option<crate::agents::HermesAgentAssociation>> {
        self.repository.load_hermes_agent(node_id)
    }

    pub fn list_hermes_agents(&self) -> Result<Vec<crate::agents::HermesAgentAssociation>> {
        self.repository.list_hermes_agents()
    }
}
