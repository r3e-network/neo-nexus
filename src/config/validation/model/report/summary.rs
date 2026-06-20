use super::ConfigValidationReport;
use crate::config::validation::ConfigValidationSeverity;

impl ConfigValidationReport {
    pub fn is_success(&self) -> bool {
        !self
            .checks
            .iter()
            .any(|check| check.severity == ConfigValidationSeverity::Critical)
    }

    pub fn has_warnings(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.severity == ConfigValidationSeverity::Warning)
    }

    pub fn status_label(&self) -> &'static str {
        if !self.is_success() {
            "invalid"
        } else if self.has_warnings() {
            "review"
        } else {
            "ready"
        }
    }

    pub fn exit_code(&self) -> i32 {
        if self.is_success() {
            0
        } else {
            1
        }
    }

    pub fn summary(&self) -> String {
        let pass = self.count(ConfigValidationSeverity::Pass);
        let warning = self.count(ConfigValidationSeverity::Warning);
        let critical = self.count(ConfigValidationSeverity::Critical);
        format!("{pass} pass, {warning} warning, {critical} critical")
    }

    pub fn operator_summary(&self) -> String {
        if let Some(check) = self
            .checks
            .iter()
            .find(|check| check.severity == ConfigValidationSeverity::Critical)
        {
            return format!("{}: {}", check.title, check.detail);
        }

        if let Some(check) = self
            .checks
            .iter()
            .find(|check| check.severity == ConfigValidationSeverity::Warning)
        {
            return format!("{}: {}", check.title, check.detail);
        }

        self.summary()
    }

    fn count(&self, severity: ConfigValidationSeverity) -> usize {
        self.checks
            .iter()
            .filter(|check| check.severity == severity)
            .count()
    }
}
