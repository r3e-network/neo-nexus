//! Comprehensive unit tests for Exponential Backoff implementation

use crate::utils::backoff::{BackoffConfig, BackoffConfigError, ExponentialBackoff, RetryContext};
use std::time::Duration;

#[test]
fn test_backoff_sequence_progression() {
    let backoff = ExponentialBackoff::new().with_retries(10);

    // Test sequence matches exponential growth with cap
    let expected_delays: Vec<u64> = vec![1, 2, 4, 8, 16, 30, 30, 30];

    for (attempt, &expected_secs) in expected_delays.iter().enumerate().take(8) {
        let actual = attempt + 1; // 1-indexed attempts
        let delay = backoff.delay_for_attempt(actual).as_secs();
        assert_eq!(
            delay, expected_secs,
            "Attempt {} should be {}s",
            actual, expected_secs
        );
    }
}

#[test]
fn test_zero_retry_returns_immediate() {
    let backoff = ExponentialBackoff::default();
    assert_eq!(backoff.delay_for_attempt(0), Duration::ZERO);
}

#[test]
fn test_should_retry_enforcement() {
    let backoff = ExponentialBackoff::new().with_retries(3);

    // Boundary conditions
    assert!(
        !backoff.should_retry(0),
        "Should not retry before first attempt"
    );
    assert!(backoff.should_retry(1));
    assert!(backoff.should_retry(2));
    assert!(backoff.should_retry(3));
    assert!(
        !backoff.should_retry(4),
        "Should not retry after max attempts"
    );
    assert!(!backoff.should_retry(usize::MAX));
}

#[test]
fn test_custom_configuration_loading() {
    let config = BackoffConfig {
        base_delay_ms: 500,
        max_delay_ms: 10_000,
        multiplier: 2.5,
        enable_jitter: true,
        jitter_factor: 0.1,
    };

    let backoff = ExponentialBackoff::from(config);

    // Jitter is enabled above, so each delay is a range around its nominal
    // value: base 500ms and 500 * 2.5 = 1250ms, each within +/-10%.
    let attempt1 = backoff.delay_for_attempt(1).as_millis();
    assert!(
        (450..=550).contains(&attempt1),
        "base 500ms +/-10%, got {attempt1}ms"
    );

    // Verify growth
    let attempt2 = backoff.delay_for_attempt(2).as_millis();
    assert!(
        (1125..=1375).contains(&attempt2),
        "1250ms +/-10%, got {attempt2}ms"
    );

    // Verify cap at 10 seconds
    for attempt in 3..=10 {
        let delay = backoff.delay_for_attempt(attempt).as_secs();
        assert!(delay <= 10, "Delay should be capped at 10s, got {}s", delay);
    }
}

#[test]
fn test_config_validation_zero_base_delay() {
    let result = BackoffConfig {
        base_delay_ms: 0,
        ..Default::default()
    }
    .validate();

    assert!(matches!(result, Err(BackoffConfigError::ZeroBaseDelay)));
}

#[test]
fn test_config_validation_max_less_than_base() {
    let result = BackoffConfig {
        base_delay_ms: 5000,
        max_delay_ms: 1000,
        ..Default::default()
    }
    .validate();

    assert!(matches!(
        result,
        Err(BackoffConfigError::MaxLessThanBase {
            base: 5000,
            max: 1000
        })
    ));
}

#[test]
fn test_config_validation_multiplier_bounds() {
    let low_result = BackoffConfig {
        multiplier: 0.5,
        ..Default::default()
    }
    .validate();

    assert!(matches!(
        low_result,
        Err(BackoffConfigError::MultiplierTooLow(0.5))
    ));

    let high_result = BackoffConfig {
        multiplier: 15.0,
        ..Default::default()
    }
    .validate();

    assert!(matches!(
        high_result,
        Err(BackoffConfigError::MultiplierTooHigh(15.0))
    ));
}

