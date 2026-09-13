use super::*;
use std::{env, time::Duration};

#[test]
fn test_default_config() {
    let config = BackoffConfig::default();
    assert_eq!(config.base_delay_ms, 1000);
    assert_eq!(config.max_delay_ms, 30000);
    assert!((config.multiplier - 2.0).abs() < 0.001);
    assert!(!config.enable_jitter);
}

#[test]
fn test_backoff_progression_no_jitter() {
    let backoff = ExponentialBackoff::new().with_retries(10);

    let delays: Vec<u64> = (1..=6)
        .map(|a| backoff.delay_for_attempt(a).as_secs())
        .collect();

    // Expected: 1, 2, 4, 8, 16, 30 (capped at max)
    assert_eq!(delays, vec![1, 2, 4, 8, 16, 30]);
}

#[test]
fn test_zero_attempt_returns_zero_duration() {
    let backoff = ExponentialBackoff::new();
    assert_eq!(backoff.delay_for_attempt(0), Duration::ZERO);
}

#[test]
fn test_should_retry_boundaries() {
    let backoff = ExponentialBackoff::new().with_retries(3);

    assert!(!backoff.should_retry(0));
    assert!(backoff.should_retry(1));
    assert!(backoff.should_retry(2));
    assert!(backoff.should_retry(3));
    assert!(!backoff.should_retry(4));
}

#[test]
fn test_max_retries_enforcement() {
    let backoff = ExponentialBackoff::new().with_retries(5);
    assert_eq!(backoff.max_retries(), 5);
}

#[test]
fn test_custom_config() {
    let config = BackoffConfig {
        base_delay_ms: 500,
        max_delay_ms: 10000,
        multiplier: 2.5,
        enable_jitter: false,
        jitter_factor: 0.0,
    };

    let backoff = ExponentialBackoff::from(config);

    let delays: Vec<u64> = (1..=4)
        .map(|a| backoff.delay_for_attempt(a).as_secs())
        .collect();

    // Expected: 0.5, 1.25, 3.125, 7.8125 (all rounded down)
    assert_eq!(delays, vec![0, 1, 3, 7]);
}

#[test]
fn test_validate_valid_configs() {
    let valid_configs = vec![
        BackoffConfig {
            base_delay_ms: 100,
            max_delay_ms: 1000,
            ..Default::default()
        },
        BackoffConfig {
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            ..Default::default()
        },
        BackoffConfig {
            base_delay_ms: 5000,
            max_delay_ms: 5000,
            ..Default::default()
        },
    ];

    for config in valid_configs {
        assert!(
            config.validate().is_ok(),
            "Config {:?} should be valid",
            config
        );
    }
}

#[test]
fn test_validate_invalid_configs() {
    assert!(matches!(
        BackoffConfig {
            base_delay_ms: 0,
            ..Default::default()
        }
        .validate(),
        Err(BackoffConfigError::ZeroBaseDelay)
    ));

    assert!(matches!(
        BackoffConfig {
            base_delay_ms: 1000,
            max_delay_ms: 500,
            ..Default::default()
        }
        .validate(),
        Err(BackoffConfigError::MaxLessThanBase {
            base: 1000,
            max: 500
        })
    ));

    assert!(matches!(
        BackoffConfig {
            multiplier: 0.5,
            ..Default::default()
        }
        .validate(),
        Err(BackoffConfigError::MultiplierTooLow(0.5))
    ));

    assert!(matches!(
        BackoffConfig {
            multiplier: 15.0,
            ..Default::default()
        }
        .validate(),
        Err(BackoffConfigError::MultiplierTooHigh(15.0))
    ));

    assert!(matches!(
        BackoffConfig {
            jitter_factor: 1.5,
            ..Default::default()
        }
        .validate(),
        Err(BackoffConfigError::InvalidJitterFactor(1.5))
    ));
}

#[test]
fn test_format_delay() {
    let backoff = ExponentialBackoff::new();

    assert!(backoff
        .format_delay(Duration::from_millis(500))
        .contains("ms"));
    assert!(backoff.format_delay(Duration::from_secs(1)).contains("s"));
    assert!(backoff.format_delay(Duration::from_secs(60)).contains("s"));
}

#[test]
fn test_retry_exhausted_message() {
    let backoff = ExponentialBackoff::new().with_retries(3);
    let msg = backoff.retry_exhausted_message("node-abc", 3, "connection timeout");

    assert!(msg.contains("node-abc"));
    assert!(msg.contains("3"));
    assert!(msg.contains("connection timeout"));
    assert!(msg.contains("Consider:"));
}

#[test]
fn test_display_impl() {
    let backoff = ExponentialBackoff::new().with_retries(5);
    let display = format!("{}", backoff);

    assert!(display.contains("ExponentialBackoff"));
    assert!(display.contains("base:"));
    assert!(display.contains("max:"));
}

#[test]
fn test_jitter_variations() {
    let backoff = ExponentialBackoff::from(BackoffConfig::default()).with_jitter(0.5);
    let delays: Vec<_> = [0.0, 0.25, 0.5, 0.75, 1.0]
        .into_iter()
        .map(|sample| {
            backoff
                .delay_for_attempt_with_random_factor(2, sample)
                .as_millis()
        })
        .collect();
    assert_eq!(delays, vec![1000, 1500, 2000, 2500, 3000]);
}

