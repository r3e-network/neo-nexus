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