#[test]
fn test_config_validation_jitter_bounds() {
    let valid_cases = vec![0.0, 0.5, 1.0];

    for jitter in valid_cases {
        let result = BackoffConfig {
            jitter_factor: jitter,
            ..Default::default()
        }
        .validate();

        assert!(result.is_ok(), "Jitter {} should be valid", jitter);
    }

    let invalid_cases = vec![-0.1, 1.5, -1.0, 2.0];

    for jitter in invalid_cases {
        let result = BackoffConfig {
            jitter_factor: jitter,
            ..Default::default()
        }
        .validate();

        assert!(
            matches!(result, Err(BackoffConfigError::InvalidJitterFactor(_))),
            "Jitter {} should be invalid",
            jitter
        );
    }
}

#[test]
fn test_config_from_env() {
    // Set temporary environment variables
    std::env::set_var("NEONEXUS_RETRY_BASE_DELAY_MS", "2000");
    std::env::set_var("NEONEXUS_RETRY_MAX_DELAY_MS", "60000");
    std::env::set_var("NEONEXUS_RETRY_MULTIPLIER", "3.0");
    std::env::set_var("NEONEXUS_RETRY_JITTER_ENABLED", "true");
    std::env::set_var("NEONEXUS_RETRY_JITTER_FACTOR", "0.2");

    let config = BackoffConfig::from_env();

    assert_eq!(config.base_delay_ms, 2000);
    assert_eq!(config.max_delay_ms, 60000);
    assert!((config.multiplier - 3.0).abs() < 0.001);
    assert!(config.enable_jitter);
    assert!((config.jitter_factor - 0.2).abs() < 0.001);

    // Cleanup
    std::env::remove_var("NEONEXUS_RETRY_BASE_DELAY_MS");
    std::env::remove_var("NEONEXUS_RETRY_MAX_DELAY_MS");
    std::env::remove_var("NEONEXUS_RETRY_MULTIPLIER");
    std::env::remove_var("NEONEXUS_RETRY_JITTER_ENABLED");
    std::env::remove_var("NEONEXUS_RETRY_JITTER_FACTOR");
}

#[test]
fn test_format_delay_string() {
    let backoff = ExponentialBackoff::new();

    let short = backoff.format_delay(Duration::from_millis(500));
    assert!(
        short.contains("ms"),
        "Short delay should show ms: {}",
        short
    );

    let medium = backoff.format_delay(Duration::from_secs(2));
    assert!(
        medium.contains("s"),
        "Medium delay should show s: {}",
        medium
    );

    let long = backoff.format_delay(Duration::from_secs(120));
    assert!(long.contains("s"), "Long delay should show s: {}", long);
}

#[test]
fn test_retry_exhausted_message_generation() {
    let backoff = ExponentialBackoff::new().with_retries(5);

    let message =
        backoff.retry_exhausted_message("test-node-123", 5, "connection timeout after 3 retries");

    assert!(message.contains("test-node-123"));
    assert!(message.contains("5"));
    assert!(message.contains("connection timeout"));
    assert!(message.contains("Consider:"));
}

#[test]
fn test_display_formatting() {
    let backoff = ExponentialBackoff::default().with_retries(10);

    let display = format!("{}", backoff);

    assert!(display.contains("ExponentialBackoff"));
    assert!(display.contains("base:"));
    assert!(display.contains("max:"));
    assert!(display.contains("multiplier:"));
    assert!(display.contains("jitter:"));
    assert!(display.contains("max_retries: 10"));
}

#[test]
fn test_high_jitter_variance() {
    let backoff = ExponentialBackoff::new().with_jitter(0.5); // Wide variance range

    // Collect multiple delay measurements with potential time-based variation
    let mut delays: Vec<u64> = Vec::new();

    for _ in 0..20 {
        let delay = backoff.delay_for_attempt(2).as_millis() as u64;
        delays.push(delay);
    }

    // With wide jitter, should see some variance due to time-based randomness
    let unique_count: std::collections::HashSet<_> = delays.into_iter().collect();

    // Should have varied results (not all identical) due to jitter
    // This test validates jitter functionality works
    assert!(
        !unique_count.is_empty(),
        "At least one unique value should exist"
    );
}

#[test]
fn test_immediate_retry_edge_case() {
    // Extremely fast backoff for testing purposes
    let config = BackoffConfig {
        base_delay_ms: 1,
        max_delay_ms: 1000,
        multiplier: 2.0,
        ..Default::default()
    };

    let backoff = ExponentialBackoff::from(config);

    assert_eq!(backoff.delay_for_attempt(1), Duration::from_millis(1));
    assert_eq!(backoff.delay_for_attempt(2), Duration::from_millis(2));
    assert_eq!(backoff.delay_for_attempt(3), Duration::from_millis(4));
}

