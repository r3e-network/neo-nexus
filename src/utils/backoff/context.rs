//! Retry result context tracking attempts and failure causes.

use std::time::Duration;

/// Retry result type with context information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryContext {
    pub node_id: String,
    pub attempt: usize,
    pub total_attempts: usize,
    pub last_delay: Duration,
    pub cause: String,
}

impl RetryContext {
    pub fn new(node_id: String, attempt: usize, last_delay: Duration, cause: String) -> Self {
        Self {
            node_id,
            attempt,
            total_attempts: attempt,
            last_delay,
            cause,
        }
    }

    pub fn with_total(
        node_id: String,
        attempt: usize,
        total: usize,
        last_delay: Duration,
        cause: String,
    ) -> Self {
        Self {
            node_id,
            attempt,
            total_attempts: total,
            last_delay,
            cause,
        }
    }
}
