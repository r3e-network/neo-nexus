use super::*;

mod counts;
mod signer;

pub(in crate::private_network) fn check_committee(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    committee: &DeploymentCommitteeManifest,
) {
    counts::check_committee_summary(checks, committee);
    check_signer_labels(checks, committee);

    let mut seen_keys = BTreeMap::new();
    for signer in &committee.signers {
        signer::check_committee_signer(checks, root_path, signer, &mut seen_keys);
    }
}

fn check_signer_labels(
    checks: &mut Vec<LaunchPackValidationCheck>,
    committee: &DeploymentCommitteeManifest,
) {
    let mut blank_labels = 0usize;
    let mut labels: BTreeMap<String, (String, usize)> = BTreeMap::new();

    for signer in &committee.signers {
        let label = signer.label.trim();
        if label.is_empty() {
            blank_labels += 1;
            continue;
        }
        let entry = labels
            .entry(label.to_ascii_lowercase())
            .or_insert_with(|| (label.to_string(), 0));
        entry.1 += 1;
    }

    let duplicate_labels = labels
        .values()
        .filter(|(_, count)| *count > 1)
        .map(|(label, count)| format!("{label} x{count}"))
        .collect::<Vec<_>>();

    add_check(
        checks,
        "committee",
        "signer-labels",
        if blank_labels == 0 && duplicate_labels.is_empty() {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        if blank_labels == 0 && duplicate_labels.is_empty() {
            format!("{} unique non-empty labels", committee.signers.len())
        } else {
            signer_label_failure_message(blank_labels, duplicate_labels)
        },
    );
}

fn signer_label_failure_message(blank_labels: usize, duplicate_labels: Vec<String>) -> String {
    let mut findings = Vec::new();
    if blank_labels > 0 {
        findings.push(format!("blank labels: {blank_labels}"));
    }
    if !duplicate_labels.is_empty() {
        findings.push(format!("duplicates: {}", duplicate_labels.join(", ")));
    }
    findings.join("; ")
}
