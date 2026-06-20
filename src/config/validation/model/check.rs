use serde::Serialize;

use super::severity::ConfigValidationSeverity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigValidationCheck {
    pub severity: ConfigValidationSeverity,
    pub title: &'static str,
    pub detail: String,
}
