use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn save_runtime_upgrade_policy(&mut self) {
        if let Some(message) = self.runtime_upgrade_policy_draft.validation_message() {
            self.notice = Some(message.to_string());
            return;
        }

        let policy = self
            .runtime_upgrade_policy_draft
            .to_policy(&self.runtime_upgrade_policy);
        match self.repository.save_runtime_upgrade_policy(&policy) {
            Ok(()) => {
                self.runtime_upgrade_policy = policy;
                self.runtime_upgrade_policy_draft =
                    RuntimeUpgradePolicyDraft::from_policy(&self.runtime_upgrade_policy);
                let message = format!(
                    "Runtime upgrade policy saved: {}",
                    self.runtime_upgrade_policy.describe()
                );
                self.record_event(
                    None,
                    None,
                    EventKind::RuntimeUpgradePolicyUpdated,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn reset_runtime_upgrade_policy_draft(&mut self) {
        self.runtime_upgrade_policy_draft =
            RuntimeUpgradePolicyDraft::from_policy(&self.runtime_upgrade_policy);
        self.notice = Some("Runtime upgrade policy draft reset".to_string());
    }
}
