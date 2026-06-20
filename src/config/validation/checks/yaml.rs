use super::{
    super::model::ConfigValidationReport,
    paths::{dotted_path, yaml_path},
};

pub(in crate::config::validation) fn check_yaml_string(
    report: &mut ConfigValidationReport,
    value: &serde_yaml::Value,
    path: &[&str],
    expected: &str,
    title: &'static str,
) {
    match yaml_path(value, path).and_then(serde_yaml::Value::as_str) {
        Some(actual) if actual == expected => {
            report.pass(title, format!("{} is {expected}.", dotted_path(path)));
        }
        Some(actual) => report.critical(
            title,
            format!("{} is {actual}, expected {expected}.", dotted_path(path)),
        ),
        None => report.critical(
            title,
            format!("{} is missing or not text.", dotted_path(path)),
        ),
    }
}

pub(in crate::config::validation) fn check_yaml_u16(
    report: &mut ConfigValidationReport,
    value: &serde_yaml::Value,
    path: &[&str],
    expected: u16,
    title: &'static str,
) {
    check_yaml_u64(report, value, path, expected as u64, title);
}

pub(in crate::config::validation) fn check_yaml_u8(
    report: &mut ConfigValidationReport,
    value: &serde_yaml::Value,
    path: &[&str],
    expected: u8,
    title: &'static str,
) {
    check_yaml_u64(report, value, path, expected as u64, title);
}

pub(in crate::config::validation) fn check_yaml_u32(
    report: &mut ConfigValidationReport,
    value: &serde_yaml::Value,
    path: &[&str],
    expected: u32,
    title: &'static str,
) {
    check_yaml_u64(report, value, path, expected as u64, title);
}

fn check_yaml_u64(
    report: &mut ConfigValidationReport,
    value: &serde_yaml::Value,
    path: &[&str],
    expected: u64,
    title: &'static str,
) {
    match yaml_path(value, path)
        .and_then(serde_yaml::Value::as_i64)
        .and_then(|actual| u64::try_from(actual).ok())
    {
        Some(actual) if actual == expected => {
            report.pass(title, format!("{} is {expected}.", dotted_path(path)));
        }
        Some(actual) => report.critical(
            title,
            format!("{} is {actual}, expected {expected}.", dotted_path(path)),
        ),
        None => report.critical(
            title,
            format!(
                "{} is missing or not an unsigned integer.",
                dotted_path(path)
            ),
        ),
    }
}

pub(in crate::config::validation) fn check_yaml_array_len_at_least(
    report: &mut ConfigValidationReport,
    value: &serde_yaml::Value,
    path: &[&str],
    minimum: usize,
    title: &'static str,
) {
    match yaml_path(value, path).and_then(serde_yaml::Value::as_sequence) {
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
