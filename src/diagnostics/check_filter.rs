use super::{CheckSeverity, DiagnosticCheck};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticCheckFilter {
    pub severity: Option<CheckSeverity>,
    pub query: String,
}

impl DiagnosticCheckFilter {
    pub fn new(severity: Option<CheckSeverity>, query: impl Into<String>) -> Self {
        Self {
            severity,
            query: query.into(),
        }
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
        .filter(|check| query.is_empty() || check_matches(check, &query))
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by(check_order);
    rows
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
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
mod tests;
