//! Configuration parameters and exponential growth delay calculations.

use std::{env, fmt, time::Duration};

use serde::{Deserialize, Serialize};

/// Configuration parameters for exponential backoff behavior
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BackoffConfig {
    /// Base delay in milliseconds (default: 1000ms)
    pub base_delay_ms: u64,

    /// Maximum delay cap in milliseconds (default: 30000ms)
    pub max_delay_ms: u64,

    /// Growth multiplier per attempt (default: 2.0)
    pub multiplier: f64,

    /// Whether to enable jitter (default: false)
    pub enable_jitter: bool,

    /// Jitter factor as fraction [0.0, 1.0] (default: 0.1 when enabled)
    pub jitter_factor: f64,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 2.0,
            enable_jitter: false,
            jitter_factor: 0.1,
        }
    }
}

impl BackoffConfig {
    /// Load configuration from environment variables with sensible defaults
    pub fn from_env() -> Self {
        let base_delay_ms = env::var("NEONEXUS_RETRY_BASE_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        let max_delay_ms = env::var("NEONEXUS_RETRY_MAX_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30000);

        let multiplier = env::var("NEONEXUS_RETRY_MULTIPLIER")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2.0);

        let enable_jitter = env::var("NEONEXUS_RETRY_JITTER_ENABLED")
            .ok()
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1" || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false);

        let jitter_factor = if enable_jitter {
            env::var("NEONEXUS_RETRY_JITTER_FACTOR")
                .ok()
                .and_then(|v| v.parse().ok())
                .filter(|f| *f >= 0.0 && *f <= 1.0)
                .unwrap_or(0.1)
        } else {
            0.0
        };

        Self {
            base_delay_ms,
            max_delay_ms,
            multiplier,
            enable_jitter,
            jitter_factor,
        }
    }

    /// Validate configuration constraints
    pub fn validate(&self) -> Result<(), BackoffConfigError> {
        if self.base_delay_ms == 0 {
            return Err(BackoffConfigError::ZeroBaseDelay);
        }

        if self.max_delay_ms < self.base_delay_ms {
            return Err(BackoffConfigError::MaxLessThanBase {
                base: self.base_delay_ms,
                max: self.max_delay_ms,
            });
        }

        if self.multiplier < 1.0 {
            return Err(BackoffConfigError::MultiplierTooLow(self.multiplier));
        }

        if self.multiplier > 10.0 {
            return Err(BackoffConfigError::MultiplierTooHigh(self.multiplier));
        }

        if !(0.0..=1.0).contains(&self.jitter_factor) {
            return Err(BackoffConfigError::InvalidJitterFactor(self.jitter_factor));
        }

        Ok(())
    }
}

/// Configuration validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum BackoffConfigError {
    ZeroBaseDelay,
    MaxLessThanBase { base: u64, max: u64 },
    MultiplierTooLow(f64),
    MultiplierTooHigh(f64),
    InvalidJitterFactor(f64),
}

impl fmt::Display for BackoffConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroBaseDelay => write!(f, "base_delay must be greater than zero"),
            Self::MaxLessThanBase { base, max } => {
                write!(f, "max_delay ({}) must be >= base_delay ({})", max, base)
            }
            Self::MultiplierTooLow(m) => {
                write!(f, "multiplier ({}) must be >= 1.0", m)
            }
            Self::MultiplierTooHigh(m) => {
                write!(f, "multiplier ({}) must be <= 10.0", m)
            }
            Self::InvalidJitterFactor(factor) => {
                write!(f, "jitter_factor ({}) must be in range [0.0, 1.0]", factor)
            }
        }
    }
}

impl std::error::Error for BackoffConfigError {}

/// Exponential backoff builder with deterministic progression
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExponentialBackoff {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: Option<f64>,
    pub retries: usize,
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::from(BackoffConfig::from_env())
    }
}

impl From<BackoffConfig> for ExponentialBackoff {
    fn from(config: BackoffConfig) -> Self {
        Self {
            base_delay: Duration::from_millis(config.base_delay_ms),
            max_delay: Duration::from_millis(config.max_delay_ms),
            multiplier: config.multiplier,
            jitter: if config.enable_jitter {
                Some(config.jitter_factor)
            } else {
                None
            },
            retries: usize::MAX,
        }
    }
}

impl ExponentialBackoff {
    /// Create new backoff with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure maximum retry attempts
    #[must_use]
    pub fn with_retries(mut self, retries: usize) -> Self {
        self.retries = retries;
        self
    }

