use crate::types::NodeConfig;

use super::super::super::{checks::*, model::ConfigValidationReport};

pub(super) fn check(node: &NodeConfig, report: &mut ConfigValidationReport, value: &toml::Value) {
    check_toml_string(
        report,
        value,
        &["storage", "backend"],
        "rocksdb",
        "Storage backend",
    );
    check_toml_string(
        report,
        value,
        &["storage", "data_dir"],
        &format!("./data/{}", node.network),
        "Data directory",
    );
    check_toml_bool(
        report,
        value,
        &["storage", "read_only"],
        false,
        "Read-only storage",
    );
}
