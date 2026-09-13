use std::time::Duration;

pub const DEFAULT_MAX_RESTART_ATTEMPTS: u32 = 3;
pub const DEFAULT_BASE_DELAY: Duration = Duration::from_secs(2);
pub const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(30);
pub const DEFAULT_JITTER_FACTOR: f64 = 0.15;

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
    pub jitter_enabled: bool,
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
            jitter_enabled: false, // Default to disabled for backward compatibility
        }
        .normalized()
    }

    pub fn with_jitter(self, jitter_enabled: bool) -> Self {
        Self {
            jitter_enabled,
            ..self
        }
        .normalized()
    }

    /// Preserve the persisted policy bounds: at least one second of base delay,
    /// a cap no smaller than the base, and at most twenty restart attempts.
    pub fn normalized(self) -> Self {
        let base_delay = self.base_delay.max(Duration::from_secs(1));
        let max_delay = self.max_delay.max(base_delay);
        Self {
            enabled: self.enabled,
            max_restart_attempts: self.max_restart_attempts.min(20),
            base_delay,
            max_delay,
            jitter_enabled: self.jitter_enabled,
        }
    }

    pub fn describe(self) -> String {
        if !self.enabled {
            return "disabled".to_string();
        }

        let jitter_status = if self.jitter_enabled() {
            "enabled"
        } else {
            "disabled"
        };

        format!(
            "{} attempts, {}s base, {}s cap (jitter: {})",
            self.max_restart_attempts,
            self.base_delay.as_secs(),
            self.max_delay.as_secs(),
            jitter_status
        )
    }

    /// Pure production delay calculation with an injected uniform sample in [0, 1].
    /// Attempt zero aliases attempt one. Samples outside the interval are clamped;
    /// NaN selects the midpoint. Jitter is uniform +/-15% before the final cap.
    /// Constructors normalize policy bounds; this method honors raw public fields,
    /// including zero and a cap below the base, without raising the supplied cap.
    pub fn delay_for_attempt_with_random_factor(self, attempt: u32, sample: f64) -> Duration {
        jittered_delay_for_attempt(self, attempt, sample)
    }

    /// Check if jitter is enabled
    pub fn jitter_enabled(&self) -> bool {
        self.jitter_enabled
    }
}

pub(super) fn delay_for_attempt(policy: RestartPolicy, attempt: u32) -> Duration {
    if policy.base_delay.is_zero() {
        return Duration::ZERO;
    }
    let nanos = 1_u128
        .checked_shl(attempt.saturating_sub(1))
        .and_then(|factor| policy.base_delay.as_nanos().checked_mul(factor))
        .unwrap_or(u128::MAX)
        .min(policy.max_delay.as_nanos());
    Duration::new(
        (nanos / 1_000_000_000) as u64,
        (nanos % 1_000_000_000) as u32,
    )
}