    /// Enable jitter with specified factor [0.0, 1.0]
    #[must_use]
    pub fn with_jitter(mut self, factor: f64) -> Self {
        self.jitter = Some(factor);
        self
    }

    /// Calculate delay for specific attempt number (1-indexed)
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        let sample = if self.jitter.is_some() {
            self.random_factor()
        } else {
            0.5
        };
        self.delay_for_attempt_with_random_factor(attempt, sample)
    }

    /// Pure production calculation with an injected uniform sample in [0, 1].
    /// Unlike watchdog policies, attempt zero is zero; uncapped calculations are
    /// truncated to milliseconds. Zero delays and caps below the base are honored.
    /// Finite growth multipliers >= 1 retain their value; invalid values use 2.
    /// Finite jitter fractions are clamped to [0, 1], non-finite fractions disable
    /// jitter. Samples are clamped to [0, 1], with NaN selecting the midpoint.
    /// The symmetric configured jitter distribution is applied before a final cap.
    pub fn delay_for_attempt_with_random_factor(&self, attempt: usize, sample: f64) -> Duration {
        if attempt == 0 || self.base_delay.as_millis() == 0 || self.max_delay.is_zero() {
            return Duration::ZERO;
        }
        let multiplier = if self.multiplier.is_finite() && self.multiplier >= 1.0 {
            self.multiplier
        } else {
            2.0
        };
        let raw_ms = self.base_delay.as_millis() as f64 * multiplier.powf((attempt - 1) as f64);
        let capped = duration_from_millis_capped(raw_ms, self.max_delay);
        let Some(fraction) = self.jitter.filter(|fraction| fraction.is_finite()) else {
            return capped;
        };
        let fraction = fraction.clamp(0.0, 1.0);
        let sample = if sample.is_nan() {
            0.5
        } else {
            sample.clamp(0.0, 1.0)
        };
        let jitter_multiplier = 1.0 - fraction + 2.0 * fraction * sample;
        if jitter_multiplier == 1.0 {
            return capped;
        }
        duration_from_millis_capped(
            capped.as_millis() as f64 * jitter_multiplier,
            self.max_delay,
        )
    }

    /// Check if another retry is allowed
    pub fn should_retry(&self, attempt: usize) -> bool {
        attempt > 0 && attempt <= self.retries
    }

    /// Get maximum retry limit
    pub fn max_retries(&self) -> usize {
        self.retries
    }

    /// Async sleep helper using tokio
    pub async fn sleep(&self, attempt: usize) -> Duration {
        let delay = self.delay_for_attempt(attempt);
        tokio::time::sleep(delay).await;
        delay
    }

    /// Generate a random factor for jitter (private helper)
    fn random_factor(&self) -> f64 {
        rand::random::<f64>()
    }

    /// Format delay with unit suffix for logging
    pub fn format_delay(&self, delay: Duration) -> String {
        if delay.as_secs() >= 1 {
            format!("{:.2}s", delay.as_secs_f64())
        } else {
            format!("{}ms", delay.as_millis())
        }
    }

    /// Generate human-readable retry error message
    pub fn retry_exhausted_message(&self, node_id: &str, attempt: usize, cause: &str) -> String {
        format!(
            "Node '{}' failed after {} retry attempts: {}. \
             Last delay was {}. \n\
             Consider: checking node logs, verifying binary availability, \
             or adjusting restart policy",
            node_id,
            attempt,
            cause,
            self.format_delay(self.max_delay)
        )
    }
}

pub(super) fn duration_from_millis_capped(millis: f64, cap: Duration) -> Duration {
    if millis >= cap.as_secs_f64() * 1000.0 {
        return cap;
    }
    let nanos = (millis as u128)
        .saturating_mul(1_000_000)
        .min(cap.as_nanos());
    Duration::new(
        (nanos / 1_000_000_000) as u64,
        (nanos % 1_000_000_000) as u32,
    )
}

impl std::fmt::Display for ExponentialBackoff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ExponentialBackoff {{")?;
        writeln!(f, "  base: {}", self.format_delay(self.base_delay))?;
        writeln!(f, "  max:  {}", self.format_delay(self.max_delay))?;
        writeln!(f, "  multiplier: {:.2}", self.multiplier)?;
        writeln!(f, "  jitter: {:?}", self.jitter)?;
        writeln!(f, "  max_retries: {}", self.retries)?;
        write!(f, "}}")
    }
}
