//! Durable node recovery metadata. Deadlines are Unix milliseconds; budgets
//! count claimed attempts, so an interrupted launch cannot refund an attempt.
use serde::{Deserialize, Serialize};

use super::{policy::delay_for_attempt, RestartOutcome, RestartPolicy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecoveryState {
    pub version: u8,
    pub attempts: u32,
    pub next_attempt_at_unix_ms: Option<u64>,
    pub claim: Option<String>,
    pub exhausted: bool,
}

impl Default for RecoveryState {
    fn default() -> Self {
        Self {
            version: 1,
            attempts: 0,
            next_attempt_at_unix_ms: None,
            claim: None,
            exhausted: false,
        }
    }
}

impl RecoveryState {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.version == 1
                && self.attempts <= 20
                && !(self.claim.is_some() && self.next_attempt_at_unix_ms.is_some())
                && !(self.exhausted
                    && (self.claim.is_some() || self.next_attempt_at_unix_ms.is_some()))
                && self
                    .claim
                    .as_ref()
                    .is_none_or(|claim| self.attempts > 0 && uuid::Uuid::parse_str(claim).is_ok()),
            "invalid node recovery state; stop the node to clear its recovery record"
        );
        Ok(())
    }

    pub fn schedule(&mut self, policy: RestartPolicy, now_ms: u64) -> RestartOutcome {
        self.claim = None;
        self.next_attempt_at_unix_ms = None;
        if !policy.enabled || policy.max_restart_attempts == 0 {
            return RestartOutcome::Disabled;
        }
        if self.attempts >= policy.max_restart_attempts {
            self.exhausted = true;
            return RestartOutcome::Exhausted {
                attempts: self.attempts,
            };
        }
        let attempt = self.attempts + 1;
        let delay = delay_for_attempt(policy, attempt);
        let millis = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX);
        self.next_attempt_at_unix_ms = Some(now_ms.saturating_add(millis));
        self.exhausted = false;
        RestartOutcome::Scheduled { attempt, delay }
    }

    pub fn apply_policy(&mut self, policy: RestartPolicy) {
        // Already claimed operations may finish; a policy edit never refunds
        // their budget or silently revives previously cancelled work.
        if !policy.enabled || self.attempts >= policy.max_restart_attempts {
            self.next_attempt_at_unix_ms = None;
        }
        if self.claim.is_none() && policy.enabled && self.attempts >= policy.max_restart_attempts {
            self.exhausted = true;
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RecoveryClaim {
    pub node_id: String,
    pub attempt: u32,
    pub token: String,
}

pub(crate) fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}