#[test]
fn production_backoff_jitter_caps_after_randomization() {
    let backoff = ExponentialBackoff::from(BackoffConfig {
        base_delay_ms: 500,
        max_delay_ms: 3000,
        multiplier: 2.5,
        enable_jitter: true,
        jitter_factor: 0.5,
    });
    assert_eq!(
        backoff.delay_for_attempt_with_random_factor(2, 1.0),
        Duration::from_millis(1875)
    );
    assert_eq!(
        backoff.delay_for_attempt_with_random_factor(usize::MAX, 0.0),
        Duration::from_millis(1500)
    );
    for attempt in [0, 1, 2, 3, 32, 100, usize::MAX] {
        for sample in [0.0, 0.5, 1.0] {
            assert!(
                backoff.delay_for_attempt_with_random_factor(attempt, sample) <= backoff.max_delay
            );
        }
        assert!(backoff.delay_for_attempt(attempt) <= backoff.max_delay);
    }
    assert_eq!(backoff.delay_for_attempt(0), Duration::ZERO);
}

#[test]
fn production_backoff_saturates_large_attempts_without_shift_plateau() {
    let mut backoff = ExponentialBackoff::from(BackoffConfig::default());
    backoff.base_delay = Duration::from_millis(1);
    backoff.max_delay = Duration::MAX;
    assert_eq!(
        backoff.delay_for_attempt_with_random_factor(34, 0.5),
        Duration::from_millis(1_u64 << 33)
    );
    assert_eq!(backoff.delay_for_attempt(usize::MAX), Duration::MAX);
    backoff.multiplier = 1.0;
    assert_eq!(backoff.delay_for_attempt(usize::MAX), backoff.base_delay);
}

#[test]
fn production_backoff_zero_reversed_bounds_and_duration_extremes() {
    for (base_delay, max_delay) in [
        (Duration::ZERO, Duration::MAX),
        (Duration::MAX, Duration::ZERO),
        (Duration::from_secs(10), Duration::from_secs(1)),
        (Duration::from_nanos(1), Duration::from_nanos(10)),
        (Duration::MAX, Duration::MAX),
    ] {
        for jitter in [None, Some(0.5), Some(1.0)] {
            let backoff = ExponentialBackoff {
                base_delay,
                max_delay,
                multiplier: 2.5,
                jitter,
                retries: usize::MAX,
            };
            for attempt in [0, 1, 2, 32, usize::MAX] {
                for sample in [0.0, 0.5, 1.0] {
                    let delay = backoff.delay_for_attempt_with_random_factor(attempt, sample);
                    assert!(delay <= max_delay);
                    if attempt == 0 || base_delay.as_millis() == 0 || max_delay.is_zero() {
                        assert_eq!(delay, Duration::ZERO);
                    }
                }
            }
        }
    }
    let extreme = ExponentialBackoff {
        base_delay: Duration::MAX,
        max_delay: Duration::MAX,
        ..ExponentialBackoff::from(BackoffConfig::default())
    };
    assert_eq!(extreme.delay_for_attempt(1), Duration::MAX);
    assert_eq!(
        extreme
            .with_jitter(0.5)
            .delay_for_attempt_with_random_factor(1, 1.0),
        Duration::MAX
    );
    let tiny_cap = ExponentialBackoff {
        max_delay: Duration::from_micros(500),
        ..extreme.with_jitter(1.0)
    };
    assert_eq!(
        tiny_cap.delay_for_attempt_with_random_factor(1, 0.0),
        Duration::ZERO
    );
}

#[test]
fn production_backoff_normalizes_invalid_floating_inputs() {
    let default = ExponentialBackoff::from(BackoffConfig::default());
    for multiplier in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.5] {
        let backoff = ExponentialBackoff {
            multiplier,
            ..default
        };
        assert_eq!(backoff.delay_for_attempt(2), Duration::from_secs(2));
    }
    for fraction in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        assert_eq!(
            default
                .with_jitter(fraction)
                .delay_for_attempt_with_random_factor(1, 0.0),
            Duration::from_secs(1)
        );
    }
    let backoff = default.with_jitter(2.0);
    assert_eq!(
        backoff.delay_for_attempt_with_random_factor(1, -1.0),
        Duration::ZERO
    );
    assert_eq!(
        backoff.delay_for_attempt_with_random_factor(1, 2.0),
        Duration::from_secs(2)
    );
    assert_eq!(
        backoff.delay_for_attempt_with_random_factor(1, f64::NAN),
        Duration::from_secs(1)
    );
}

#[test]
fn test_config_from_env_defaults() {
    // Temporarily unset any existing env vars
    env::remove_var("NEONEXUS_RETRY_BASE_DELAY_MS");
    env::remove_var("NEONEXUS_RETRY_MAX_DELAY_MS");
    env::remove_var("NEONEXUS_RETRY_MULTIPLIER");
    env::remove_var("NEONEXUS_RETRY_JITTER_ENABLED");
    env::remove_var("NEONEXUS_RETRY_JITTER_FACTOR");

    let config = BackoffConfig::from_env();
    assert_eq!(config.base_delay_ms, 1000);
    assert_eq!(config.max_delay_ms, 30000);
}

#[test]
fn test_edge_case_immediate_retry() {
    // Test with very small base delay
    let config = BackoffConfig {
        base_delay_ms: 1,
        max_delay_ms: 1000,
        multiplier: 2.0,
        ..Default::default()
    };

    let backoff = ExponentialBackoff::from(config);
    assert_eq!(backoff.delay_for_attempt(1), Duration::from_millis(1));
    assert_eq!(backoff.delay_for_attempt(2), Duration::from_millis(2));
}

#[test]
fn test_edge_case_high_max_delay() {
    // Test with very large max delay
    let config = BackoffConfig {
        base_delay_ms: 1000,
        max_delay_ms: 300000, // 5 minutes
        multiplier: 2.0,
        ..Default::default()
    };

    let backoff = ExponentialBackoff::from(config);

    // First 7 attempts should still grow exponentially
    for attempt in 1..=7 {
        let delay = backoff.delay_for_attempt(attempt);
        assert!(delay <= Duration::from_secs(128)); // 2^7 = 128 seconds
    }
}
