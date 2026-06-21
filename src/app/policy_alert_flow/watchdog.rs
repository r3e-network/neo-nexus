use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn save_watchdog_policy(&mut self) {
        if let Some(message) = self.watchdog_policy_draft.validation_message() {
            self.notice = Some(message.to_string());
            return;
        }

        let policy = self.watchdog_policy_draft.to_policy();
        match self.repository.save_watchdog_policy(policy) {
            Ok(()) => {
                self.watchdog.update_policy(policy);
                self.watchdog_policy_draft = WatchdogPolicyDraft::from_policy(policy);
                let message = format!("Watchdog policy saved: {}", policy.describe());
                self.record_event_notice(
                    EventKind::WatchdogPolicyUpdated,
                    EventSeverity::Info,
                    message,
                );
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn reset_watchdog_policy_draft(&mut self) {
        self.watchdog_policy_draft = WatchdogPolicyDraft::from_policy(self.watchdog.policy());
        self.notice = Some("Watchdog policy draft reset".to_string());
    }
}
