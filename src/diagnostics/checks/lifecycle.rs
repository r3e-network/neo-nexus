use crate::{
    diagnostics::{CheckSeverity, DiagnosticCheck, DiagnosticResolution},
    types::{NodeConfig, NodeStatus},
};

pub(in crate::diagnostics) fn launch_lifecycle_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    match node.status {
        NodeStatus::Running => vec![DiagnosticCheck::new(
            CheckSeverity::Critical,
            "Launch lifecycle",
            "Node is already marked running; stop it before starting again.",
            DiagnosticResolution::Monitor,
        )],
        NodeStatus::Starting => vec![DiagnosticCheck::new(
            CheckSeverity::Critical,
            "Launch lifecycle",
            "Node is already starting; wait for reconciliation before retrying.",
            DiagnosticResolution::Monitor,
        )],
        NodeStatus::Error => vec![DiagnosticCheck::new(
            CheckSeverity::Warning,
            "Launch lifecycle",
            "Node is in error state; a successful start will replace the failed session.",
            DiagnosticResolution::Logs,
        )],
        NodeStatus::Stopped => vec![DiagnosticCheck::new(
            CheckSeverity::Pass,
            "Launch lifecycle",
            "Node is stopped and eligible for start.",
            DiagnosticResolution::Operations,
        )],
    }
}

pub(in crate::diagnostics) fn restart_lifecycle_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    match node.status {
        NodeStatus::Running => vec![DiagnosticCheck::new(
            CheckSeverity::Pass,
            "Restart lifecycle",
            "Node is running and eligible for managed restart.",
            DiagnosticResolution::Operations,
        )],
        NodeStatus::Starting => vec![DiagnosticCheck::new(
            CheckSeverity::Critical,
            "Restart lifecycle",
            "Node is already starting; wait for reconciliation before restart.",
            DiagnosticResolution::Monitor,
        )],
        NodeStatus::Error => vec![DiagnosticCheck::new(
            CheckSeverity::Critical,
            "Restart lifecycle",
            "Node is in error state; use Start to recover from a stopped session.",
            DiagnosticResolution::Logs,
        )],
        NodeStatus::Stopped => vec![DiagnosticCheck::new(
            CheckSeverity::Critical,
            "Restart lifecycle",
            "Node is stopped; use Start instead of Restart.",
            DiagnosticResolution::Operations,
        )],
    }
}

pub(in crate::diagnostics) fn version_check(node: &NodeConfig) -> DiagnosticCheck {
    if node.runtime_version == "latest" {
        DiagnosticCheck::new(
            CheckSeverity::Info,
            "Version",
            "Runtime follows latest; pin an exact version for repeatable operations.",
            DiagnosticResolution::RuntimeManager,
        )
    } else {
        DiagnosticCheck::new(
            CheckSeverity::Pass,
            "Version",
            format!("Runtime is pinned to {}.", node.runtime_version),
            DiagnosticResolution::RuntimeManager,
        )
    }
}

pub(in crate::diagnostics) fn status_check(node: &NodeConfig) -> DiagnosticCheck {
    match (node.status, node.pid) {
        (NodeStatus::Running, Some(pid)) => DiagnosticCheck::new(
            CheckSeverity::Pass,
            "Lifecycle",
            format!("Process is running with PID {pid}."),
            DiagnosticResolution::Monitor,
        ),
        (NodeStatus::Running, None) => DiagnosticCheck::new(
            CheckSeverity::Critical,
            "Lifecycle",
            "Node is marked running but has no recorded PID.",
            DiagnosticResolution::Monitor,
        ),
        (NodeStatus::Stopped, None) => DiagnosticCheck::new(
            CheckSeverity::Info,
            "Lifecycle",
            "Node is stopped.",
            DiagnosticResolution::Operations,
        ),
        (NodeStatus::Stopped, Some(pid)) => DiagnosticCheck::new(
            CheckSeverity::Warning,
            "Lifecycle",
            format!("Stopped node still has recorded PID {pid}."),
            DiagnosticResolution::Monitor,
        ),
        (NodeStatus::Starting, _) => DiagnosticCheck::new(
            CheckSeverity::Info,
            "Lifecycle",
            "Node is starting.",
            DiagnosticResolution::Monitor,
        ),
        (NodeStatus::Error, _) => DiagnosticCheck::new(
            CheckSeverity::Critical,
            "Lifecycle",
            "Node is in error state.",
            DiagnosticResolution::Logs,
        ),
    }
}