/// Calculate jittered delay for retry attempt
pub(super) fn jittered_delay_for_attempt(
    policy: RestartPolicy,
    attempt: u32,
    rng_factor: f64,
) -> Duration {
    let base_delay = delay_for_attempt(policy, attempt);

    if !policy.jitter_enabled() || base_delay.as_millis() == 0 {
        return base_delay;
    }

    let sample = if rng_factor.is_nan() {
        0.5
    } else {
        rng_factor.clamp(0.0, 1.0)
    };
    let random_multiplier = 1.0 - DEFAULT_JITTER_FACTOR + 2.0 * DEFAULT_JITTER_FACTOR * sample;
    // Retain millisecond truncation without narrowing Duration's full range to u64 ms.
    let millis = (base_delay.as_millis() as f64 * random_multiplier) as u128;
    let nanos = millis
        .saturating_mul(1_000_000)
        .min(policy.max_delay.as_nanos());
    Duration::new(
        (nanos / 1_000_000_000) as u64,
        (nanos % 1_000_000_000) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_delay_attempt_boundaries_and_saturation() {
        let policy = default_restart_policy();
        for (attempt, seconds) in [(0, 2), (1, 2), (2, 4), (5, 30), (u32::MAX, 30)] {
            assert_eq!(
                policy.delay_for_attempt_with_random_factor(attempt, 0.0),
                Duration::from_secs(seconds)
            );
        }
        let unbounded = RestartPolicy {
            base_delay: Duration::from_nanos(1),
            max_delay: Duration::MAX,
            ..policy
        };
        assert_eq!(
            unbounded.delay_for_attempt_with_random_factor(34, 0.5),
            Duration::from_nanos(1_u64 << 33)
        );
        assert_eq!(
            unbounded.delay_for_attempt_with_random_factor(u32::MAX, 0.5),
            Duration::MAX
        );
    }

    #[test]
    fn production_jitter_endpoints_and_final_cap() {
        let policy = default_restart_policy().with_jitter(true);
        assert_eq!(
            policy.delay_for_attempt_with_random_factor(1, 0.0),
            Duration::from_millis(1700)
        );
        assert_eq!(
            policy.delay_for_attempt_with_random_factor(1, 1.0),
            Duration::from_millis(2300)
        );
        assert_eq!(
            policy.delay_for_attempt_with_random_factor(u32::MAX, 0.0),
            Duration::from_millis(25500)
        );
        assert_eq!(
            policy.delay_for_attempt_with_random_factor(u32::MAX, 1.0),
            policy.max_delay
        );
        for (sample, normalized) in [
            (-1.0, 0.0),
            (2.0, 1.0),
            (f64::NEG_INFINITY, 0.0),
            (f64::INFINITY, 1.0),
            (f64::NAN, 0.5),
        ] {
            assert_eq!(
                policy.delay_for_attempt_with_random_factor(1, sample),
                policy.delay_for_attempt_with_random_factor(1, normalized)
            );
        }
    }

    #[test]
    fn production_delay_raw_zero_reversed_bounds_and_duration_extremes() {
        for (base, cap) in [
            (Duration::ZERO, Duration::MAX),
            (Duration::MAX, Duration::ZERO),
            (Duration::from_secs(10), Duration::from_secs(1)),
            (Duration::from_nanos(1), Duration::from_nanos(10)),
            (Duration::MAX, Duration::MAX),
        ] {
            for jitter_enabled in [false, true] {
                let policy = RestartPolicy {
                    base_delay: base,
                    max_delay: cap,
                    jitter_enabled,
                    ..default_restart_policy()
                };
                for attempt in [0, 1, 2, 32, 128, u32::MAX] {
                    for sample in [0.0, 0.5, 1.0] {
                        let delay = policy.delay_for_attempt_with_random_factor(attempt, sample);
                        assert!(delay <= cap);
                        if base.is_zero() || cap.is_zero() {
                            assert_eq!(delay, Duration::ZERO);
                        }
                    }
                }
            }
        }
        let extreme = RestartPolicy {
            base_delay: Duration::MAX,
            max_delay: Duration::MAX,
            jitter_enabled: true,
            ..default_restart_policy()
        };
        assert_eq!(
            extreme.delay_for_attempt_with_random_factor(1, 1.0),
            Duration::MAX
        );
        assert!(
            extreme.delay_for_attempt_with_random_factor(1, 0.0)
                > Duration::from_secs(u64::MAX / 2)
        );
    }

    #[test]
    fn normalization_keeps_legacy_bounds_and_explicit_jitter() {
        assert!(!default_restart_policy().jitter_enabled);
        for jitter_enabled in [false, true] {
            let policy = RestartPolicy {
                enabled: false,
                max_restart_attempts: u32::MAX,
                base_delay: Duration::ZERO,
                max_delay: Duration::ZERO,
                jitter_enabled,
            }
            .normalized();
            assert!(!policy.enabled);
            assert_eq!(policy.max_restart_attempts, 20);
            assert_eq!(policy.base_delay, Duration::from_secs(1));
            assert_eq!(policy.max_delay, Duration::from_secs(1));
            assert_eq!(policy.jitter_enabled, jitter_enabled);
        }
    }

    #[test]
    fn normalized_preserves_jitter_flag() {
        let policy = RestartPolicy {
            enabled: true,
            max_restart_attempts: 5,
            base_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(30),
            jitter_enabled: false,
        }
        .normalized();

        assert!(!policy.jitter_enabled());
    }

    #[test]
    fn jittered_delay_disabled_returns_base() {
        let policy = default_restart_policy();
        assert!(!policy.jitter_enabled());

        let delay = jittered_delay_for_attempt(policy, 1, 0.5);
        assert_eq!(delay, delay_for_attempt(policy, 1));
    }

    #[test]
    fn jittered_delay_stays_within_bounds() {
        let policy =
            RestartPolicy::with_enabled(true, 3, Duration::from_secs(2), Duration::from_secs(30))
                .with_jitter(true);

        let base = delay_for_attempt(policy, 2);

        for rng in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let delay = jittered_delay_for_attempt(policy, 2, rng);
            let lower = base.as_millis() as f64 * (1.0 - DEFAULT_JITTER_FACTOR);
            let upper = base.as_millis() as f64 * (1.0 + DEFAULT_JITTER_FACTOR);
            let millis = delay.as_millis() as f64;
            assert!(millis >= lower.floor() && millis <= upper.ceil());
            assert!(delay <= policy.max_delay);
        }
    }
}
