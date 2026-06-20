use super::{CheckSeverity, DiagnosticCheck};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDiagnostics {
    pub node_id: String,
    pub node_name: String,
    pub score: usize,
    pub checks: Vec<DiagnosticCheck>,
}

impl NodeDiagnostics {
    pub fn critical_count(&self) -> usize {
        self.count(CheckSeverity::Critical)
    }

    pub fn warning_count(&self) -> usize {
        self.count(CheckSeverity::Warning)
    }

    pub fn is_ready(&self) -> bool {
        self.critical_count() == 0 && self.warning_count() == 0
    }

    fn count(&self, severity: CheckSeverity) -> usize {
        self.checks
            .iter()
            .filter(|check| check.severity == severity)
            .count()
    }
}
