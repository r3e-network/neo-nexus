use super::{CheckSeverity, DiagnosticCheck};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchReadinessReport {
    pub node_id: String,
    pub node_name: String,
    pub display_command: String,
    pub checks: Vec<DiagnosticCheck>,
}

impl LaunchReadinessReport {
    pub fn is_blocked(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.severity == CheckSeverity::Critical)
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_blocked() {
            "blocked"
        } else if self
            .checks
            .iter()
            .any(|check| check.severity == CheckSeverity::Warning)
        {
            "review"
        } else {
            "ready"
        }
    }

    pub fn blocking_summary(&self) -> Option<String> {
        self.checks
            .iter()
            .find(|check| check.severity == CheckSeverity::Critical)
            .map(|check| format!("{}: {}", check.title, check.detail))
    }

    pub fn operator_summary(&self) -> String {
        if let Some(blocker) = self.blocking_summary() {
            return blocker;
        }

        if let Some(warning) = self
            .checks
            .iter()
            .find(|check| check.severity == CheckSeverity::Warning)
        {
            return format!("{}: {}", warning.title, warning.detail);
        }

        "Ready to launch with generated managed config.".to_string()
    }
}
