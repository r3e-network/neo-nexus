use crate::redaction::redact_sensitive_text;

pub(super) fn wallet_provisioning_sensitive_string(value: &str) -> bool {
    !value.trim().is_empty() && redact_sensitive_text(value) != value
}
