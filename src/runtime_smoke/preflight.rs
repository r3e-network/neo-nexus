use std::path::Path;

use crate::{
    preflight::{inspect_runtime_command, RuntimeBinaryPreflight},
    types::NodeType,
};

pub(super) fn inspect_smoke_preflight(
    node_type: NodeType,
    binary_path: &Path,
    node_args: &[String],
) -> RuntimeBinaryPreflight {
    inspect_runtime_command(node_type, binary_path, node_args)
}

pub(super) fn resolved_command_path<'a>(
    preflight: &'a RuntimeBinaryPreflight,
    requested_path: &'a Path,
) -> &'a Path {
    preflight.resolved_path.as_deref().unwrap_or(requested_path)
}
