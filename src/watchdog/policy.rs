use std::time::Duration;

pub const DEFAULT_MAX_RESTART_ATTEMPTS: u32 = 3;
pub const DEFAULT_BASE_DELAY: Duration = Duration::from_secs(2);
pub const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(30);

pub fn default_restart_policy() -> RestartPolicy {
    RestartPolicy::new(
        DEFAULT_MAX_RESTART_ATTEMPTS,
        DEFAULT_BASE_DELAY,
        DEFAULT_MAX_DELAY,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestartPolicy {
    pub enabled: bool,
    pub max_restart_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl RestartPolicy {
    pub fn new(max_restart_attempts: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self::with_enabled(true, max_restart_attempts, base_delay, max_delay)
    }

    pub fn with_enabled(
        enabled: bool,
        max_restart_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        Self {
            enabled,
            max_restart_attempts,
            base_delay,
            max_delay,
        }
        .normalized()
    }

    pub fn normalized(self) -> Self {
        let base_delay = self.base_delay.max(Duration::from_secs(1));
        let max_delay = self.max_delay.max(base_delay);
        Self {
            enabled: self.enabled,
            max_restart_attempts: self.max_restart_attempts.min(20),
            base_delay,
            max_delay,
        }
    }

    pub fn describe(self) -> String {
        if !self.enabled {
            return "disabled".to_string();
        }

        format!(
            "{} attempts, {}s base, {}s cap",
            self.max_restart_attempts,
            self.base_delay.as_secs(),
            self.max_delay.as_secs()
        )
    }
}

pub(super) fn delay_for_attempt(policy: RestartPolicy, attempt: u32) -> Duration {
    let shift = attempt.saturating_sub(1).min(31);
    let factor = 1_u32.checked_shl(shift).unwrap_or(u32::MAX);
    policy
        .base_delay
        .saturating_mul(factor)
        .min(policy.max_delay)
}
