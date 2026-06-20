mod args;
mod patterns;
mod text;

pub use args::redact_sensitive_args;
pub use text::redact_sensitive_text;

pub const REDACTED_VALUE: &str = "<redacted>";

#[cfg(test)]
mod tests;
