//! Nodes: the fleet list, the per-node studio, and the delete confirmation.
//!
//! The list is the manager's front door, so it carries the actions an operator
//! came for — add, edit, delete — instead of only linking onward. Lifecycle
//! controls post to the same core pipeline the CLI drives, and deletion is a
//! two-step flow because nothing here can undo it.

use axum::{
    extract::{Path, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    core::{
        node::{filter_nodes, NodeConfig, NodeInventoryFilter, NodeStatus},
        node_health::node_rpc_health_history,
        operations::{EventKind, EventSeverity, NewRuntimeEvent},
    },
    web::{fleet::Fleet, html, time, WebState},
};

#[derive(Default, serde::Deserialize)]
pub struct NodeListQuery {
    #[serde(default)]
    status: String,
    #[serde(default)]
    q: String,
}

pub async fn node_list(
    State(state): State<WebState>,
    RawQuery(flash): RawQuery,
    Query(params): Query<NodeListQuery>,
) -> Response {
    let body = match Fleet::load(&state.repository) {
        Ok(fleet) => list_body(&fleet, &params),
        Err(error) => html::note(&format!("failed to load nodes: {error}")),
    };
    Html(html::layout(
        "Nodes",
        "nodes",
        &html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

fn list_body(fleet: &Fleet, params: &NodeListQuery) -> String {
    let head = html::page_head(
        "Nodes",
        "Every node this workspace can configure, launch and watch.",
        &add_button(),
    );
    if fleet.rows.is_empty() {
        return format!(
            "{head}\n{}",
            html::empty_state(
                "No nodes yet",
                "Add a node to give the workbench something to configure, launch and monitor.",
                &add_button(),
            )
        );
    }

    let all = fleet
        .rows
        .iter()
        .map(|row| row.node.clone())
        .collect::<Vec<_>>();
    let visible = filter_nodes(
        &all,
        &NodeInventoryFilter::new(status_filter(&params.status), params.q.trim()),
    );
    let filters = html::filter_form("/nodes", &[("status", &params.status), ("q", &params.q)]);
    let table = if visible.is_empty() {
        html::note("No node matches this filter.")
    } else {
        manager_table(fleet, &visible)
    };
    format!(
        "{head}\n{}\n{filters}\n{table}",
        status_tiles(fleet),
        table = table,
    )
}

/// Counts come from the fleet itself. Reading host pressure too would mean a
/// full system scan on a page whose subject is the nodes.
fn status_tiles(fleet: &Fleet) -> String {
    let counts = fleet.count_by_status();
    html::cards(&[
        ("Nodes", counts.total.to_string()),
        ("Running", counts.running.to_string()),
        ("Stopped", counts.stopped.to_string()),
        ("Error", counts.error.to_string()),
    ])
}

fn add_button() -> String {
    r#"<a class="btn primary" href="/nodes/new">Add node</a>"#.to_string()
}

fn status_filter(raw: &str) -> Option<NodeStatus> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "running" => Some(NodeStatus::Running),
        "starting" => Some(NodeStatus::Starting),
        "stopped" => Some(NodeStatus::Stopped),
        "error" => Some(NodeStatus::Error),
        _ => None,
    }
}

fn manager_table(fleet: &Fleet, visible: &[NodeConfig]) -> String {
    let rows = visible
        .iter()
        .filter_map(|node| fleet.rows.iter().find(|row| row.node.id == node.id))
        .map(|row| {
            let id = html::urlencoding_lite(&row.node.id);
            let actions = format!(
                r#"<div class="row-actions"><a class="btn small" href="/nodes/{id}">View</a><a class="btn small" href="/nodes/{id}/edit">Edit</a><a class="btn small danger" href="/nodes/{id}/delete">Delete</a></div>"#
            );
            html::row(&[
                html::raw_cell(&format!(
                    r#"<a href="/nodes/{id}">{}</a>"#,
                    html::escape(&row.node.name)
                )),
                html::cell(&row.node.node_type.to_string()),
                html::cell(&row.node.network.to_string()),
                html::raw_cell(&html::status_badge(row.node.status.label())),
                html::cell(&row.node.rpc_port.to_string()),
                html::raw_cell(&format!(
                    r#"<span data-node-rpc>{}</span>"#,
                    html::escape(&row.rpc_health)
                )),
                html::raw_cell(&actions),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Name", "Client", "Network", "Status", "RPC", "Health", "Actions",
        ],
        &rows,
    )
}

pub async fn node_detail(
    State(state): State<WebState>,
    Path(id): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let body = match render_detail(&state, &id) {
        Ok(body) => body,
        Err(_) => return Redirect::to("/nodes").into_response(),
    };
    Html(html::layout(
        "Node",
        "nodes",
        &html::flash(query.as_deref()),
        &body,
    ))
    .into_response()
}

fn render_detail(state: &WebState, id: &str) -> anyhow::Result<String> {
    let fleet = Fleet::load(&state.repository)?;
    let row = fleet
        .rows
        .iter()
        .find(|row| row.node.id == id)
        .ok_or_else(|| anyhow::anyhow!("node {id} was not found"))?;
    let node = &row.node;
    let history = node_rpc_health_history(&state.repository, &node.id, 10)?;
    let plugins = state.repository.list_plugin_states(&node.id)?;
    let encoded = html::urlencoding_lite(id);
    let command = crate::argv::format_command(&node.binary_path, &node.args);

    let trend = history
        .iter()
        .map(|record| {
            html::row(&[
                html::raw_cell(&time::time_cell(Some(record.checked_at_unix))),
                html::cell(record.status.label()),
                html::cell(
                    &record
                        .block_count
                        .map_or_else(|| "—".to_string(), |block| block.to_string()),
                ),
                html::cell(&record.message),
            ])
        })
        .collect::<Vec<_>>();

    let header_actions = format!(
        r#"<a class="btn" href="/nodes/{encoded}/edit">Edit</a><a class="btn" href="/logs?node={encoded}">Logs</a><a class="btn" href="/plugins?node={encoded}">Plugins</a>"#
    );

    // Running is not the same as supervised: after a workbench restart, or when
    // the CLI started the node, the row is true and the handle is not.
    let supervision = if node.status.is_running() && !state.is_supervised(&node.id) {
        html::notice(
            "warn",
            "Running, but not supervised by this workbench process. Stop can still reach it by pid; the watchdog here will not restart it.",
        )
    } else {
        String::new()
    };
    let config = format!(
        "<h2>Configuration</h2>\n{facts}\n{supervision}<h2>Launch command</h2>\n{command}",
        facts = fact_rows(node),
        command = html::text_block(&command),
    );
    let runtime = format!(
        "<h2>Plugins</h2>\n{plugins}\n<h2>RPC health history</h2>\n{trend}",
        plugins = plugin_summary(&plugins),
        trend = if trend.is_empty() {
            html::note("No RPC probes recorded yet.")
        } else {
            html::table(&["Checked", "Status", "Block", "Message"], &trend)
        },
    );

    Ok(format!(
        r#"{breadcrumb}
{head}
{controls}
<div class="grid">
<div>{config}</div>
<div>{runtime}</div>
</div>"#,
        breadcrumb = html::breadcrumb(&[("Nodes", "/nodes"), (&node.name, "")]),
        head = html::page_head(
            &node.name,
            &format!(
                "{} on {} · {}",
                node.node_type,
                node.network,
                node.status.label()
            ),
            &header_actions,
        ),
        controls = control_bar(id, node.status.label()),
        config = config,
        runtime = runtime,
    ))
}

fn plugin_summary(plugins: &[crate::catalog::PluginState]) -> String {
    if plugins.is_empty() {
        return html::note("No plugin state recorded for this node.");
    }
    let rows = plugins
        .iter()
        .map(|plugin| {
            let state = if plugin.enabled {
                r#"<span class="badge running">enabled</span>"#
            } else {
                r#"<span class="badge stopped">off</span>"#
            };
            html::row(&[
                html::cell(&plugin.plugin_id.to_string()),
                html::raw_cell(state),
            ])
        })
        .collect::<Vec<_>>();
    html::table(&["Plugin", "State"], &rows)
}

fn control_bar(node_id: &str, status: &str) -> String {
    let running = matches!(status, "Running" | "Starting");
    let encoded = html::urlencoding_lite(node_id);
    let disabled = if running { "" } else { " disabled" };
    format!(
        r#"<div class="actions" style="margin-bottom:20px">
<form method="post" action="/nodes/{encoded}/start"><button class="primary" type="submit">Start</button></form>
<form method="post" action="/nodes/{encoded}/stop"><button type="submit"{disabled}>Stop</button></form>
<form method="post" action="/nodes/{encoded}/restart"><button type="submit"{disabled}>Restart</button></form>
</div>"#,
        disabled = disabled,
    )
}

fn fact_rows(node: &NodeConfig) -> String {
    let facts = [
        ("Client", node.node_type.to_string()),
        ("Network", node.network.to_string()),
        ("Storage", node.node_type.storage_label(node.storage_engine)),
        ("Binary", node.binary_path.display().to_string()),
        ("Runtime", node.runtime_version.clone()),
        ("RPC port", node.rpc_port.to_string()),
        ("P2P port", node.p2p_port.to_string()),
        (
            "WebSocket port",
            node.ws_port
                .map_or_else(|| "none".to_string(), |port| port.to_string()),
        ),
        (
            "Process",
            node.pid
                .map_or_else(|| "not running".to_string(), |pid| pid.to_string()),
        ),
        ("Node id", node.id.clone()),
    ];
    let rows = facts
        .iter()
        .map(|(label, value)| html::row(&[html::cell(label), html::cell(value)]))
        .collect::<Vec<_>>();
    html::table(&["Setting", "Value"], &rows)
}

/// The confirmation step. Deleting drops plugin state, managed installs and RPC
/// health history, so the operator reads that before agreeing rather than
/// discovering it afterwards.
pub async fn delete_form(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let node = match state.repository.list_nodes() {
        Ok(nodes) => nodes.into_iter().find(|node| node.id == id),
        Err(_) => None,
    };
    let Some(node) = node else {
        return Redirect::to("/nodes").into_response();
    };
    let encoded = html::urlencoding_lite(&node.id);
    let detail = "Also removed: plugin state, managed plugin installs and RPC health history \
                  for this node. Nothing outside the workspace database is touched — the node's \
                  own files and chain data stay on disk.";
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
    let name = state
        .repository
        .list_nodes()
        .ok()
        .and_then(|nodes| {
            nodes
                .into_iter()
                .find(|node| node.id == id)
                .map(|node| node.name)
        })
        .unwrap_or_else(|| id.clone());

    // Journal first: once the row is gone the event could not name the node,
    // and an audit trail that cannot say what was removed is not a trail.
    let outcome = (|| -> anyhow::Result<()> {
        state.repository.record_event(NewRuntimeEvent {
            node_id: Some(id.clone()),
            node_name: Some(name.clone()),
            kind: EventKind::NodeDeleted,
            severity: EventSeverity::Warning,
            message: format!("{name} deleted"),
        })?;
        state.repository.delete_node(&id)
    })();

    let message = match outcome {
        Ok(()) => format!("{name} deleted."),
        Err(error) => format!("delete failed: {error}"),
    };
    Redirect::to(&format!(
        "/nodes?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}
