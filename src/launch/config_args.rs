use crate::types::NodeType;

pub fn runtime_args_include_config(node_type: NodeType, args: &[String]) -> bool {
    match node_type {
        NodeType::NeoCli => false,
        NodeType::NeoGo => has_neo_go_config_arg(args),
        NodeType::NeoRs => has_neo_rs_config_arg(args),
    }
}

pub(super) fn has_neo_rs_config_arg(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--config" || arg == "-c" || arg.starts_with("--config="))
}

pub(super) fn has_neo_go_config_arg(args: &[String]) -> bool {
    args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "--config" | "-c" | "--config-file" | "--config-path"
        ) || arg.starts_with("--config=")
            || arg.starts_with("--config-file=")
            || arg.starts_with("--config-path=")
    })
}
