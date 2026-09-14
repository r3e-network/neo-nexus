//! Workspace mutations for frontends.
//!
//! Everything here changes the workspace; everything in
//! [`super::workspace_queries::WorkspaceQueries`] only reads it. The split is
//! deliberate: a page that renders may call the query service freely, while a
//! handler that writes has to name the command service. That makes the write
//! paths easy to find when reviewing, and it is the boundary where auditing,
//! rate limiting, or an operation journal would later go.

use anyhow::Result;

use crate::{
    events::NewRuntimeEvent,
    repository::Repository,
    types::{NewNode, NodeConfig},
};

/// Write-side workspace service shared by the web and CLI adapters.
#[derive(Clone)]
pub struct WorkspaceCommands {
    repository: Repository,
}

impl WorkspaceCommands {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    /// Register or update a runtime catalog source.
    ///
    /// Until this was exposed, the only caller of the repository method was the
    /// backup importer — so a fresh workspace could never gain a catalog
    /// profile, `/runtimes` had nothing to install from, and the runtime
    /// upgrade policy required a `catalog_profile_id` that could only come from
    /// a backup of a workspace that could not have created one either.
    pub fn upsert_runtime_catalog_profile(
        &self,
        profile: &crate::runtime::RuntimeCatalogProfile,
    ) -> Result<()> {
        self.repository.upsert_runtime_catalog_profile(profile)
    }

    /// Stop the watchdog restarting one node, without touching the fleet.
    pub fn hold_node_restarts(&self, node_id: &str, reason: &str, at_unix: u64) -> Result<()> {
        self.repository.hold_node_restarts(node_id, reason, at_unix)
    }

    /// Let the watchdog manage this node again.
    pub fn release_node_restarts(&self, node_id: &str) -> Result<()> {
        self.repository.release_node_restarts(node_id)
    }

    /// Register a federation peer.
    ///
    /// `create_remote_server`, `update_remote_server` and `delete_remote_server`
    /// were called only from tests; the sole production insert was backup
    /// import. The page told the operator to "add one through the Rust API",
    /// while every neighbouring entity had an in-page create form.
    pub fn create_remote_server(
        &self,
        input: crate::federation::NewRemoteServerProfile,
    ) -> Result<crate::federation::RemoteServerProfile> {
        self.repository.create_remote_server(input)
    }

    pub fn update_remote_server(
        &self,
        id: &str,
        input: crate::federation::NewRemoteServerProfile,
    ) -> Result<crate::federation::RemoteServerProfile> {
        self.repository.update_remote_server(id, input)
    }

    /// Forget a federation peer, and its probe history with it.
    pub fn delete_remote_server(&self, id: &str) -> Result<()> {
        self.repository.delete_remote_server(id)
    }

    /// Write a support bundle: readiness, integrity, metrics and a redacted log
    /// diagnosis, checksummed.
    ///
    /// The exporter was complete and reachable only from `cli/actions/reports`.
    /// The operator filing a ticket is the one least likely to have a shell on
    /// the host, so a bundle they cannot produce is a second escalation rather
    /// than a diagnosis.
    pub fn export_support_bundle(
        &self,
        output_dir: impl AsRef<std::path::Path>,
        application_version: &str,
    ) -> Result<crate::support_bundle::WorkspaceSupportBundleExport> {
        crate::support_bundle::WorkspaceSupportBundleExporter::write(
            &self.repository,
            self.repository.db_path(),
            output_dir,
            application_version,
        )
    }

    /// Write the fleet readiness report.
    pub fn export_readiness_report(
        &self,
        output_dir: impl AsRef<std::path::Path>,
        diagnostics: &crate::diagnostics::FleetDiagnostics,
        application_version: &str,
    ) -> Result<crate::readiness_report::WorkspaceReadinessReportExport> {
        crate::readiness_report::WorkspaceReadinessReporter::write(
            output_dir,
            self.repository.db_path(),
            diagnostics,
            application_version,
        )
    }

    /// Check the workspace database against the schema this build expects.
    ///
    /// Computed for every support bundle and never shown live, so the one
    /// moment an operator wants it — "is my workspace itself damaged" — was the
    /// one moment they could not ask.
    pub fn check_workspace_integrity(
        &self,
        application_version: &str,
    ) -> Result<crate::workspace_integrity::WorkspaceIntegrityReport> {
        crate::workspace_integrity::WorkspaceIntegrityChecker::check(
            self.repository.db_path(),
            application_version,
        )
    }

    /// Rewrite a node's managed config from what this workspace would generate,
    /// keeping a timestamped copy of whatever was there.
    ///
    /// The counterpart to the drift check: seeing that a file has drifted and
    /// having no way to put it back is half a feature.
    pub fn reconcile_node_config(
        &self,
        node: &NodeConfig,
        config_path: &std::path::Path,
    ) -> Result<crate::config::ConfigReconciliationReport> {
        crate::config::ConfigReconciler::reconcile(node, config_path, true)
    }

    /// Apply a workspace archive from a path on this host.
    ///
    /// Behind the facade like every other mutation, so the console and the CLI
    /// restore a workspace through the same code rather than each having its
    /// own idea of what a restore does.
    pub fn import_backup(
        &self,
        archive_path: impl AsRef<std::path::Path>,
    ) -> Result<crate::backup::WorkspaceBackupImport> {
        crate::backup::WorkspaceBackupImporter::import_path(&self.repository, archive_path)
    }

