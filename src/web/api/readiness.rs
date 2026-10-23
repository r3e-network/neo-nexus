//! The readiness answer, in one shape.
//!
//! Two surfaces need it: the authenticated `/api/readiness` for scripted
//! operators, and the public `/ready` probe for an orchestrator. They must not
//! drift apart, so both are built from the same diagnostics and the same
//! status vocabulary — `ready`, `review`, `blocked`.
//!
//! The authenticated payload carries the findings and the per-node checks,
//! because that is where an operator is told what to do next. The public probe
//! carries counts only: a load balancer needs to know whether to route here,
//! not which node is broken or where its configuration lives.

use serde::Serialize;

use crate::diagnostics::{CheckSeverity, DiagnosticCheck, FleetDiagnostics, NodeDiagnostics};

use super::super::html::urlencoding_lite;

/// The full readiness answer, for a caller that is allowed to see the fleet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadinessPayload {
    /// `ready`, `review` or `blocked`.
    pub status: &'static str,
    pub score: usize,
    pub node_count: usize,
    pub ready_nodes: usize,
    pub warning_count: usize,
    pub critical_count: usize,
    pub findings: Vec<ReadinessFinding>,
    pub nodes: Vec<ReadinessNode>,
}

/// One thing that needs a person, and where to go about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadinessFinding {
    pub severity: &'static str,
    pub node_id: String,
    pub node_name: String,
    pub title: &'static str,
    pub detail: String,
    /// Stable identifier of the surface that resolves this finding.
    pub resolution: &'static str,
    pub resolution_label: &'static str,
    pub resolution_action: &'static str,
    pub resolution_hint: &'static str,
    /// Where the operator goes, ready to click.
    pub resolution_href: String,
}

/// One node's verdict and the checks behind it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadinessNode {
    pub node_id: String,
    pub node_name: String,
    pub score: usize,
    /// `ready`, `review` or `blocked`.
    pub status: &'static str,
    pub critical_count: usize,
    pub warning_count: usize,
    pub checks: Vec<ReadinessCheck>,
}

/// One check on one node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadinessCheck {
    pub severity: &'static str,
    pub title: &'static str,
    pub detail: String,
    pub resolution: &'static str,
    pub resolution_action: &'static str,
    pub resolution_hint: &'static str,
}

/// Build the authenticated payload.
pub fn payload(diagnostics: &FleetDiagnostics) -> ReadinessPayload {
    ReadinessPayload {
        status: crate::readiness_report::readiness_status(diagnostics),
        score: diagnostics.score,
        node_count: diagnostics.nodes.len(),
        ready_nodes: diagnostics.ready_nodes,
        warning_count: diagnostics.warning_count,
        critical_count: diagnostics.critical_count,
        findings: findings(diagnostics),
        nodes: diagnostics.nodes.iter().map(node).collect(),
    }
}

/// The public probe's answer: the verdict and the counts, nothing that names a
/// node or a path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadinessProbe {
    /// `ready`, `review` or `blocked`.
    pub status: &'static str,
    pub score: usize,
    pub node_count: usize,
    pub ready_nodes: usize,
    pub warning_count: usize,
    pub critical_count: usize,
}

/// Build the public probe's answer.
pub fn probe(diagnostics: &FleetDiagnostics) -> ReadinessProbe {
    ReadinessProbe {
        status: crate::readiness_report::readiness_status(diagnostics),
        score: diagnostics.score,
        node_count: diagnostics.nodes.len(),
        ready_nodes: diagnostics.ready_nodes,
        warning_count: diagnostics.warning_count,
        critical_count: diagnostics.critical_count,
    }
}

fn findings(diagnostics: &FleetDiagnostics) -> Vec<ReadinessFinding> {
    [CheckSeverity::Critical, CheckSeverity::Warning]
        .into_iter()
        .flat_map(|severity| {
            diagnostics
                .nodes
                .iter()
                .flat_map(move |node| findings_for(node, severity))
        })
        .collect()
}

fn findings_for(node: &NodeDiagnostics, severity: CheckSeverity) -> Vec<ReadinessFinding> {
    node.checks
        .iter()
        .filter(|check| check.severity == severity)
        .map(|check| ReadinessFinding {
            severity: severity.label(),
            node_id: node.node_id.clone(),
            node_name: node.node_name.clone(),
            title: check.title,
            detail: check.detail.clone(),
            resolution: check.resolution.key(),
            resolution_label: check.resolution.label(),
            resolution_action: check.resolution.action_label(),
            resolution_hint: check.resolution.hint(),
            resolution_href: check.resolution.href(&node.node_id, urlencoding_lite),
        })
        .collect()
}

fn node(node: &NodeDiagnostics) -> ReadinessNode {
    ReadinessNode {
        node_id: node.node_id.clone(),
        node_name: node.node_name.clone(),
        score: node.score,
        status: node_status(node),
        critical_count: node.critical_count(),
        warning_count: node.warning_count(),
        checks: node.checks.iter().map(check).collect(),
    }
}

fn node_status(node: &NodeDiagnostics) -> &'static str {
    if node.critical_count() > 0 {
        "blocked"
    } else if node.warning_count() > 0 {
        "review"
    } else {
        "ready"
    }
}

fn check(check: &DiagnosticCheck) -> ReadinessCheck {
    ReadinessCheck {
        severity: check.severity.label(),
        title: check.title,
        detail: check.detail.clone(),
        resolution: check.resolution.key(),
        resolution_action: check.resolution.action_label(),
        resolution_hint: check.resolution.hint(),
    }
}
