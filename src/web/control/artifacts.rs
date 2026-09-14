//! The artifacts an incident needs, produced from the browser.
//!
//! Every one of these was implemented, tested, and reachable only from the CLI.
//! That is exactly backwards: **the operator filing a ticket is the one least
//! likely to have a shell on the host.** A support bundle that can only be
//! produced by someone with SSH is not a support bundle, it is a second
//! escalation.
//!
//! `EventKind::SupportBundleExported` already existed so the journal could
//! *display* a bundle this console could not trigger.
//!
//! Each one journals what it did. `--import-backup` and
//! `--reconcile-node-config` mutated a workspace with no audit entry at all,
//! which makes the operations most worth recording the ones that left no trace.

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
};

use crate::core::operations::{EventKind, EventSeverity, NewRuntimeEvent};

use super::super::{html, WebState};

/// Write a support bundle: readiness, integrity, metrics and a redacted log
/// diagnosis, checksummed.
pub async fn export_support_bundle(State(state): State<WebState>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let output = state.workspace_child_dir("export").join("support");
        let bundle = state
            .commands
            .export_support_bundle(&output, env!("CARGO_PKG_VERSION"))?;
        Ok(format!(
            "support bundle written to {} ({} bytes, sha256 {})",
            bundle.archive_path.display(),
            bundle.archive_bytes,
            bundle.archive_sha256,
        ))
    })();
    journal_and_return(&state, EventKind::SupportBundleExported, outcome)
}

/// Write the readiness report the CLI has always been able to produce.
pub async fn export_readiness_report(State(state): State<WebState>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let nodes = state.workspace.list_nodes()?;
        let plugin_states = nodes
            .iter()
            .map(|node| {
                state
                    .workspace
                    .list_plugin_states(&node.id)
                    .map(|states| (node.id.clone(), states))
            })
            .collect::<anyhow::Result<std::collections::BTreeMap<_, _>>>()?;
        let diagnostics = crate::core::operations::evaluate_fleet(&nodes, &plugin_states);
        let output = state.workspace_child_dir("export").join("readiness");
        let export = state.commands.export_readiness_report(
            &output,
            &diagnostics,
            env!("CARGO_PKG_VERSION"),
        )?;
        Ok(format!(
            "readiness report written to {} — score {}/100, {} of {} node(s) ready",
            export.text_path.display(),
            diagnostics.score,
            diagnostics.ready_nodes,
            nodes.len(),
        ))
    })();
    journal_and_return(&state, EventKind::WorkspaceReadinessReportExported, outcome)
}

/// Check the workspace database against the schema this build expects.
///
/// Computed for every support bundle and never shown live, so the one moment an
/// operator wants it — "is my workspace itself damaged" — was the one moment
/// they could not ask.
pub async fn check_workspace_integrity(State(state): State<WebState>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let report = state
            .commands
            .check_workspace_integrity(env!("CARGO_PKG_VERSION"))?;
        Ok(format!(
            "workspace integrity: {} — {} table(s) and {} index(es) checked, {} foreign-key \
             violation(s)",
            report.status_label(),
            report.required_tables.len(),
            report.required_indexes.len(),
            report.foreign_key_violations.len(),
        ))
    })();
    journal_and_return(&state, EventKind::WorkspaceIntegrityChecked, outcome)
}

/// Rewrite one node's managed config from what this workspace would generate.
///
/// The counterpart to the drift check the Configuration page now runs: seeing
/// that a file has drifted and having no way to put it back is half a feature.
pub async fn reconcile_node_config(
    State(state): State<WebState>,
    Path(node_id): Path<String>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let node = state
            .workspace
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == node_id)
            .ok_or_else(|| anyhow::anyhow!("node {node_id} was not found"))?;
        let plugins = state.workspace.list_plugin_states(&node.id)?;
        let work_dir =
            crate::types::node_workspace_path(state.workspace_child_dir("nodes"), &node.id)?;
        let target = crate::core::workspace::ConfigExporter::managed_target_path(&work_dir, &node);
        let _ = plugins;
        let report = state.commands.reconcile_node_config(&node, &target)?;
        Ok(if report.reconciled {
            format!(
                "{}: config rewritten ({} bytes){}; now {}",
                report.node_name,
                report.bytes_written,
                report
                    .backup_path
                    .as_ref()
                    .map(|path| format!(", previous file kept at {}", path.display()))
                    .unwrap_or_default(),
                report.post_check_status.label(),
            )
        } else {
            format!(
                "{}: already matches what this workspace would generate",
                report.node_name
            )
        })
    })();
    journal_and_return(&state, EventKind::ConfigApplied, outcome)
}

/// Record the outcome and send the operator back to where they pressed the
/// button, with the result in front of them.
fn journal_and_return(
    state: &WebState,
    kind: EventKind,
    outcome: anyhow::Result<String>,
) -> Response {
    let (message, severity) = match outcome {
        Ok(message) => (message, EventSeverity::Info),
        Err(error) => (format!("{kind} failed: {error:#}"), EventSeverity::Warning),
    };
    let _ = state.commands.record_event(NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind,
        severity,
        message: message.clone(),
    });
    Redirect::to(&format!(
        "/operations?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}
