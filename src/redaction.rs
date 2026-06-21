mod args;
mod patterns;
mod text;

pub use args::redact_sensitive_args;
pub use text::redact_sensitive_text;

pub const REDACTED_VALUE: &str = "<redacted>";

#[cfg(test)]
#[path = "../tests/unit/redaction/tests.rs"]
mod tests;
