use std::path::{Path, PathBuf};

use crate::snapshots::sha256_file;

use super::{RuntimeSmokeBinaryEvidence, RuntimeSmokeBinaryEvidenceStatus};

pub(super) fn runtime_binary_evidence(
    command_path: &Path,
    node_args: &[String],
) -> RuntimeSmokeBinaryEvidence {
    let runtime_path = runtime_target_path(command_path, node_args);
    match sha256_file(&runtime_path) {
        Ok((sha256, bytes)) => RuntimeSmokeBinaryEvidence {
            command_path: command_path.to_path_buf(),
            runtime_path,
            sha256: Some(sha256),
            bytes: Some(bytes),
            status: RuntimeSmokeBinaryEvidenceStatus::Verified,
            message: "Runtime binary hash recorded.".to_string(),
        },
        Err(error) => RuntimeSmokeBinaryEvidence {
            command_path: command_path.to_path_buf(),
            runtime_path,
            sha256: None,
            bytes: None,
            status: RuntimeSmokeBinaryEvidenceStatus::Unavailable,
            message: format!("Runtime binary hash unavailable: {error}"),
        },
    }
}

fn runtime_target_path(command_path: &Path, node_args: &[String]) -> PathBuf {
    if is_runtime_wrapper(command_path) {
        if let Some(target) = wrapped_runtime_target(node_args) {
            return target;
        }
    }
    command_path.to_path_buf()
}

fn wrapped_runtime_target(node_args: &[String]) -> Option<PathBuf> {
    node_args
        .iter()
        .filter(|arg| !is_wrapper_control_arg(arg))
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

fn is_wrapper_control_arg(arg: &str) -> bool {
    let normalized = arg.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "/c" | "-c" | "-command" | "-file" | "-executionpolicy" | "bypass"
    ) || normalized.starts_with('-')
}

fn is_runtime_wrapper(command_path: &Path) -> bool {
    let command = command_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        command.as_str(),
        "sh" | "bash" | "dash" | "zsh" | "cmd" | "powershell" | "pwsh" | "dotnet"
    )
}
