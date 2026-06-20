use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LaunchPackValidationCheck {
    pub category: String,
    pub label: String,
    pub status: LaunchPackValidationStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchPackValidationStatus {
    Pass,
    Warn,
    Fail,
}

impl LaunchPackValidationStatus {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }
}