    pub fn record_event(&self, event: NewRuntimeEvent) -> Result<crate::events::RuntimeEvent> {
        self.repository.record_event(event)
    }

    pub fn create_node(&self, input: NewNode) -> Result<NodeConfig> {
        self.repository.create_node(input)
    }

    pub fn update_node(&self, id: &str, input: NewNode) -> Result<NodeConfig> {
        self.repository.update_node(id, input)
    }

    pub fn delete_node(&self, id: &str) -> Result<()> {
        self.repository.delete_node(id)
    }

    pub fn create_api_token(
        &self,
        name: &str,
        permissions: Vec<crate::wallet::TokenPermission>,
        expires_at: Option<i64>,
    ) -> Result<(crate::wallet::ApiToken, String)> {
        self.repository
            .create_api_token(name, permissions, expires_at)
    }

    pub fn delete_api_token(&self, id: &str) -> Result<usize> {
        self.repository.delete_api_token(id)
    }

    pub fn save_watchdog_policy(&self, policy: crate::watchdog::RestartPolicy) -> Result<()> {
        self.repository.save_watchdog_policy(policy)
    }

    pub fn save_rpc_health_monitor_policy(
        &self,
        policy: crate::rpc_health::RpcHealthMonitorPolicy,
    ) -> Result<()> {
        self.repository.save_rpc_health_monitor_policy(policy)
    }

    pub fn save_app_ui_density(&self, density: &str) -> Result<()> {
        self.repository.save_app_ui_density(density)
    }

    pub fn upsert_neo_wallet_profile(
        &self,
        profile: &crate::wallet::NeoWalletProfile,
    ) -> Result<()> {
        self.repository.upsert_neo_wallet_profile(profile)
    }

    pub fn delete_neo_wallet_profile(&self, id: &str) -> Result<()> {
        self.repository.delete_neo_wallet_profile(id)
    }

    pub fn upsert_plugin_installation(
        &self,
        installation: &crate::plugins::PluginInstallation,
    ) -> Result<()> {
        self.repository.upsert_plugin_installation(installation)
    }

    pub fn apply_node_role_plan(&self, node_id: &str, plan: &crate::roles::RolePlan) -> Result<()> {
        self.repository.apply_node_role_plan(node_id, plan)
    }

    pub fn set_node_role(&self, node_id: &str, role: Option<crate::roles::NodeRole>) -> Result<()> {
        self.repository.set_node_role(node_id, role)
    }

    pub fn save_alert_routing_policy(
        &self,
        policy: crate::alerts::AlertRoutingPolicy,
    ) -> Result<()> {
        self.repository.save_alert_routing_policy(policy)
    }

    pub fn save_runtime_upgrade_policy(
        &self,
        policy: &crate::runtime::RuntimeUpgradePolicy,
    ) -> Result<()> {
        self.repository.save_runtime_upgrade_policy(policy)
    }

    pub fn save_remote_federation_monitor_policy(
        &self,
        policy: crate::federation::RemoteFederationMonitorPolicy,
    ) -> Result<()> {
        self.repository
            .save_remote_federation_monitor_policy(policy)
    }

    pub fn set_remote_server_enabled(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<crate::federation::RemoteServerProfile> {
        self.repository.set_remote_server_enabled(id, enabled)
    }

    pub fn upsert_fast_sync_snapshot(
        &self,
        input: crate::snapshots::NewFastSyncSnapshot,
    ) -> Result<crate::snapshots::FastSyncSnapshot> {
        self.repository.upsert_fast_sync_snapshot(input)
    }

    pub fn mark_fast_sync_snapshot_cached(
        &self,
        id: &str,
        cache: &crate::snapshots::SnapshotCache,
    ) -> Result<()> {
        self.repository.mark_fast_sync_snapshot_cached(id, cache)
    }

    pub fn mark_fast_sync_snapshot_verified(
        &self,
        id: &str,
        verification: &crate::snapshots::SnapshotVerification,
    ) -> Result<()> {
        self.repository
            .mark_fast_sync_snapshot_verified(id, verification)
    }

    pub fn mark_runtime_catalog_profile_loaded(
        &self,
        id: &str,
        load: &crate::runtime::RuntimeCatalogLoad,
    ) -> Result<()> {
        self.repository
            .mark_runtime_catalog_profile_loaded(id, load)
    }

    pub fn upsert_runtime_installation(
        &self,
        installation: &crate::runtime::RuntimeInstallation,
    ) -> Result<()> {
        self.repository.upsert_runtime_installation(installation)
    }

    pub fn set_node_signer_key(
        &self,
        node_id: &str,
        key: Option<&crate::signing::SignerKeyRef>,
    ) -> Result<()> {
        self.repository.set_node_signer_key(node_id, key)
    }

    pub fn set_plugin_enabled(
        &self,
        node_id: &str,
        plugin_id: crate::catalog::PluginId,
        enabled: bool,
    ) -> Result<()> {
        self.repository
            .set_plugin_enabled(node_id, plugin_id, enabled)
    }

    pub fn save_hermes_agent(&self, assoc: &crate::agents::HermesAgentAssociation) -> Result<()> {
        self.repository.save_hermes_agent(assoc)
    }

    /// Record a liveness report from an enrolled guest agent. `false` means the
    /// instance has no enabled association, so nothing was recorded.
    pub fn record_hermes_heartbeat(
        &self,
        node_id: &str,
        agent_version: Option<&str>,
    ) -> Result<bool> {
        self.repository
            .record_hermes_heartbeat(node_id, agent_version)
    }
}
