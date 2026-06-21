use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn save_private_network_sidecar_execution_policy(&mut self) {
        match self
            .repository
            .save_private_network_allow_external_sidecars(
                self.private_network_allow_external_sidecars,
            ) {
            Ok(()) => {
                let label =
                    sidecar_execution_policy_label(self.private_network_allow_external_sidecars);
                let message = format!("Sidecar execution policy saved: {label}");
                self.record_event_notice(
                    EventKind::PrivateNetworkSignerSidecarPolicyUpdated,
                    EventSeverity::Info,
                    message,
                );
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
