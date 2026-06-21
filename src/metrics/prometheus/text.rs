use std::fmt::Display;

pub(super) fn push_gauge(
    output: &mut String,
    name: &'static str,
    help: &'static str,
    labels: &[(&'static str, String)],
    value: impl Display,
) {
    push_header(output, name, help);
    push_sample(output, name, labels, value);
}

pub(super) fn push_header(output: &mut String, name: &'static str, help: &'static str) {
    output.push_str("# HELP ");
    output.push_str(name);
    output.push(' ');
    output.push_str(&escape_help_text(help));
    output.push('\n');
    output.push_str("# TYPE ");
    output.push_str(name);
    output.push_str(" gauge\n");
}

pub(super) fn push_sample(
    output: &mut String,
    name: &'static str,
    labels: &[(&'static str, String)],
    value: impl Display,
) {
    output.push_str(name);
    if !labels.is_empty() {
        output.push('{');
        for (index, (label, value)) in labels.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push_str(label);
            output.push_str("=\"");
            output.push_str(&escape_label_value(value));
            output.push('"');
        }
        output.push('}');
    }
    output.push(' ');
    output.push_str(&value.to_string());
    output.push('\n');
}

fn escape_help_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn escape_label_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
#[path = "../../../tests/unit/metrics/prometheus/text/tests.rs"]
mod tests;