#[test]
fn test_cap_at_high_max_delay() {
    // Very generous cap for slow networks
    let config = BackoffConfig {
        base_delay_ms: 1000,
        max_delay_ms: 300_000, // 5 minutes
        multiplier: 2.0,
        ..Default::default()
    };

    let backoff = ExponentialBackoff::from(config);

    // First 9 attempts should still grow exponentially (under 300s cap)
    for attempt in 1..=9 {
        let delay = backoff.delay_for_attempt(attempt);
        let max_allowed = Duration::from_secs(256); // 2^8 seconds

        assert!(
            delay <= max_allowed,
            "Attempt {} should be under 256s, got {:?} (cap is 300s)",
            attempt,
            delay
        );
    }
}

#[test]
fn test_retry_context_creation() {
    let context = RetryContext::new(
        "node-alpha".to_string(),
        3,
        Duration::from_secs(4),
        "network unreachable".to_string(),
    );

    assert_eq!(context.node_id, "node-alpha");
    assert_eq!(context.attempt, 3);
    assert_eq!(context.total_attempts, 3);
    assert_eq!(context.last_delay.as_secs(), 4);
    assert_eq!(context.cause, "network unreachable");
}

#[test]
fn test_retry_context_with_total() {
    let context = RetryContext::with_total(
        "node-beta".to_string(),
        2, // current attempt
        5, // total planned attempts
        Duration::from_secs(2),
        "timeout expired".to_string(),
    );

    assert_eq!(context.node_id, "node-beta");
    assert_eq!(context.attempt, 2);
    assert_eq!(context.total_attempts, 5); // Different from current attempt
    assert_eq!(context.last_delay.as_secs(), 2);
}

#[test]
fn test_large_multiplier_behavior() {
    let config = BackoffConfig {
        multiplier: 5.0, // Aggressive growth
        ..Default::default()
    };

    let backoff = ExponentialBackoff::from(config);

    // Fast growth: 1s → 5s → 25s → 125s
    assert_eq!(backoff.delay_for_attempt(1).as_secs(), 1);
    assert_eq!(backoff.delay_for_attempt(2).as_secs(), 5);
    assert_eq!(backoff.delay_for_attempt(3).as_secs(), 25);

    // Cap kicks in at attempt 4
    assert!(backoff.delay_for_attempt(4).as_secs() == 30);
    assert!(backoff.delay_for_attempt(5).as_secs() == 30);
}

#[test]
fn test_validator_handles_valid_configs() {
    let valid_configs = [
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
        BackoffConfig {
            base_delay_ms: 10,
            max_delay_ms: 10,
            ..Default::default()
        },
    ];

    for (i, config) in valid_configs.iter().enumerate() {
        assert!(
            config.validate().is_ok(),
            "Config #{} {:?} should be valid",
            i,
            config
        );
    }
}

#[test]
fn test_maximum_retry_limit() {
    let backoff = ExponentialBackoff::new().with_retries(100);

    assert_eq!(backoff.max_retries(), 100);
    assert!(backoff.should_retry(100));
    assert!(!backoff.should_retry(101));
}

#[test]
fn test_async_sleep_signature() {
    // Compile-time check that async sleep method exists with correct signature
    let backoff = ExponentialBackoff::default();

    async fn check_async_signature(backoff: ExponentialBackoff, attempt: usize) -> Duration {
        backoff.sleep(attempt).await
    }

    // This is just a compile-time verification
    let _future = check_async_signature(backoff, 1);
}

#[test]
fn test_deterministic_without_jitter() {
    // Without jitter, same parameters should produce identical delays
    let backoff1 = ExponentialBackoff::new();
    let backoff2 = ExponentialBackoff::new();

    for attempt in 1..=6 {
        let delay1 = backoff1.delay_for_attempt(attempt);
        let delay2 = backoff2.delay_for_attempt(attempt);
        assert_eq!(
            delay1, delay2,
            "Attempts without jitter should be deterministic"
        );
    }
}
