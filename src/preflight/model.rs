use std::path::PathBuf;

use serde::Serialize;

use crate::types::NodeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreflightSeverity {
    Pass,
    Info,
    Warning,
    Critical,
}

impl PreflightSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimePreflightCheck {
    pub severity: PreflightSeverity,
    pub title: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeBinaryPreflight {
    pub node_type: NodeType,
    pub binary_path: PathBuf,
    pub resolved_path: Option<PathBuf>,
    pub checks: Vec<RuntimePreflightCheck>,
}

impl RuntimeBinaryPreflight {
    pub fn has_blockers(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.severity == PreflightSeverity::Critical)
    }

    pub fn has_warnings(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.severity == PreflightSeverity::Warning)
    }

    pub fn status_label(&self) -> &'static str {
        if self.has_blockers() {
            "blocked"
        } else if self.has_warnings() {
            "review"
        } else {
            "ready"
        }
    }

    pub fn blocker_details(&self) -> Vec<String> {
        self.checks
            .iter()
            .filter(|check| check.severity == PreflightSeverity::Critical)
            .map(|check| check.detail.clone())
            .collect()
    }

    pub fn launch_blocker(&self) -> Option<String> {
        let details = self.blocker_details();
        if details.is_empty() {
            None
        } else {
            Some(details.join("; "))
        }
    }

    pub fn summary(&self) -> String {
        let pass = self.count(PreflightSeverity::Pass);
        let info = self.count(PreflightSeverity::Info);
        let warning = self.count(PreflightSeverity::Warning);
        let critical = self.count(PreflightSeverity::Critical);
        format!("{pass} pass, {info} info, {warning} warning, {critical} critical")
    }

    pub fn operator_summary(&self) -> String {
        if let Some(blocker) = self.launch_blocker() {
            return blocker;
        }

        if let Some(warning) = self
            .checks
            .iter()
            .find(|check| check.severity == PreflightSeverity::Warning)
        {
            return warning.detail.clone();
        }

        self.summary()
    }

    fn count(&self, severity: PreflightSeverity) -> usize {
        self.checks
            .iter()
            .filter(|check| check.severity == severity)
            .count()
    }
}
