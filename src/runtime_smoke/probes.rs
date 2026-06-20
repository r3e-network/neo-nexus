use std::path::Path;

use crate::types::NodeType;

pub(super) fn runtime_probe_args(
    node_type: NodeType,
    binary_path: &Path,
    node_args: &[String],
) -> Vec<Vec<String>> {
    match node_type {
        NodeType::NeoCli => {
            let base_args = neo_cli_base_args(binary_path, node_args);
            [
                appended(&base_args, "--version"),
                appended(&base_args, "--help"),
                appended(&base_args, "help"),
            ]
            .into_iter()
            .collect()
        }
        NodeType::NeoGo => {
            let base_args = runtime_wrapper_base_args(binary_path, node_args);
            vec![
                appended(&base_args, "--version"),
                appended(&base_args, "version"),
                appended_many(&base_args, &["node", "--help"]),
            ]
        }
        NodeType::NeoRs => {
            let base_args = runtime_wrapper_base_args(binary_path, node_args);
            vec![
                appended(&base_args, "--version"),
                appended(&base_args, "version"),
                appended(&base_args, "--help"),
                appended_many(&base_args, &["node", "--help"]),
            ]
        }
    }
}

fn neo_cli_base_args(binary_path: &Path, node_args: &[String]) -> Vec<String> {
    let binary_name = binary_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !binary_name.contains("dotnet") {
        return Vec::new();
    }

    let mut base = Vec::new();
    for arg in node_args {
        base.push(arg.clone());
        let normalized = arg.to_ascii_lowercase();
        if normalized.ends_with("neo.cli.dll") || normalized.ends_with("neo-cli.dll") {
            break;
        }
    }
    base
}

fn appended(base: &[String], arg: &str) -> Vec<String> {
    let mut args = base.to_vec();
    args.push(arg.to_string());
    args
}

fn appended_many(base: &[String], args: &[&str]) -> Vec<String> {
    let mut output = base.to_vec();
    output.extend(args.iter().map(|arg| (*arg).to_string()));
    output
}

fn runtime_wrapper_base_args(binary_path: &Path, node_args: &[String]) -> Vec<String> {
    if is_runtime_wrapper(binary_path) {
        node_args.to_vec()
    } else {
        Vec::new()
    }
}

fn is_runtime_wrapper(binary_path: &Path) -> bool {
    let command = binary_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        command.as_str(),
        "sh" | "bash" | "dash" | "zsh" | "cmd" | "powershell" | "pwsh"
    )
}
