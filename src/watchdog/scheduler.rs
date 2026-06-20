use std::{collections::BTreeMap, time::Instant};

use super::{
    model::{RestartAttempt, RestartOutcome, WatchdogStatus},
    policy::{delay_for_attempt, RestartPolicy},
    state::RestartState,
};

#[derive(Debug, Clone)]
pub struct Watchdog {
    policy: RestartPolicy,
    states: BTreeMap<String, RestartState>,
}

impl Watchdog {
    pub fn new(policy: RestartPolicy) -> Self {
        Self {
            policy,
            states: BTreeMap::new(),
        }
    }

    pub fn policy(&self) -> RestartPolicy {
        self.policy
    }

    pub fn update_policy(&mut self, policy: RestartPolicy) {
        self.policy = policy.normalized();
        self.states.clear();
    }

    pub fn clear(&mut self, node_id: &str) {
        self.states.remove(node_id);
    }

    pub fn record_failure(&mut self, node_id: &str, now: Instant) -> RestartOutcome {
        let policy = self.policy;
        if !policy.enabled || policy.max_restart_attempts == 0 {
            self.states.remove(node_id);
            return RestartOutcome::Disabled;
        }

        let state = self
            .states
            .entry(node_id.to_string())
            .or_insert_with(RestartState::new);
        let next_attempt = state.attempts.saturating_add(1);

        if next_attempt > self.policy.max_restart_attempts {
            state.exhausted = true;
            state.next_restart_at = None;
            return RestartOutcome::Exhausted {
                attempts: self.policy.max_restart_attempts,
            };
        }

        let delay = delay_for_attempt(policy, next_attempt);
        state.attempts = next_attempt;
        state.exhausted = false;
        state.next_restart_at = Some(now + delay);
        RestartOutcome::Scheduled {
            attempt: next_attempt,
            delay,
        }
    }

    pub fn due_restarts(&mut self, now: Instant) -> Vec<RestartAttempt> {
        self.states
            .iter_mut()
            .filter_map(|(node_id, state)| {
                let due = state
                    .next_restart_at
                    .is_some_and(|restart_at| restart_at <= now);
                if due {
                    state.next_restart_at = None;
                    Some(RestartAttempt {
                        node_id: node_id.clone(),
                        attempt: state.attempts,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn status(&self, node_id: &str, now: Instant) -> WatchdogStatus {
        let Some(state) = self.states.get(node_id) else {
            return WatchdogStatus::Idle;
        };

        if state.exhausted {
            return WatchdogStatus::Exhausted {
                attempts: self.policy.max_restart_attempts,
            };
        }

        state
            .next_restart_at
            .map_or(WatchdogStatus::Idle, |restart_at| WatchdogStatus::Pending {
                attempt: state.attempts,
                remaining: restart_at.saturating_duration_since(now),
            })
    }

    pub fn has_pending_restart(&self) -> bool {
        self.states
            .values()
            .any(|state| state.next_restart_at.is_some())
    }
}
