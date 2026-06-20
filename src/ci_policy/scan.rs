use std::{collections::BTreeSet, path::PathBuf};

use serde_yaml::Value;

use super::{
    model::{CiPolicyFinding, CiPolicyReport},
    requirements::{
        required_commands, FORBIDDEN_CI_MARKERS, REQUIRED_OS, REQUIRED_RELEASE_COMMANDS,
    },
    yaml::collect_matrix_os,
};

pub(super) fn build_report(
    workflow_path: PathBuf,
    text: &str,
    yaml: &Value,
    checked_at_unix: u64,
) -> CiPolicyReport {
    let required_os = REQUIRED_OS
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let found_os = collect_matrix_os(yaml);
    let found_os_set = found_os.iter().cloned().collect::<BTreeSet<_>>();
    let missing_os = required_os
        .iter()
        .filter(|os| !found_os_set.contains(*os))
        .cloned()
        .collect::<Vec<_>>();

    let policy_commands = required_commands()
        .chain(REQUIRED_RELEASE_COMMANDS.iter())
        .collect::<Vec<_>>();
    let required_commands = policy_commands
        .iter()
        .map(|command| command.label.to_string())
        .collect::<Vec<_>>();
    let missing_commands = policy_commands
        .iter()
        .filter(|command| !text.contains(command.fragment))
        .map(|command| command.label.to_string())
        .collect::<Vec<_>>();

    let mut findings = Vec::new();
    for os in &missing_os {
        findings.push(CiPolicyFinding {
            category: "missing-os".to_string(),
            marker: os.clone(),
            message: format!("CI matrix must include {os} for desktop release confidence"),
        });
    }
    for command in policy_commands {
        if missing_commands.iter().any(|label| label == command.label) {
            findings.push(CiPolicyFinding {
                category: "missing-command".to_string(),
                marker: command.label.to_string(),
                message: format!("missing required CI command fragment: {}", command.fragment),
            });
        }
    }

    let forbidden_findings = forbidden_ci_findings(text);
    let forbidden_count = forbidden_findings.len();
    findings.extend(forbidden_findings);

    let status = if findings.is_empty() {
        "native-ci"
    } else {
        "failed"
    }
    .to_string();
    let finding_count = findings.len();

    CiPolicyReport {
        schema_version: 1,
        status,
        checked_at_unix,
        workflow_path,
        required_os,
        found_os,
        missing_os,
        required_commands,
        missing_commands,
        forbidden_count,
        finding_count,
        findings,
    }
}

fn forbidden_ci_findings(text: &str) -> Vec<CiPolicyFinding> {
    let normalized = text.to_lowercase();
    FORBIDDEN_CI_MARKERS
        .iter()
        .filter(|marker| normalized.contains(marker.marker))
        .map(|marker| CiPolicyFinding {
            category: "forbidden-ci-tooling".to_string(),
            marker: marker.marker.to_string(),
            message: marker.message.to_string(),
        })
        .collect()
}
