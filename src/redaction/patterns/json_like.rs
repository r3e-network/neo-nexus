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

        if is_sensitive_arg(key) {
            output.push_str(&value[cursor..colon + 1]);
            output.push_str(REDACTED_VALUE);
            cursor = value_end;
            changed = true;
            scan_from = value_end;
        } else if value[value_start..].starts_with(['{', '[']) {
            // Descend into a non-sensitive container so nested sensitive keys
            // are still examined instead of being skipped as an opaque value.
            scan_from = value_start + 1;
        } else {
            scan_from = value_end;
        }
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
    let rest = &value[from..];
    if rest.starts_with('"') {
        return find_unescaped_quote(value, from + 1).map_or(value.len(), |index| index + 1);
    }
    if rest.starts_with(['{', '[']) {
        return balanced_container_end(value, from);
    }

    rest.find([',', '}', ']'])
        .map_or(value.len(), |index| from + index)
}

/// Returns the byte offset just past the closing brace/bracket that matches the
/// container opening at `from`, ignoring delimiters inside string literals so a
/// nested object or array is treated as a single value. Falls back to the end
/// of the string when the container is not closed.
fn balanced_container_end(value: &str, from: usize) -> usize {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, character) in value[from..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                depth -= 1;
                if depth == 0 {
                    return from + offset + character.len_utf8();
                }
            }
            _ => {}
        }
    }
    value.len()
}
