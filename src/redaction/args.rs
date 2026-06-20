use super::{
    patterns::{
        is_sensitive_arg, redact_embedded_sensitive_values, redact_sensitive_assignment,
        sensitive_value_extends_to_next,
    },
    REDACTED_VALUE,
};

pub fn redact_sensitive_args(args: &[String]) -> Vec<String> {
    let mut redacted = Vec::with_capacity(args.len());
    let mut redact_next = false;

    for arg in args {
        if redact_next {
            redacted.push(REDACTED_VALUE.to_string());
            redact_next = sensitive_value_extends_to_next(arg);
            continue;
        }

        if let Some((redacted_arg, redacts_next)) = redact_sensitive_assignment(arg) {
            redacted.push(redacted_arg);
            redact_next = redacts_next;
            continue;
        }

        if let Some(redacted_arg) = redact_embedded_sensitive_values(arg) {
            redacted.push(redacted_arg);
            continue;
        }

        if is_sensitive_arg(arg) {
            redacted.push(arg.clone());
            redact_next = true;
        } else {
            redacted.push(arg.clone());
        }
    }

    redacted
}
