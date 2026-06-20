mod finding;
mod patterns;
mod rules;

use super::super::model::LogFinding;

pub(super) fn classify_log_line(line_number: usize, line: &str) -> Option<LogFinding> {
    let lower = line.to_ascii_lowercase();
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    rules::classify_known_failure(line_number, trimmed, &lower)
}
