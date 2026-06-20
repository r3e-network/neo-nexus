use super::REDACTED_VALUE;

mod json_like;

pub(super) use self::json_like::redact_json_like_sensitive_values;

const SENSITIVE_KEY_FRAGMENTS: &[&str] = &[
    "password",
    "passphrase",
    "secret",
    "token",
    "api-key",
    "apikey",
    "access-key",
    "accesskey",
    "auth",
    "authorization",
    "bearer",
    "routing-key",
    "chat-id",
    "mnemonic",
    "seed",
    "wif",
    "private-key",
    "privatekey",
    "wallet-key",
    "webhook",
    "credential",
    "jwt",
    "cookie",
];

pub(super) fn redact_sensitive_assignment(value: &str) -> Option<(String, bool)> {
    let (key, separator, assigned_value) = sensitive_assignment(value)?;
    Some((
        format!("{key}{separator}{REDACTED_VALUE}"),
        assigned_value.is_empty(),
    ))
}

pub(super) fn redact_embedded_sensitive_values(value: &str) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0;
    let mut scan_from = 0;
    let mut changed = false;

    while let Some(segment_start) = next_parameter_delimiter(value, scan_from) {
        let key_start = segment_start + 1;
        let Some(separator) = next_assignment_separator(value, key_start) else {
            scan_from = key_start;
            continue;
        };

        if has_delimiter_before_assignment(value, key_start, separator) {
            scan_from = key_start;
            continue;
        }

        let key = &value[key_start..separator];
        let value_start = separator + 1;
        let value_end = next_parameter_value_end(value, value_start);
        scan_from = value_end;

        if !is_sensitive_arg(key) {
            continue;
        }

        output.push_str(&value[cursor..value_start]);
        output.push_str(REDACTED_VALUE);
        cursor = value_end;
        changed = true;
    }

    changed.then(|| {
        output.push_str(&value[cursor..]);
        output
    })
}

pub(super) fn sensitive_value_extends_to_next(value: &str) -> bool {
    matches!(
        normalized_sensitive_key(value).as_str(),
        "bearer" | "basic" | "token"
    )
}

pub(super) fn is_sensitive_arg(value: &str) -> bool {
    let value = normalized_sensitive_key(value);
    SENSITIVE_KEY_FRAGMENTS
        .iter()
        .any(|fragment| value.contains(fragment))
}

fn sensitive_assignment(value: &str) -> Option<(&str, char, &str)> {
    ['=', ':'].iter().find_map(|separator| {
        let (key, assigned_value) = value.split_once(*separator)?;
        (is_sensitive_arg(key) && !assigned_value.contains(REDACTED_VALUE)).then_some((
            key,
            *separator,
            assigned_value,
        ))
    })
}

fn normalized_sensitive_key(value: &str) -> String {
    value
        .trim_start_matches('-')
        .trim_matches(|character: char| {
            matches!(character, '"' | '\'' | ':' | ',' | '{' | '}' | '[' | ']')
        })
        .to_ascii_lowercase()
        .replace('_', "-")
}

fn next_parameter_delimiter(value: &str, from: usize) -> Option<usize> {
    value[from..]
        .find(['?', '&', ';'])
        .map(|index| from + index)
}

fn next_assignment_separator(value: &str, from: usize) -> Option<usize> {
    value[from..].find('=').map(|index| from + index)
}

fn has_delimiter_before_assignment(value: &str, from: usize, assignment: usize) -> bool {
    value[from..assignment].contains(['?', '&', ';', '#'])
}

fn next_parameter_value_end(value: &str, from: usize) -> usize {
    value[from..]
        .find(['&', ';', '#'])
        .map_or(value.len(), |index| from + index)
}
