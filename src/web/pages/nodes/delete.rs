//! Node deletion flow with two-stage confirmation and cleanup.

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    core::operations::{EventKind, EventSeverity, NewRuntimeEvent},
    web::{html, WebState},
};

/// The confirmation step. Deleting drops plugin state, managed installs and RPC
/// health history, so the operator reads that before agreeing rather than
/// discovering it afterwards.
pub async fn delete_form(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let node = match state.workspace.list_nodes() {
        Ok(nodes) => nodes.into_iter().find(|node| node.id == id),
        Err(_) => None,
    };
    let Some(node) = node else {
        return Redirect::to("/nodes").into_response();
    };
    let encoded = html::urlencoding_lite(&node.id);
    let detail =
        "If the node is running, NeoNexus first stops it and confirms the process exited. \
                  Also removed: plugin state, managed plugin installs and RPC health history for \
                  this node. The node's own files and chain data stay on disk.";
    let body = format!(
        r#"{breadcrumb}
{head}
{detail}
<form method="post" action="/nodes/{encoded}/delete">
<div class="form-actions">
<button class="danger" type="submit">Delete {name}</button>
<a class="btn" href="/nodes/{encoded}">Cancel</a>
</div>
</form>"#,
        breadcrumb = html::breadcrumb(&[
            ("Nodes", "/nodes"),
            (&node.name, &format!("/nodes/{encoded}")),
            ("Delete", ""),
        ]),
        head = html::page_head(
            &format!("Delete {}?", node.name),
            "This removes the node from the workspace. It cannot be undone.",
            "",
        ),
        detail = html::notice("danger", detail),
        name = html::escape(&node.name),
    );
    Html(html::layout("Delete node", "nodes", "", &body)).into_response()
}

pub async fn delete(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let node = state
        .workspace
        .list_nodes()
        .ok()
        .and_then(|nodes| nodes.into_iter().find(|node| node.id == id));
    let name = node
        .as_ref()
        .map(|node| node.name.clone())
        .unwrap_or_else(|| id.clone());

    // Keep the name/id snapshot for the post-delete audit event. Recording a
    // NodeDeleted event before the guarded delete could leave a false audit
    // claim when a concurrent Start wins the race.
    let outcome = (|| -> anyhow::Result<()> {
        let node = node.ok_or_else(|| anyhow::anyhow!("node {id} was not found"))?;
        if node.status.is_active() || node.pid.is_some() {
            crate::supervision::stop_node(&state.engine_state(), &node)?;
        }
        state.commands.delete_node(&id)
    })();

    let message = match outcome {
        Ok(()) => match state.commands.record_event(NewRuntimeEvent {
            node_id: Some(id.clone()),
            node_name: Some(name.clone()),
            kind: EventKind::NodeDeleted,
            severity: EventSeverity::Warning,
            message: format!("{name} deleted"),
        }) {
            Ok(_) => format!("{name} deleted."),
            Err(error) => format!("{name} deleted, but its audit event failed: {error}"),
        },
        Err(error) => format!("delete failed: {error}"),
    };
    Redirect::to(&format!(
        "/nodes?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}
