use super::{
    patterns::{
        is_sensitive_arg, redact_embedded_sensitive_values, redact_json_like_sensitive_values,
        redact_sensitive_assignment, sensitive_value_extends_to_next,
    },
    REDACTED_VALUE,
};

pub fn redact_sensitive_text(value: &str) -> String {
    let value = redact_json_like_sensitive_values(value).unwrap_or_else(|| value.to_string());
    let mut redacted = Vec::new();
    let mut redact_next = false;

    for token in value.split_whitespace() {
        if redact_next {
            redacted.push(REDACTED_VALUE.to_string());
            redact_next = sensitive_value_extends_to_next(token);
            continue;
        }

        if token.contains(REDACTED_VALUE) {
            redacted.push(token.to_string());
            continue;
        }

        if let Some(redacted_token) = redact_embedded_sensitive_values(token) {
            redacted.push(redacted_token);
            continue;
        }

        if let Some((redacted_token, redacts_next)) = redact_sensitive_assignment(token) {
            redacted.push(redacted_token);
            redact_next = redacts_next;
            continue;
        }

        if is_sensitive_arg(token) {
            redacted.push(token.to_string());
            redact_next = true;
        } else {
            redacted.push(token.to_string());
        }
    }

    redacted.join(" ")
}
