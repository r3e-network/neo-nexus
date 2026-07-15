use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn save_rpc_health_monitor_policy(&mut self) {
        if let Some(message) = self.rpc_health_monitor_policy_draft.validation_message() {
            self.session.notice = Some(message.to_string());
            return;
        }

        let policy = self.rpc_health_monitor_policy_draft.to_policy();
        match self.repository.save_rpc_health_monitor_policy(policy) {
            Ok(()) => {
                self.rpc_health_monitor_policy = policy.normalized();
                self.rpc_health_monitor_policy_draft =
                    RpcHealthMonitorPolicyDraft::from_policy(self.rpc_health_monitor_policy);
                self.rpc_health_last_started.clear();
                let message = format!(
                    "RPC health monitor policy saved: {}",
                    self.rpc_health_monitor_policy.describe()
                );
                self.record_event_notice(
                    EventKind::RpcHealthMonitorPolicyUpdated,
                    EventSeverity::Info,
                    message,
                );
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn reset_rpc_health_monitor_policy_draft(&mut self) {
        self.rpc_health_monitor_policy_draft =
            RpcHealthMonitorPolicyDraft::from_policy(self.rpc_health_monitor_policy);
        self.session.notice = Some("RPC health monitor policy draft reset".to_string());
    }

    pub(in crate::app) fn save_remote_federation_monitor_policy(&mut self) {
        if let Some(message) = self
            .remote_federation_monitor_policy_draft
            .validation_message()
        {
            self.session.notice = Some(message.to_string());
            return;
        }

        let policy = self.remote_federation_monitor_policy_draft.to_policy();
        match self
            .repository
            .save_remote_federation_monitor_policy(policy)
        {
            Ok(()) => {
                self.remote_federation_monitor_policy = policy.normalized();
                self.remote_federation_monitor_policy_draft =
                    RemoteFederationMonitorPolicyDraft::from_policy(
                        self.remote_federation_monitor_policy,
                    );
                self.remote_federation_last_started.clear();
                let message = format!(
                    "Remote Federation monitor policy saved: {}",
                    self.remote_federation_monitor_policy.describe()
                );
                self.record_event_notice(
                    EventKind::RemoteFederationMonitorPolicyUpdated,
                    EventSeverity::Info,
                    message,
                );
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn reset_remote_federation_monitor_policy_draft(&mut self) {
        self.remote_federation_monitor_policy_draft =
            RemoteFederationMonitorPolicyDraft::from_policy(self.remote_federation_monitor_policy);
        self.session.notice = Some("Remote Federation monitor policy draft reset".to_string());
    }
}
