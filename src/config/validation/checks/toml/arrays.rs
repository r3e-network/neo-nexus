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

pub(in crate::config::validation) fn check_toml_string_array_exact(
    report: &mut ConfigValidationReport,
    value: &toml::Value,
    path: &[&str],
    expected: &[String],
    title: &'static str,
) {
    let path_label = dotted_path(path);
    let Some(array) = toml_path(value, path).and_then(toml::Value::as_array) else {
        report.critical(title, format!("{path_label} is missing or not an array."));
        return;
    };

    let mut actual = Vec::with_capacity(array.len());
    for (index, item) in array.iter().enumerate() {
        let Some(text) = item.as_str() else {
            report.critical(
                title,
                format!("{path_label}[{index}] is not a string value."),
            );
            return;
        };
        actual.push(text.to_string());
    }

    if actual == expected {
        report.pass(title, format!("{path_label} matches expected entries."));
    } else {
        report.critical(
            title,
            format!(
                "{path_label} differs from expected entries: found {}, expected {}.",
                actual.len(),
                expected.len()
            ),
        );
    }
}
