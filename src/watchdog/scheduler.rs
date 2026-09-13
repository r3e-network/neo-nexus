use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use super::{
    model::{RestartAttempt, RestartOutcome, WatchdogStatus},
    policy::RestartPolicy,
    state::RestartState,
};
use rand::Rng;

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
        let sample = if self.policy.jitter_enabled {
            rand::thread_rng().gen_range(0.0..=1.0)
        } else {
            0.5
        };
        self.record_failure_with_random_factor(node_id, now, sample)
    }

    fn record_failure_with_random_factor(
        &mut self,
        node_id: &str,
        now: Instant,
        sample: f64,
    ) -> RestartOutcome {
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

        let requested = policy.delay_for_attempt_with_random_factor(next_attempt, sample);
        let (restart_at, delay) = bounded_deadline(now, requested);
        state.attempts = next_attempt;
        state.exhausted = false;
        state.next_restart_at = Some(restart_at);

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

// Duration can exceed the platform's Instant range. Saturate only that exceptional
// case, retaining the actual scheduled delay in both the outcome and the state.
fn bounded_deadline(now: Instant, requested: Duration) -> (Instant, Duration) {
    if let Some(deadline) = now.checked_add(requested) {
        return (deadline, requested);
    }
    let mut lower = 0;
    let mut upper = requested.as_nanos();
    let mut deadline = now;
    let mut delay = Duration::ZERO;
    while lower < upper {
        let middle = lower + (upper - lower).div_ceil(2);
        let candidate = Duration::new(
            (middle / 1_000_000_000) as u64,
            (middle % 1_000_000_000) as u32,
        );
        if let Some(instant) = now.checked_add(candidate) {
            lower = middle;
            deadline = instant;
            delay = candidate;
        } else {
            upper = middle - 1;
        }
    }
    (deadline, delay)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_uses_production_delay_and_exact_due_boundary() {
        let policy =
            RestartPolicy::new(3, Duration::from_secs(2), Duration::from_secs(3)).with_jitter(true);
        for sample in [0.0, 0.5, 1.0] {
            let mut watchdog = Watchdog::new(policy);
            let now = Instant::now();
            for attempt in 1..=3 {
                let delay = policy.delay_for_attempt_with_random_factor(attempt, sample);
                assert_eq!(
                    watchdog.record_failure_with_random_factor("node", now, sample),
                    RestartOutcome::Scheduled { attempt, delay }
                );
                assert_eq!(
                    watchdog.status("node", now),
                    WatchdogStatus::Pending {
                        attempt,
                        remaining: delay,
                    }
                );
                assert!(watchdog
                    .due_restarts(now + delay - Duration::from_nanos(1))
                    .is_empty());
                let due = watchdog.due_restarts(now + delay);
                assert_eq!(due.len(), 1);
                assert_eq!(due[0].attempt, attempt);
                assert!(watchdog.due_restarts(now + delay).is_empty());
            }
            assert_eq!(
                watchdog.record_failure_with_random_factor("node", now, sample),
                RestartOutcome::Exhausted { attempts: 3 }
            );
        }
    }

    #[test]
    fn scheduler_public_random_path_obeys_bounds_and_due_state() {
        let policy =
            RestartPolicy::new(3, Duration::from_secs(2), Duration::from_secs(2)).with_jitter(true);
        let mut watchdog = Watchdog::new(policy);
        let now = Instant::now();
        for attempt in 1..=3 {
            let outcome = watchdog.record_failure("node", now);
            assert!(matches!(outcome, RestartOutcome::Scheduled { .. }));
            let RestartOutcome::Scheduled { delay, .. } = outcome else {
                unreachable!("asserted scheduled outcome");
            };
            assert!(delay >= Duration::from_millis(1700));
            assert!(delay <= policy.max_delay);
            assert_eq!(watchdog.due_restarts(now + delay)[0].attempt, attempt);
        }
    }

    #[test]
    fn scheduler_extreme_duration_saturates_instant_without_panicking() {
        for jitter in [false, true] {
            let policy = RestartPolicy::new(1, Duration::MAX, Duration::MAX).with_jitter(jitter);
            let now = Instant::now();
            let mut watchdog = Watchdog::new(policy);
            let outcome = watchdog.record_failure_with_random_factor("node", now, 1.0);
            assert!(matches!(outcome, RestartOutcome::Scheduled { .. }));
            let RestartOutcome::Scheduled { delay, .. } = outcome else {
                unreachable!("asserted scheduled outcome");
            };
            assert!(delay <= policy.max_delay);
            assert!(delay > Duration::ZERO);
            assert_eq!(
                watchdog.status("node", now),
                WatchdogStatus::Pending {
                    attempt: 1,
                    remaining: delay,
                }
            );
            let deadline = now
                .checked_add(delay)
                .unwrap_or_else(|| now + Duration::from_secs(1));
            assert_eq!(watchdog.due_restarts(deadline).len(), 1);
        }
    }
}
