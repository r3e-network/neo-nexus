use std::time::Duration;

use crate::watchdog::RestartPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) struct WatchdogPolicyDraft {
    pub(in crate::app) enabled: bool,
    pub(in crate::app) max_restart_attempts: u32,
    pub(in crate::app) base_delay_seconds: u64,
    pub(in crate::app) max_delay_seconds: u64,
}

impl WatchdogPolicyDraft {
    pub(in crate::app) fn from_policy(policy: RestartPolicy) -> Self {
        Self {
            enabled: policy.enabled,
            max_restart_attempts: policy.max_restart_attempts,
            base_delay_seconds: policy.base_delay.as_secs(),
            max_delay_seconds: policy.max_delay.as_secs(),
        }
    }

    pub(in crate::app) fn to_policy(self) -> RestartPolicy {
        RestartPolicy::with_enabled(
            self.enabled,
            self.max_restart_attempts,
            Duration::from_secs(self.base_delay_seconds),
            Duration::from_secs(self.max_delay_seconds),
        )
    }

    pub(in crate::app) fn validation_message(self) -> Option<&'static str> {
        if self.enabled && self.max_restart_attempts == 0 {
            return Some("Enabled watchdog policy needs at least one restart attempt");
        }
        if self.base_delay_seconds == 0 {
            return Some("Base delay must be at least 1 second");
        }
        if self.max_delay_seconds < self.base_delay_seconds {
            return Some("Max delay must be greater than or equal to base delay");
        }
        None
    }

    pub(in crate::app) fn differs_from(self, policy: RestartPolicy) -> bool {
        self != Self::from_policy(policy)
    }
}
