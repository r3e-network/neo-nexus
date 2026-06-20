use crate::types::NodeConfig;

use super::super::super::{
    super::format::max_transactions_per_block, checks::*, model::ConfigValidationReport,
};

pub(super) fn check(node: &NodeConfig, report: &mut ConfigValidationReport, value: &toml::Value) {
    check_toml_u32(
        report,
        value,
        &["blockchain", "block_time"],
        15_000,
        "Block time",
    );
    check_toml_u32(
        report,
        value,
        &["blockchain", "max_transactions_per_block"],
        max_transactions_per_block(node.network),
        "Max transactions",
    );
}
