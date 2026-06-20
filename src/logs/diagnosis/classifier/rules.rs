mod catalog;
mod model;

use super::finding::finding;
use crate::logs::LogFinding;

use self::catalog::known_failure_rules;

pub(super) fn classify_known_failure(
    line_number: usize,
    trimmed: &str,
    lower: &str,
) -> Option<LogFinding> {
    known_failure_rules()
        .iter()
        .find(|rule| rule.matches(lower))
        .map(|rule| {
            finding(
                rule.label,
                line_number,
                trimmed,
                rule.recommendation,
                rule.status,
            )
        })
}
