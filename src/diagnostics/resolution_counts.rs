use super::DiagnosticResolution;

pub(super) fn empty_resolution_counts() -> Vec<(DiagnosticResolution, usize)> {
    DiagnosticResolution::ALL
        .into_iter()
        .map(|resolution| (resolution, 0))
        .collect()
}

pub(super) fn increment_resolution_count(
    counts: &mut [(DiagnosticResolution, usize)],
    resolution: DiagnosticResolution,
) {
    if let Some((_, count)) = counts
        .iter_mut()
        .find(|(counted, _)| *counted == resolution)
    {
        *count += 1;
    }
}
