//! Maintenance control operations: snapshot application, backup export, and log cleanup.

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    core::operations::{EventKind, EventSeverity, NewRuntimeEvent},
    snapshots::FastSyncSnapshotManager,
};

use super::{
    super::{html, WebState},
    node::{back_to_node, load_node},
};

pub async fn apply_snapshot(
    State(state): State<WebState>,
    Path((snapshot_id, node_id)): Path<(String, String)>,
) -> Response {
    use crate::types::node_workspace_path;

    let outcome = (|| -> anyhow::Result<String> {
        // Load the snapshot
        let snapshot = state
            .workspace
            .list_fast_sync_snapshots()?
            .into_iter()
            .find(|s| s.id == snapshot_id)
            .ok_or_else(|| anyhow::anyhow!("snapshot {snapshot_id} was not found"))?;

        // Verify snapshot is ready to apply (has verified hash and cached file)
        if snapshot.verified_sha256.is_none() {
            anyhow::bail!("snapshot {snapshot_id} has not been hash-verified");
        }
        if snapshot.cached_path.is_none() {
            anyhow::bail!("snapshot {snapshot_id} has not been downloaded");
        }

        // Load the target node
        let node = load_node(&state.workspace, &node_id)?;

        // Construct node data directory path
        const NODES_DIR: &str = "nodes";
        let node_workspace = node_workspace_path(state.workspace_child_dir(NODES_DIR), &node.id)?;
        let node_data_dir = node_workspace
            .join("data")
            .join(snapshot.network.to_string());

        // Apply the snapshot
        let application = FastSyncSnapshotManager::apply_to_node(&snapshot, &node, &node_data_dir)?;

        let message = format!(
            "snapshot '{}' applied to node '{}' ({}) — {} files, {} decompressed",
            snapshot.label,
            node.id,
            node.name,
            application.imported_files,
            crate::core::operations::format_bytes(application.expanded_bytes),
        );

        // The apply already happened; a journal failure must not be reported as
        // if the apply itself had failed.
        let _ = state.commands.record_event(NewRuntimeEvent {
            node_id: Some(node.id.clone()),
            node_name: Some(node.name.clone()),
            kind: EventKind::SnapshotApplied,
            severity: EventSeverity::Info,
            message: message.clone(),
        });

        Ok(message)
    })();

    match outcome {
        Ok(message) => back_to_node(&node_id, &message),
        Err(error) => back_to_node(&node_id, &format!("failed: {error}")),
    }
}

pub async fn handle_backup_export(State(state): State<WebState>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let output = state.workspace_child_dir("export").join("backup");
        let export = state
            .workspace
            .export_backup(&output, env!("CARGO_PKG_VERSION"))?;

        // Calculate total profile count
        let total_profiles = export.remote_server_count
            + export.runtime_catalog_profile_count
            + export.runtime_signer_profile_count
            + export.neo_wallet_profile_count;

        let message = format!(
            "workspace backup exported — {} nodes, {} profiles, {} snapshots, {} events → {}",
            export.node_count,
            total_profiles,
            export.fast_sync_snapshot_count,
            export.event_count,
            export.path.display(),
        );
        let _ = state.commands.record_event(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::BackupExported,
            severity: EventSeverity::Info,
            message: message.clone(),
        });
        Ok(message)
    })();

    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("backup export failed: {error}"),
    };
    Redirect::to(&format!(
        "/backup?flash={}",
        html::urlencoding_lite(&message),
    ))
    .into_response()
}

pub async fn clear_logs(State(state): State<WebState>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        // Get the logs directory
        let logs_dir = state.workspace_child_dir("logs");

        // Clear all log files
        let cleared_count = crate::logs::clear_all_logs(&logs_dir)?;

        // Record audit event
        let message = format!("cleared {} log file(s)", cleared_count);
        let _ = state.commands.record_event(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::LogCleared,
            severity: EventSeverity::Info,
            message: message.clone(),
        });

        Ok(message)
    })();

    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("failed to clear logs: {error}"),
    };

    Redirect::to(&format!("/logs?flash={}", html::urlencoding_lite(&message))).into_response()
}
