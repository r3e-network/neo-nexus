mod catalog;
mod execution;
mod model;
mod persistence;

use crate::app::domain::{EventKind, EventSeverity};

use super::{current_unix_time, NeoNexusApp};

use model::RuntimeUpgradeRunMode;

impl NeoNexusApp {
    pub(super) fn run_due_runtime_upgrade_policy(&mut self) {
        let Ok(now_unix) = current_unix_time() else {
            return;
        };
        if self.runtime_upgrade_policy.is_due(now_unix) {
            self.run_runtime_upgrade_policy(now_unix, RuntimeUpgradeRunMode::Scheduled);
        }
    }

    pub(super) fn run_runtime_upgrade_policy_now(&mut self) {
        match current_unix_time() {
            Ok(now_unix) => {
                self.run_runtime_upgrade_policy(now_unix, RuntimeUpgradeRunMode::Manual)
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    fn run_runtime_upgrade_policy(&mut self, now_unix: u64, mode: RuntimeUpgradeRunMode) {
        if !self.runtime_upgrade_policy.enabled {
            if matches!(mode, RuntimeUpgradeRunMode::Manual) {
                self.notice =
                    Some("Enable the runtime upgrade policy before running it".to_string());
            }
            return;
        }

        if let Err(error) = self.persist_runtime_upgrade_policy_state(
            self.runtime_upgrade_policy.with_checked_at(now_unix),
        ) {
            self.notice = Some(error.to_string());
            return;
        }

        match self.execute_runtime_upgrade_policy() {
            Ok(summary) => {
                let updated_policy = if summary.upgraded > 0 {
                    self.runtime_upgrade_policy.with_applied_at(now_unix)
                } else {
                    self.runtime_upgrade_policy.clone()
                };
                if let Err(error) = self.persist_runtime_upgrade_policy_state(updated_policy) {
                    self.notice = Some(error.to_string());
                    return;
                }
                let message = summary.message(mode);
                self.record_event(
                    None,
                    None,
                    EventKind::RuntimeUpgradePolicyRun,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.notice = Some(message);
            }
            Err(error) => {
                let message = format!("Runtime upgrade policy {} failed: {error}", mode.label());
                self.record_event(
                    None,
                    None,
                    EventKind::RuntimeUpgradePolicyRun,
                    EventSeverity::Warning,
                    message.clone(),
                );
                self.notice = Some(message);
            }
        }
    }
}
