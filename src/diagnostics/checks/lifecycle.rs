use crate::{
    diagnostics::{CheckSeverity, DiagnosticCheck},
    types::{NodeConfig, NodeStatus},
};

pub(in crate::diagnostics) fn launch_lifecycle_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    match node.status {
        NodeStatus::Running => vec![DiagnosticCheck {
            severity: CheckSeverity::Critical,
            title: "Launch lifecycle",
            detail: "Node is already marked running; stop it before starting again.".to_string(),
        }],
        NodeStatus::Starting => vec![DiagnosticCheck {
            severity: CheckSeverity::Critical,
            title: "Launch lifecycle",
            detail: "Node is already starting; wait for reconciliation before retrying."
                .to_string(),
        }],
        NodeStatus::Error => vec![DiagnosticCheck {
            severity: CheckSeverity::Warning,
            title: "Launch lifecycle",
            detail: "Node is in error state; a successful start will replace the failed session."
                .to_string(),
        }],
        NodeStatus::Stopped => vec![DiagnosticCheck {
            severity: CheckSeverity::Pass,
            title: "Launch lifecycle",
            detail: "Node is stopped and eligible for start.".to_string(),
        }],
    }
}

pub(in crate::diagnostics) fn restart_lifecycle_checks(node: &NodeConfig) -> Vec<DiagnosticCheck> {
    match node.status {
        NodeStatus::Running => vec![DiagnosticCheck {
            severity: CheckSeverity::Pass,
            title: "Restart lifecycle",
            detail: "Node is running and eligible for managed restart.".to_string(),
        }],
        NodeStatus::Starting => vec![DiagnosticCheck {
            severity: CheckSeverity::Critical,
            title: "Restart lifecycle",
            detail: "Node is already starting; wait for reconciliation before restart.".to_string(),
        }],
        NodeStatus::Error => vec![DiagnosticCheck {
            severity: CheckSeverity::Critical,
            title: "Restart lifecycle",
            detail: "Node is in error state; use Start to recover from a stopped session."
                .to_string(),
        }],
        NodeStatus::Stopped => vec![DiagnosticCheck {
            severity: CheckSeverity::Critical,
            title: "Restart lifecycle",
            detail: "Node is stopped; use Start instead of Restart.".to_string(),
        }],
    }
}

pub(in crate::diagnostics) fn version_check(node: &NodeConfig) -> DiagnosticCheck {
    if node.runtime_version == "latest" {
        DiagnosticCheck {
            severity: CheckSeverity::Info,
            title: "Version",
            detail: "Runtime follows latest; pin an exact version for repeatable operations."
                .to_string(),
        }
    } else {
        DiagnosticCheck {
            severity: CheckSeverity::Pass,
            title: "Version",
            detail: format!("Runtime is pinned to {}.", node.runtime_version),
        }
    }
}

pub(in crate::diagnostics) fn status_check(node: &NodeConfig) -> DiagnosticCheck {
    match (node.status, node.pid) {
        (NodeStatus::Running, Some(pid)) => DiagnosticCheck {
            severity: CheckSeverity::Pass,
            title: "Lifecycle",
            detail: format!("Process is running with PID {pid}."),
        },
        (NodeStatus::Running, None) => DiagnosticCheck {
            severity: CheckSeverity::Critical,
            title: "Lifecycle",
            detail: "Node is marked running but has no recorded PID.".to_string(),
        },
        (NodeStatus::Stopped, None) => DiagnosticCheck {
            severity: CheckSeverity::Info,
            title: "Lifecycle",
            detail: "Node is stopped.".to_string(),
        },
        (NodeStatus::Stopped, Some(pid)) => DiagnosticCheck {
            severity: CheckSeverity::Warning,
            title: "Lifecycle",
            detail: format!("Stopped node still has recorded PID {pid}."),
        },
        (NodeStatus::Starting, _) => DiagnosticCheck {
            severity: CheckSeverity::Info,
            title: "Lifecycle",
            detail: "Node is starting.".to_string(),
        },
        (NodeStatus::Error, _) => DiagnosticCheck {
            severity: CheckSeverity::Critical,
            title: "Lifecycle",
            detail: "Node is in error state.".to_string(),
        },
    }
}
