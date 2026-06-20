use std::path::Path;

use crate::types::{NodeConfig, NodeType};

use super::{
    command_path::{path_check, resolve_command_path},
    identity::runtime_identity_check,
    model::RuntimeBinaryPreflight,
    permissions::permission_checks,
};

pub fn inspect_node_binary(node: &NodeConfig) -> RuntimeBinaryPreflight {
    inspect_runtime_command(node.node_type, &node.binary_path, &node.args)
}

pub fn inspect_runtime_command(
    node_type: NodeType,
    binary_path: &Path,
    args: &[String],
) -> RuntimeBinaryPreflight {
    let resolved_path = resolve_command_path(binary_path);
    let mut checks = Vec::new();
    checks.push(path_check(binary_path, resolved_path.as_deref()));
    checks.extend(permission_checks(binary_path, resolved_path.as_deref()));
    checks.push(runtime_identity_check(
        node_type,
        binary_path,
        resolved_path.as_deref(),
        args,
    ));

    RuntimeBinaryPreflight {
        node_type,
        binary_path: binary_path.to_path_buf(),
        resolved_path,
        checks,
    }
}
