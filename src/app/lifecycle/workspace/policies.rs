use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn reload_workspace_policies(&mut self) {
        self.reload_watchdog_policy();
        self.reload_runtime_upgrade_policy();
        self.reload_rpc_health_monitor_policy();
        self.reload_remote_federation_monitor_policy();
        self.reload_alert_routing_policy();
        self.reload_private_network_sidecar_policy();
    }

    fn reload_watchdog_policy(&mut self) {
        match self.repository.load_watchdog_policy() {
            Ok(policy) => {
                self.watchdog.update_policy(policy);
                self.watchdog_policy_draft = WatchdogPolicyDraft::from_policy(policy);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    fn reload_runtime_upgrade_policy(&mut self) {
        match self.repository.load_runtime_upgrade_policy() {
            Ok(policy) => {
                self.runtime_upgrade_policy = policy;
                self.runtime_upgrade_policy_draft =
                    RuntimeUpgradePolicyDraft::from_policy(&self.runtime_upgrade_policy);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    fn reload_rpc_health_monitor_policy(&mut self) {
        match self.repository.load_rpc_health_monitor_policy() {
            Ok(policy) => {
                self.rpc_health_monitor_policy = policy;
                self.rpc_health_monitor_policy_draft =
                    RpcHealthMonitorPolicyDraft::from_policy(self.rpc_health_monitor_policy);
                self.rpc_health_last_started.clear();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    fn reload_remote_federation_monitor_policy(&mut self) {
        match self.repository.load_remote_federation_monitor_policy() {
            Ok(policy) => {
                self.remote_federation_monitor_policy = policy;
                self.remote_federation_monitor_policy_draft =
                    RemoteFederationMonitorPolicyDraft::from_policy(
                        self.remote_federation_monitor_policy,
                    );
                self.remote_federation_last_started.clear();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    fn reload_alert_routing_policy(&mut self) {
        match self.repository.load_alert_routing_policy() {
            Ok(policy) => {
                self.alert_routing_policy = policy;
                self.alert_routing_policy_draft =
                    AlertRoutingPolicyDraft::from_policy(&self.alert_routing_policy);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    fn reload_private_network_sidecar_policy(&mut self) {
        match self
            .repository
            .load_private_network_allow_external_sidecars()
        {
            Ok(allow_external) => self.private_network_allow_external_sidecars = allow_external,
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
