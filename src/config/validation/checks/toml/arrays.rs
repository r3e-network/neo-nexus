use super::super::{
    super::model::ConfigValidationReport,
    paths::{dotted_path, toml_path},
};

pub(in crate::config::validation) fn check_toml_array_len_at_least(
    report: &mut ConfigValidationReport,
    value: &toml::Value,
    path: &[&str],
    minimum: usize,
    title: &'static str,
) {
    match toml_path(value, path).and_then(toml::Value::as_array) {
        Some(actual) if actual.len() >= minimum => report.pass(
            title,
            format!("{} has {} item(s).", dotted_path(path), actual.len()),
        ),
        Some(actual) => report.critical(
            title,
            format!(
                "{} has {} item(s), expected at least {minimum}.",
                dotted_path(path),
                actual.len()
            ),
        ),
        None => report.critical(
            title,
            format!("{} is missing or not an array.", dotted_path(path)),
        ),
    }
}
