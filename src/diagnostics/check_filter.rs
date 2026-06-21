use super::{
    resolution_counts::{empty_resolution_counts, increment_resolution_count},
    severity_counts::{empty_severity_counts, increment_severity_count},
    CheckSeverity, DiagnosticCheck, DiagnosticResolution,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticCheckFilter {
    pub severity: Option<CheckSeverity>,
    pub query: String,
    pub resolution: Option<DiagnosticResolution>,
}

impl DiagnosticCheckFilter {
    pub fn new(severity: Option<CheckSeverity>, query: impl Into<String>) -> Self {
        Self {
            severity,
            query: query.into(),
            resolution: None,
        }
    }

    pub fn with_resolution(mut self, resolution: Option<DiagnosticResolution>) -> Self {
        self.resolution = resolution;
        self
    }
}

pub fn filter_diagnostic_checks(
    checks: &[DiagnosticCheck],
    filter: &DiagnosticCheckFilter,
) -> Vec<DiagnosticCheck> {
    let query = filter.query.trim().to_lowercase();
    let mut rows = checks
        .iter()
        .filter(|check| {
            filter
                .severity
                .is_none_or(|severity| check.severity == severity)
        })
        .filter(|check| {
            filter
                .resolution
                .is_none_or(|resolution| check.resolution == resolution)
        })
        .filter(|check| query.is_empty() || check_matches(check, &query))
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by(check_order);
    rows
}

pub fn diagnostic_check_resolution_counts(
    checks: &[DiagnosticCheck],
    filter: &DiagnosticCheckFilter,
) -> Vec<(DiagnosticResolution, usize)> {
    let query = filter.query.trim().to_lowercase();
    let mut counts = empty_resolution_counts();
    for check in checks {
        if filter
            .severity
            .is_some_and(|severity| check.severity != severity)
        {
            continue;
        }
        if !query.is_empty() && !check_matches(check, &query) {
            continue;
        }
        increment_resolution_count(&mut counts, check.resolution);
    }
    counts
}

pub fn diagnostic_check_severity_counts(
    checks: &[DiagnosticCheck],
    filter: &DiagnosticCheckFilter,
) -> Vec<(CheckSeverity, usize)> {
    let count_filter =
        DiagnosticCheckFilter::new(None, filter.query.as_str()).with_resolution(filter.resolution);
    let mut counts = empty_severity_counts();
    for check in filter_diagnostic_checks(checks, &count_filter) {
        increment_severity_count(&mut counts, check.severity);
    }
    counts
}

fn check_order(left: &DiagnosticCheck, right: &DiagnosticCheck) -> std::cmp::Ordering {
    right
        .severity
        .cmp(&left.severity)
        .then_with(|| left.title.cmp(right.title))
        .then_with(|| left.detail.cmp(&right.detail))
}

fn check_matches(check: &DiagnosticCheck, query: &str) -> bool {
    text_matches(check.severity.label(), query)
        || text_matches(check.title, query)
        || text_matches(&check.detail, query)
        || check.resolution.matches_query(query)
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
#[path = "../../tests/unit/diagnostics/check_filter/tests.rs"]
mod tests;
