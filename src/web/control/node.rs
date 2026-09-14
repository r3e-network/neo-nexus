//! Node lifecycle control operations: start, stop, restart, and smoke test.

use std::time::Duration;

use axum::{
    extract::{Path, RawForm, State},
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    core::{
        lifecycle::LaunchAction,
        operations::{EventKind, EventSeverity, NewRuntimeEvent},
    },
    runtime_smoke, supervision,
    types::NodeConfig,
};

use super::super::{html, WebState};

pub async fn node_start(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    blocking(move || control_redirect(&state, &id, LaunchAction::Start)).await
}

pub async fn node_restart(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    blocking(move || control_redirect(&state, &id, LaunchAction::Restart)).await
}

pub async fn node_stop(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    blocking(move || {
        let outcome = load_node(&state.workspace, &id)
            .and_then(|node| supervision::stop_node(&state.engine_state(), &node));
        match outcome {
            Ok(message) => back_to_node(&id, &message),
            Err(error) => back_to_node(&id, &format!("stop failed: {error}")),
        }
    })
    .await
}

/// Run a lifecycle operation off the async executor.
///
/// Starting, restarting and stopping a node all block: they render config to
/// disk, spawn or signal a process, and wait out the launch settle window or
/// the termination grace period. Holding a Tokio worker thread for that stalls
/// every other request the server is serving, so the work moves to the blocking
/// pool and only the response comes back here.
async fn blocking<F>(work: F) -> Response
where
    F: FnOnce() -> Response + Send + 'static,
{
    match tokio::task::spawn_blocking(work).await {
        Ok(response) => response,
        // The only way to get here is a panic inside the operation. The node is
        // in whatever state the panic left it, so say so rather than implying
        // the request did nothing.
        Err(error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("the lifecycle operation did not complete: {error}"),
        )
            .into_response(),
    }
}

pub async fn smoke_test_node(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    // First load the node synchronously
    let node = match load_node(&state.workspace, &id) {
        Ok(n) => n,
        Err(error) => return back_to_node(&id, &format!("node not found: {error}")),
    };

    // Run smoke test in a blocking thread since it spawns processes
    let report = match tokio::task::spawn_blocking({
        let node_config = node.clone();
        move || runtime_smoke::smoke_node_binary(&node_config, Duration::from_secs(3))
    })
    .await
    .map_err(|e| anyhow::anyhow!("smoke test task failed: {e}"))
    {
        Ok(r) => r,
        Err(e) => {
            return back_to_node(&id, &format!("smoke test failed: {e}"));
        }
    };

    let message = format!(
        "runtime smoke test {} — {}",
        if report.status.is_success() {
            "passed"
        } else {
            "failed"
        },
        report.message
    );

    // Record event with severity matching status
    let _ = state.commands.record_event(NewRuntimeEvent {
        node_id: Some(id.clone()),
        node_name: Some(node.name.clone()),
        kind: EventKind::RuntimeSmokeTested,
        severity: if report.status.is_success() {
            EventSeverity::Info
        } else {
            EventSeverity::Critical // ⚠️ Failed smoke tests indicate potential binary corruption
        },
        message: message.clone(),
    });

    back_to_node(&id, &message)
}

pub(crate) fn control_redirect(state: &WebState, id: &str, action: LaunchAction) -> Response {
    let node = match load_node(&state.workspace, id) {
        Ok(node) => node,
        Err(error) => return back_to_node(id, &format!("failed: {error}")),
    };
    match supervision::launch_node(&state.engine_state(), &node, action) {
        Ok(message) => back_to_node(id, &message),
        Err(error) => {
            let message = format!("failed: {error}");
            // A manual launch failure must leave a trail, not just a flash that
            // vanishes on the next page load. The watchdog already journals
            // NodeStartFailed for its automatic retries, so a hand-driven start
            // or restart that fails records the same way — "why did this never
            // come up at 03:00?" then has an answer.
            let _ = state.commands.record_event(NewRuntimeEvent {
                node_id: Some(node.id.clone()),
                node_name: Some(node.name.clone()),
                kind: EventKind::NodeStartFailed,
                severity: EventSeverity::Warning,
                message: message.clone(),
            });
            back_to_node(id, &message)
        }
    }
}

pub(crate) fn back_to_node(id: &str, message: &str) -> Response {
    Redirect::to(&format!(
        "/nodes/{}?flash={}",
        html::urlencoding_lite(id),
        html::urlencoding_lite(message),
    ))
    .into_response()
}

pub(crate) fn load_node(
    workspace: &crate::core::workspace_queries::WorkspaceQueries,
    id: &str,
) -> anyhow::Result<NodeConfig> {
    workspace
        .list_nodes()?
        .into_iter()
        .find(|node| node.id == id)
        .ok_or_else(|| anyhow::anyhow!("node {id} was not found"))
}

