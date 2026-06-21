mod defaults;
mod download;
mod manifest;
mod model;
mod release;

use crate::app::domain::NodeType;

pub(super) use model::RuntimePackageDraft;

pub(super) const BYTES_PER_MIB: u64 = 1024 * 1024;

pub(super) fn bytes_to_mib_ceil(bytes: u64) -> u64 {
    bytes.saturating_add(BYTES_PER_MIB - 1) / BYTES_PER_MIB
}

pub(super) fn optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(super) fn default_runtime_node_type() -> NodeType {
    NodeType::NeoRs
}
