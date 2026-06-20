use super::CheckSeverity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCheck {
    pub severity: CheckSeverity,
    pub title: &'static str,
    pub detail: String,
}
