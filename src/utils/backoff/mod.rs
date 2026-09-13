//! Exponential backoff calculation and retry context.

mod context;
mod policy;

pub use context::RetryContext;
pub use policy::{BackoffConfig, BackoffConfigError, ExponentialBackoff};

#[cfg(test)]
mod tests;
