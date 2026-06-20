use super::CheckSeverity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCheck {
    pub severity: CheckSeverity,
    pub title: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCheckKey {
    pub severity: CheckSeverity,
    pub title: String,
    pub detail: String,
}

impl DiagnosticCheck {
    pub fn key(&self) -> DiagnosticCheckKey {
        DiagnosticCheckKey {
            severity: self.severity,
            title: self.title.to_string(),
            detail: self.detail.clone(),
        }
    }
}

impl DiagnosticCheckKey {
    pub fn matches(&self, check: &DiagnosticCheck) -> bool {
        self.severity == check.severity && self.title == check.title && self.detail == check.detail
    }
}
