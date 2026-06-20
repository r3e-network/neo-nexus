use std::path::Path;

use crate::types::NodeType;

use super::model::{PreflightSeverity, RuntimePreflightCheck};

pub(super) fn runtime_identity_check(
    node_type: NodeType,
    binary_path: &Path,
    resolved_path: Option<&Path>,
    args: &[String],
) -> RuntimePreflightCheck {
    let binary_hints = runtime_hints(binary_path, resolved_path);
    let args_text = args.join(" ").to_ascii_lowercase();
    let matches = |values: &[&str]| {
        values
            .iter()
            .any(|value| binary_hints.iter().any(|hint| hint.contains(value)))
    };

    match node_type {
        NodeType::NeoCli => {
            if matches(&["neo-cli", "neo.cli"]) || args_text.contains("neo-cli.dll") {
                RuntimePreflightCheck {
                    severity: PreflightSeverity::Pass,
                    title: "Runtime identity",
                    detail: "Command matches Neo CLI runtime naming.".to_string(),
                }
            } else if matches(&["dotnet"]) && args_text.contains("neo.cli.dll") {
                RuntimePreflightCheck {
                    severity: PreflightSeverity::Pass,
                    title: "Runtime identity",
                    detail: "dotnet wrapper targets Neo.CLI.dll.".to_string(),
                }
            } else if matches(&["dotnet"]) {
                RuntimePreflightCheck {
                    severity: PreflightSeverity::Warning,
                    title: "Runtime identity",
                    detail: "dotnet wrapper is present, but args do not reference Neo.CLI.dll."
                        .to_string(),
                }
            } else {
                RuntimePreflightCheck {
                    severity: PreflightSeverity::Warning,
                    title: "Runtime identity",
                    detail: "Binary name does not look like neo-cli or dotnet Neo.CLI.dll."
                        .to_string(),
                }
            }
        }
        NodeType::NeoGo => {
            if matches(&["neo-go"]) {
                RuntimePreflightCheck {
                    severity: PreflightSeverity::Pass,
                    title: "Runtime identity",
                    detail: "Command matches neo-go runtime naming.".to_string(),
                }
            } else {
                RuntimePreflightCheck {
                    severity: PreflightSeverity::Warning,
                    title: "Runtime identity",
                    detail: "Binary name does not look like neo-go.".to_string(),
                }
            }
        }
        NodeType::NeoRs => {
            if matches(&["neo-node", "neo-rs", "neors"]) {
                RuntimePreflightCheck {
                    severity: PreflightSeverity::Pass,
                    title: "Runtime identity",
                    detail: "Command matches neo-rs neo-node runtime naming.".to_string(),
                }
            } else {
                RuntimePreflightCheck {
                    severity: PreflightSeverity::Warning,
                    title: "Runtime identity",
                    detail: "Binary name does not look like neo-rs neo-node.".to_string(),
                }
            }
        }
    }
}

fn runtime_hints(binary_path: &Path, resolved_path: Option<&Path>) -> Vec<String> {
    let mut hints = Vec::new();
    push_path_hints(&mut hints, binary_path);
    if let Some(resolved_path) = resolved_path {
        push_path_hints(&mut hints, resolved_path);
    }
    hints
}

fn push_path_hints(hints: &mut Vec<String>, path: &Path) {
    if let Some(file_name) = path.file_name().and_then(|value| value.to_str()) {
        hints.push(file_name.to_ascii_lowercase());
    }
    if let Some(file_stem) = path.file_stem().and_then(|value| value.to_str()) {
        hints.push(file_stem.to_ascii_lowercase());
    }
}
