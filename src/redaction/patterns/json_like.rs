use crate::redaction::REDACTED_VALUE;

use super::is_sensitive_arg;

pub(in crate::redaction) fn redact_json_like_sensitive_values(value: &str) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0;
    let mut scan_from = 0;
    let mut changed = false;

    while let Some(key_quote_start) = next_quote(value, scan_from) {
        let key_start = key_quote_start + 1;
        let Some(key_end) = find_unescaped_quote(value, key_start) else {
            break;
        };
        let Some(colon) = next_non_whitespace(value, key_end + 1) else {
            break;
        };
        if !value[colon..].starts_with(':') {
            scan_from = key_end + 1;
            continue;
        }

        let key = &value[key_start..key_end];
        let Some(value_start) = next_non_whitespace(value, colon + 1) else {
            break;
        };
        let value_end = json_like_value_end(value, value_start);
        scan_from = value_end;

        if !is_sensitive_arg(key) {
            continue;
        }

        output.push_str(&value[cursor..colon + 1]);
        output.push_str(REDACTED_VALUE);
        cursor = value_end;
        changed = true;
    }

    changed.then(|| {
        output.push_str(&value[cursor..]);
        output
    })
}

fn next_quote(value: &str, from: usize) -> Option<usize> {
    value[from..].find('"').map(|index| from + index)
}

fn next_non_whitespace(value: &str, from: usize) -> Option<usize> {
    value[from..]
        .char_indices()
        .find_map(|(index, character)| (!character.is_whitespace()).then_some(from + index))
}

fn find_unescaped_quote(value: &str, from: usize) -> Option<usize> {
    let mut escaped = false;
    for (index, character) in value[from..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '"' => return Some(from + index),
            _ => {}
        }
    }
    None
}

fn json_like_value_end(value: &str, from: usize) -> usize {
    if value[from..].starts_with('"') {
        return find_unescaped_quote(value, from + 1).map_or(value.len(), |index| index + 1);
    }

    value[from..]
        .find([',', '}', ']'])
        .map_or(value.len(), |index| from + index)
}
