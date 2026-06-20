use super::super::*;

pub(in crate::private_network) fn collect_port(
    ports: &mut BTreeMap<u16, Vec<String>>,
    port: u16,
    label: String,
) {
    ports.entry(port).or_default().push(label);
}

pub(in crate::private_network) fn add_file_check(
    checks: &mut Vec<LaunchPackValidationCheck>,
    category: &str,
    label: impl Into<String>,
    path: &Path,
) {
    add_check(
        checks,
        category,
        label,
        if path.is_file() {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        path.display().to_string(),
    );
}

pub(in crate::private_network) fn add_dir_check(
    checks: &mut Vec<LaunchPackValidationCheck>,
    category: &str,
    label: impl Into<String>,
    path: &Path,
) {
    add_check(
        checks,
        category,
        label,
        if path.is_dir() {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        path.display().to_string(),
    );
}

pub(in crate::private_network) fn add_check(
    checks: &mut Vec<LaunchPackValidationCheck>,
    category: &str,
    label: impl Into<String>,
    status: LaunchPackValidationStatus,
    message: String,
) {
    checks.push(LaunchPackValidationCheck {
        category: category.to_string(),
        label: label.into(),
        status,
        message,
    });
}
