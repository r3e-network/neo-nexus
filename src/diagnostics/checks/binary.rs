use crate::{
    diagnostics::{CheckSeverity, DiagnosticCheck, DiagnosticResolution},
    preflight::{inspect_node_binary, PreflightSeverity, RuntimePreflightCheck},
    types::NodeConfig,
};

pub(in crate::diagnostics) fn binary_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    inspect_node_binary(node)
        .checks
        .into_iter()
        .map(diagnostic_from_preflight)
        .collect()
}

fn diagnostic_from_preflight(check: RuntimePreflightCheck) -> DiagnosticCheck {
    DiagnosticCheck::new(
        severity_from_preflight(check.severity),
        check.title,
        check.detail,
        DiagnosticResolution::RuntimeManager,
    )
}

fn severity_from_preflight(severity: PreflightSeverity) -> CheckSeverity {
    match severity {
        PreflightSeverity::Pass => CheckSeverity::Pass,
        PreflightSeverity::Info => CheckSeverity::Info,
        PreflightSeverity::Warning => CheckSeverity::Warning,
        PreflightSeverity::Critical => CheckSeverity::Critical,
    }
}
