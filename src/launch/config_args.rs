use crate::types::NodeType;

pub fn runtime_args_include_config(node_type: NodeType, args: &[String]) -> bool {
    match node_type {
        NodeType::NeoCli => false,
        NodeType::NeoGo => has_neo_go_config_arg(args),
        NodeType::NeoRs => has_neo_rs_config_arg(args),
        NodeType::NeoXGeth | NodeType::NeoXReth => has_neox_config_arg(args),
    }
}

/// Both Neo X clients spell it `--config` and neither accepts `-c`: Reth
/// declares the field `#[arg(long)]` with no short alias, and geth's is
/// `--config` too. Reporting `-c` as an operator-supplied config told the
/// operator NeoNexus would leave their file alone while the planner went on to
/// inject `--config` as well, putting two conflicting flags on one command line.
pub(super) fn has_neox_config_arg(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--config" || arg.starts_with("--config="))
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