pub async fn batch_node_action(State(state): State<WebState>, RawForm(body): RawForm) -> Response {
    let mut action_slug = String::new();
    let mut node_ids = Vec::new();
    for (name, value) in url::form_urlencoded::parse(&body) {
        if name == "action" {
            action_slug = value.trim().to_ascii_lowercase();
        } else if name == "node_ids" {
            for part in value.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() && !node_ids.iter().any(|id| id == trimmed) {
                    node_ids.push(trimmed.to_string());
                }
            }
        }
    }
    if node_ids.is_empty() {
        return Redirect::to("/nodes?flash=No%20instances%20selected%20for%20batch%20action")
            .into_response();
    }
    let mut successes = 0;
    let mut failures = 0;
    for id in &node_ids {
        let outcome = match action_slug.as_str() {
            "start" | "restart" | "stop" => {
                // Same reason as the single-node controls: each of these blocks
                // on process work, and a batch multiplies it by the selection.
                let state = state.clone();
                let id = id.clone();
                let slug = action_slug.clone();
                tokio::task::spawn_blocking(move || {
                    let node = load_node(&state.workspace, &id)?;
                    let engine = state.engine_state();
                    match slug.as_str() {
                        "start" => supervision::launch_node(&engine, &node, LaunchAction::Start),
                        "restart" => {
                            supervision::launch_node(&engine, &node, LaunchAction::Restart)
                        }
                        _ => supervision::stop_node(&engine, &node),
                    }
                })
                .await
                .map_err(|error| anyhow::anyhow!("{action_slug} did not complete: {error}"))
                .and_then(|outcome| outcome)
            }
            "smoke" => {
                let report_res = match load_node(&state.workspace, id) {
                    Ok(node) => tokio::task::spawn_blocking(move || {
                        runtime_smoke::smoke_node_binary(&node, Duration::from_secs(3))
                    })
                    .await
                    .map_err(|e| anyhow::anyhow!("smoke test task failed: {e}")),
                    Err(e) => Err(e),
                };
                match report_res {
                    Ok(report) => {
                        let _ = state.commands.record_event(NewRuntimeEvent {
                            node_id: Some(id.clone()),
                            node_name: None,
                            kind: EventKind::RuntimeSmokeTested,
                            severity: if report.status.is_success() {
                                EventSeverity::Info
                            } else {
                                EventSeverity::Critical
                            },
                            message: format!("Batch smoke test: {}", report.message),
                        });
                        if report.status.is_success() {
                            Ok(report.message)
                        } else {
                            Err(anyhow::anyhow!("{}", report.message))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            other => {
                return Redirect::to(&format!(
                    "/nodes?flash={}",
                    html::urlencoding_lite(&format!("Unknown batch action '{other}'"))
                ))
                .into_response();
            }
        };
        match outcome {
            Ok(_) => successes += 1,
            Err(e) => {
                failures += 1;
                log::warn!("Batch action {action_slug} failed for {id}: {e}");
            }
        }
    }
    let msg = format!(
        "Batch action '{action_slug}' executed on {} instances: {successes} succeeded, {failures} failed",
        node_ids.len()
    );
    Redirect::to(&format!("/nodes?flash={}", html::urlencoding_lite(&msg))).into_response()
}

/// Stop — or resume — automatic restart for one node.
///
/// There is one workspace watchdog policy, read for every node, so the only way
/// to stop the watchdog relaunching the node an operator was editing was to
/// disable automatic restart for the whole fleet. That leaves every other node
/// unsupervised, and it has to be remembered and undone on a page the node does
/// not link to.
pub async fn toggle_restart_hold(
    State(state): State<WebState>,
    Path(node_id): Path<String>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let node = load_node(&state.workspace, &node_id)?;
        let held = state.workspace.node_restart_hold(&node.id)?.is_some();
        if held {
            state.commands.release_node_restarts(&node.id)?;
            journal_hold(
                &state,
                &node,
                format!("automatic restart resumed for {}", node.name),
            );
            Ok(format!(
                "{} follows the workspace restart policy again",
                node.name
            ))
        } else {
            state.commands.hold_node_restarts(
                &node.id,
                "held from the node page",
                crate::web::time::now_unix(),
            )?;
            journal_hold(
                &state,
                &node,
                format!(
                    "automatic restart held for {}; the rest of the fleet is unaffected",
                    node.name
                ),
            );
            Ok(format!("{} will not be restarted automatically", node.name))
        }
    })();
    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("restart hold not changed: {error:#}"),
    };
    back_to_node(&node_id, &message)
}

fn journal_hold(state: &WebState, node: &crate::types::NodeConfig, message: String) {
    let _ = state.commands.record_event(crate::events::NewRuntimeEvent {
        node_id: Some(node.id.clone()),
        node_name: Some(node.name.clone()),
        kind: crate::events::EventKind::WatchdogPolicyUpdated,
        severity: crate::events::EventSeverity::Info,
        message,
    });
}
