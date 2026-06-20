use std::path::Path;

use crate::{
    argv::format_command,
    redaction::redact_sensitive_args,
    types::{NodeConfig, NodeType},
};

use super::{
    config_args::{has_neo_go_config_arg, has_neo_rs_config_arg},
    LaunchPlan,
};

pub struct LaunchPlanner;

impl LaunchPlanner {
    pub fn plan(
        node: &NodeConfig,
        managed_config_path: impl AsRef<Path>,
        working_dir: impl AsRef<Path>,
    ) -> LaunchPlan {
        let managed_config_path = managed_config_path.as_ref().to_path_buf();
        let working_dir = working_dir.as_ref().to_path_buf();
        let mut args = node.args.clone();
        let managed_config_path = match node.node_type {
            NodeType::NeoRs => neo_rs_managed_config_path(&mut args, managed_config_path),
            NodeType::NeoGo => neo_go_managed_config_path(&mut args, managed_config_path),
            NodeType::NeoCli => Some(managed_config_path),
        };
        let display_command = format_command(&node.binary_path, &redact_sensitive_args(&args));

        LaunchPlan {
            binary_path: node.binary_path.clone(),
            args,
            working_dir,
            managed_config_path,
            display_command,
        }
    }
}

fn neo_rs_managed_config_path(
    args: &mut Vec<String>,
    managed_config_path: std::path::PathBuf,
) -> Option<std::path::PathBuf> {
    if has_neo_rs_config_arg(args) {
        return None;
    }
    args.push("--config".to_string());
    args.push(managed_config_path.display().to_string());
    Some(managed_config_path)
}

fn neo_go_managed_config_path(
    args: &mut Vec<String>,
    managed_config_path: std::path::PathBuf,
) -> Option<std::path::PathBuf> {
    if args.first().is_none_or(|arg| arg != "node") {
        args.insert(0, "node".to_string());
    }
    if has_neo_go_config_arg(args) {
        return None;
    }
    args.push("--config-file".to_string());
    args.push(managed_config_path.display().to_string());
    Some(managed_config_path)
}
