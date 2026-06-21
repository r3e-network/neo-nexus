/// Every CLI option the dispatcher recognizes. Used to point an operator at the
/// closest match when they mistype a flag. Kept in lockstep with the dispatcher
/// by `known_options_are_all_recognized` in the CLI tests.
pub(super) const KNOWN_OPTIONS: &[&str] = &[
    "-V",
    "--version",
    "-h",
    "--help",
    "--self-check",
    "--runtime-smoke",
    "--runtime-smoke-json",
    "--rpc-health",
    "--rpc-health-json",
    "--workspace-readiness",
    "--workspace-readiness-json",
    "--workspace-metrics",
    "--workspace-metrics-json",
    "--workspace-metrics-prometheus",
    "--workspace-integrity",
    "--workspace-integrity-json",
    "--source-purity",
    "--source-purity-json",
    "--source-quality",
    "--source-quality-json",
    "--native-ui-audit",
    "--native-ui-audit-json",
    "--ci-policy",
    "--ci-policy-json",
    "--alert-preview",
    "--alert-preview-json",
    "--export-readiness-report",
    "--export-support-bundle",
    "--export-support-bundle-json",
    "--export-event-journal",
    "--export-node-configs",
    "--export-node-configs-json",
    "--generate-node-config",
    "--generate-node-config-json",
    "--validate-node-config",
    "--validate-node-config-json",
    "--export-backup",
    "--export-backup-json",
    "--import-backup",
    "--import-backup-json",
    "--validate-backup",
    "--validate-backup-json",
    "--validate-wallet",
    "--validate-wallet-json",
    "--import-wallet-profile",
    "--validate-launch-pack",
    "--launch-pack-sidecars",
    "--launch-pack-sidecars-json",
    "--package-release",
    "--verify-release-package",
    "--verify-release-package-json",
];

/// Returns the recognized option closest to `unknown`, but only when the edit
/// distance is small enough to be a plausible typo rather than a coincidental
/// near-match to an unrelated flag.
pub(super) fn suggest_option(unknown: &str) -> Option<&'static str> {
    let threshold = suggestion_threshold(unknown);
    KNOWN_OPTIONS
        .iter()
        .map(|candidate| (*candidate, edit_distance(unknown, candidate)))
        .filter(|(_, distance)| *distance <= threshold)
        .min_by_key(|(_, distance)| *distance)
        .map(|(candidate, _)| candidate)
}

fn suggestion_threshold(unknown: &str) -> usize {
    (unknown.chars().count() / 4).max(2)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right_chars: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right_chars.len()).collect();
    let mut current = vec![0usize; right_chars.len() + 1];

    for (left_index, left_char) in left.chars().enumerate() {
        current[0] = left_index + 1;
        for (right_index, &right_char) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_char != right_char);
            current[right_index + 1] = (previous[right_index] + substitution_cost)
                .min(previous[right_index + 1] + 1)
                .min(current[right_index] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right_chars.len()]
}

#[cfg(test)]
#[path = "../../../tests/unit/cli/actions/suggest/tests.rs"]
mod tests;
