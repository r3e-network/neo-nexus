use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigValidationSeverity {
    Pass,
    Warning,
    Critical,
}

impl ConfigValidationSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}
