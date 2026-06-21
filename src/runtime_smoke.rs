mod attempt;
mod binary;
mod evaluation;
mod identity;
mod model;
mod preflight;
mod probes;
mod text;

#[cfg(test)]
#[path = "../tests/unit/runtime_smoke/tests.rs"]
mod tests;

use std::{path::Path, time::Duration};

use crate::types::{NodeConfig, NodeType};

use self::{
    attempt::run_attempt,
    binary::runtime_binary_evidence,
    evaluation::{ProbeEvaluation, ProbeOutcome},
    preflight::{inspect_smoke_preflight, resolved_command_path},
    probes::runtime_probe_args,
};

pub use model::{
    RuntimeSmokeAttempt, RuntimeSmokeBinaryEvidence, RuntimeSmokeBinaryEvidenceStatus,
    RuntimeSmokeReport, RuntimeSmokeStatus,
};

pub fn smoke_node_binary(node: &NodeConfig, timeout: Duration) -> RuntimeSmokeReport {
    smoke_runtime_command(node.node_type, &node.binary_path, &node.args, timeout)
}

pub fn smoke_runtime_command(
    node_type: NodeType,
    binary_path: &Path,
    node_args: &[String],
    timeout: Duration,
) -> RuntimeSmokeReport {
    let preflight = inspect_smoke_preflight(node_type, binary_path, node_args);
    let command_path = resolved_command_path(&preflight, binary_path);
    let binary_evidence = runtime_binary_evidence(command_path, node_args);

    if let Some(blocker) = preflight.launch_blocker() {
        return RuntimeSmokeReport {
            node_type,
            binary_path: binary_path.to_path_buf(),
            preflight,
            binary_evidence,
            status: RuntimeSmokeStatus::Blocked,
            message: blocker,
            attempts: Vec::new(),
        };
    }

    let probes = runtime_probe_args(node_type, binary_path, node_args);
    let mut attempts = Vec::new();
    let mut evaluation = ProbeEvaluation::default();

    for args in probes {
        match run_attempt(command_path, &args, timeout) {
            Ok(attempt) => {
                let outcome = evaluation.record_attempt(node_type, &attempt);
                attempts.push(attempt);
                if let ProbeOutcome::Passed(message) = outcome {
                    return RuntimeSmokeReport {
                        node_type,
                        binary_path: binary_path.to_path_buf(),
                        preflight,
                        binary_evidence,
                        status: RuntimeSmokeStatus::Passed,
                        message,
                        attempts,
                    };
                }
            }
            Err(error) => {
                evaluation.record_error(error.to_string());
            }
        }
    }

    let (status, message) = evaluation.finish(timeout);

    RuntimeSmokeReport {
        node_type,
        binary_path: binary_path.to_path_buf(),
        preflight,
        binary_evidence,
        status,
        message,
        attempts,
    }
}
