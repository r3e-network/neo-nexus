use super::super::{
    super::model::ConfigValidationReport,
    paths::{dotted_path, toml_path},
};

pub(in crate::config::validation) fn check_toml_string(
    report: &mut ConfigValidationReport,
    value: &toml::Value,
    path: &[&str],
    expected: &str,
    title: &'static str,
) {
    match toml_path(value, path).and_then(toml::Value::as_str) {
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

pub(in crate::config::validation) fn check_toml_bool(
    report: &mut ConfigValidationReport,
    value: &toml::Value,
    path: &[&str],
    expected: bool,
    title: &'static str,
) {
    match toml_path(value, path).and_then(toml::Value::as_bool) {
        Some(actual) if actual == expected => {
            report.pass(title, format!("{} is {expected}.", dotted_path(path)));
        }
        Some(actual) => report.critical(
            title,
            format!("{} is {actual}, expected {expected}.", dotted_path(path)),
        ),
        None => report.critical(
            title,
            format!("{} is missing or not boolean.", dotted_path(path)),
        ),
    }
}

pub(in crate::config::validation) fn check_toml_u16(
    report: &mut ConfigValidationReport,
    value: &toml::Value,
    path: &[&str],
    expected: u16,
    title: &'static str,
) {
    check_toml_u64(report, value, path, expected as u64, title);
}

pub(in crate::config::validation) fn check_toml_u32(
    report: &mut ConfigValidationReport,
    value: &toml::Value,
    path: &[&str],
    expected: u32,
    title: &'static str,
) {
    check_toml_u64(report, value, path, expected as u64, title);
}

fn check_toml_u64(
    report: &mut ConfigValidationReport,
    value: &toml::Value,
    path: &[&str],
    expected: u64,
    title: &'static str,
) {
    match toml_path(value, path)
        .and_then(toml::Value::as_integer)
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
