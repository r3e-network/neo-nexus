use crate::logs::LogDiagnosisStatus;

use super::super::patterns::{contains_any, is_database_lock, is_port_binding_failure};

#[derive(Debug, Clone, Copy)]
pub(super) struct LogFailureRule {
    pub(super) label: &'static str,
    pub(super) recommendation: &'static str,
    pub(super) status: LogDiagnosisStatus,
    pub(super) matcher: LogRuleMatcher,
}

impl LogFailureRule {
    pub(super) fn matches(self, lower: &str) -> bool {
        match self.matcher {
            LogRuleMatcher::PortBinding => is_port_binding_failure(lower),
            LogRuleMatcher::DatabaseLock => is_database_lock(lower),
            LogRuleMatcher::ContainsAny(markers) => contains_any(lower, markers),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum LogRuleMatcher {
    PortBinding,
    DatabaseLock,
    ContainsAny(&'static [&'static str]),
}
