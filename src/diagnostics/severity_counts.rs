use super::CheckSeverity;

pub(super) fn empty_severity_counts() -> Vec<(CheckSeverity, usize)> {
    [
        CheckSeverity::Critical,
        CheckSeverity::Warning,
        CheckSeverity::Info,
        CheckSeverity::Pass,
    ]
    .into_iter()
    .map(|severity| (severity, 0))
    .collect()
}

pub(super) fn increment_severity_count(
    counts: &mut [(CheckSeverity, usize)],
    severity: CheckSeverity,
) {
    if let Some((_, count)) = counts.iter_mut().find(|(counted, _)| *counted == severity) {
        *count += 1;
    }
}
