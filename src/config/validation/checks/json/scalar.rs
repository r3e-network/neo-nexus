use serde_json::Value;

use super::super::{
    super::model::ConfigValidationReport,
    paths::{dotted_path, json_path},
};

pub(in crate::config::validation) fn check_json_string(
    report: &mut ConfigValidationReport,
    value: &Value,
    path: &[&str],
    expected: &str,
    title: &'static str,
) {
    match json_path(value, path).and_then(Value::as_str) {
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

pub(in crate::config::validation) fn check_json_bool(
    report: &mut ConfigValidationReport,
    value: &Value,
    path: &[&str],
    expected: bool,
    title: &'static str,
) {
    match json_path(value, path).and_then(Value::as_bool) {
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

pub(in crate::config::validation) fn check_json_u16(
    report: &mut ConfigValidationReport,
    value: &Value,
    path: &[&str],
    expected: u16,
    title: &'static str,
) {
    check_json_u64(report, value, path, expected as u64, title);
}

pub(in crate::config::validation) fn check_json_u8(
    report: &mut ConfigValidationReport,
    value: &Value,
    path: &[&str],
    expected: u8,
    title: &'static str,
) {
    check_json_u64(report, value, path, expected as u64, title);
}

pub(in crate::config::validation) fn check_json_u32(
    report: &mut ConfigValidationReport,
    value: &Value,
    path: &[&str],
    expected: u32,
    title: &'static str,
) {
    check_json_u64(report, value, path, expected as u64, title);
}

fn check_json_u64(
    report: &mut ConfigValidationReport,
    value: &Value,
    path: &[&str],
    expected: u64,
    title: &'static str,
) {
    match json_path(value, path).and_then(Value::as_u64) {
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
