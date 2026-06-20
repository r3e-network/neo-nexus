use super::ConfigValidationReport;
use crate::config::validation::{ConfigValidationCheck, ConfigValidationSeverity};

impl ConfigValidationReport {
    pub(in crate::config::validation) fn push(
        &mut self,
        severity: ConfigValidationSeverity,
        title: &'static str,
        detail: impl Into<String>,
    ) {
        self.checks.push(ConfigValidationCheck {
            severity,
            title,
            detail: detail.into(),
        });
    }

    pub(in crate::config::validation) fn pass(
        &mut self,
        title: &'static str,
        detail: impl Into<String>,
    ) {
        self.push(ConfigValidationSeverity::Pass, title, detail);
    }

    pub(in crate::config::validation) fn warning(
        &mut self,
        title: &'static str,
        detail: impl Into<String>,
    ) {
        self.push(ConfigValidationSeverity::Warning, title, detail);
    }

    pub(in crate::config::validation) fn critical(
        &mut self,
        title: &'static str,
        detail: impl Into<String>,
    ) {
        self.push(ConfigValidationSeverity::Critical, title, detail);
    }
}
