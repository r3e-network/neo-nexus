use serde::Serialize;

use crate::{config::format::ConfigFormat, types::NodeType};

use super::check::ConfigValidationCheck;

mod checks;
mod summary;
mod text;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigValidationReport {
    pub node_type: NodeType,
    pub format: ConfigFormat,
    pub checks: Vec<ConfigValidationCheck>,
}
