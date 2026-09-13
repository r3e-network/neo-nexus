//! Node list page, filtering, density handling, and inventory tables.

use axum::{
    extract::{Query, RawQuery, State},
    response::{Html, IntoResponse, Response},
};

use crate::{
    core::node::{filter_nodes, NodeConfig, NodeInventoryFilter, NodeStatus},
    web::{assets::DensityMode, fleet::Fleet, html, WebState},
};

#[derive(Default, serde::Deserialize)]
pub struct NodeListQuery {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub q: String,
}

pub async fn node_list(
    State(state): State<WebState>,
    RawQuery(flash): RawQuery,
    Query(params): Query<NodeListQuery>,
) -> Response {
    let density = resolve_density(&state);
    let body = match Fleet::load(&state.workspace) {
        Ok(fleet) => list_body(&state, &fleet, &params, density),
        Err(error) => html::note(&format!("failed to load nodes: {error}")),
    };
    Html(html::layout_with_density(
        "Nodes",
        "nodes",
        &html::flash(flash.as_deref()),
        &body,
        density,
    ))
    .into_response()
}

/// Resolve the stored UI density preference, falling back to comfortable when
/// it is unset or the settings read fails, so the list still renders.
pub fn resolve_density(state: &WebState) -> DensityMode {
    state
        .workspace
        .load_app_ui_density()
        .ok()
        .flatten()
        .as_deref()
        .map_or(DensityMode::DEFAULT, DensityMode::from_str)
}

fn list_body(
    state: &WebState,
    fleet: &Fleet,
    params: &NodeListQuery,
    density: DensityMode,
) -> String {
    let breadcrumb = html::breadcrumb(&[("EC2", "/nodes"), ("Instances", "")]);
    let head = html::page_head(
        "Instances",
        "AWS EC2-style sovereign virtual node instances, consensus health status checks, and fleet orchestration.",
        &add_button(),
    );
    if fleet.rows.is_empty() {
        return format!(
            "{breadcrumb}\n{head}\n{}",
            html::empty_state(
                "No instances yet",
                "Launch a node instance to give the workbench something to configure, launch and monitor.",
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
    let role_query = params.role.trim();
    let visible = if role_query.is_empty() {
        visible
    } else {
        visible
            .into_iter()
            .filter(|node| {
                state
                    .workspace
                    .load_node_role(&node.id)
                    .ok()
                    .flatten()
                    .is_some_and(|r| r.slug() == role_query || r.persist_key() == role_query)
            })
            .collect::<Vec<_>>()
    };
    let filters = html::typed_filter_form(
        "/nodes",
        &[],
        &[
            html::FilterControl::Select {
                label: "Status",
                name: "status",
                selected: &params.status,
                options: &[
                    ("", "All statuses"),
                    ("running", "Running"),
                    ("starting", "Starting"),
                    ("stopped", "Stopped"),
                    ("error", "Error"),
                ],
            },
            html::FilterControl::Select {
                label: "Role",
                name: "role",
                selected: &params.role,
                options: &[
                    ("", "All roles"),
                    ("validator", "⚡ Validator"),
                    ("rpc-api", "🌐 RPC Gateway"),
                    ("indexer", "📊 Indexer"),
                    ("oracle", "🔮 Oracle"),
                    ("observer", "👁️ Observer"),
                ],
            },
            html::FilterControl::Search {
                label: "Search",
                name: "q",
                value: &params.q,
                placeholder: "Name, id, client, or network",
            },
        ],
    );
    let table = if visible.is_empty() {
        html::note("No node matches this filter.")
    } else if density == DensityMode::Compact {
        manager_table_compact(state, fleet, &visible)
    } else {
        manager_table(state, fleet, &visible)
    };
    let drawer = if visible.is_empty() {
        String::new()
    } else {
        ec2_instance_drawer(state, &visible)
    };
    format!(
        "{breadcrumb}\n{head}\n{}\n{filters}\n{table}\n{drawer}",
        status_tiles(state, fleet),
        table = table,
        drawer = drawer,
    )
}

fn ec2_instance_drawer(state: &WebState, visible: &[NodeConfig]) -> String {
    let first = match visible.first() {
        Some(node) => node,
        None => return String::new(),
    };
    let id = html::urlencoding_lite(&first.id);
    let role = state.workspace.load_node_role(&first.id).ok().flatten();
    let role_label = role.map_or("Node", |r| r.label());
    let state_badge = html::status_badge(first.status.label());
    let signer = state
        .workspace
        .list_all_signer_bindings()
        .ok()
        .and_then(|all| {
            all.into_iter()
                .find(|(nid, _)| nid == &first.id)
                .map(|(_, k)| k)
        });
    let signer_arn = signer.map_or_else(
        || "Unbound".to_string(),
        |k| format!("arn:neo:kms:mesh-1a:key/{}", k.key_id),
    );

    format!(
        r#"<div class="aws-detail-drawer" id="ec2-instance-drawer">
            <div style="display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--line); padding-bottom: 12px; margin-bottom: 14px; flex-wrap: wrap; gap: 8px;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <span class="aws-pulse-dot"></span>
                    <span style="font-weight: 700; font-size: 14px; color: #fff;">Instance: <span data-drawer-name>{}</span></span>
                    <span class="muted mono" style="font-size: 12px;" data-drawer-id>(i-{})</span>
                    <span data-drawer-status>{}</span>
                </div>
                <div style="display: flex; align-items: center; gap: 8px;">
                    <a class="btn small primary" data-drawer-studio-link href="/nodes/{}">Open EC2 Studio ↗</a>
                    <a class="btn small" data-drawer-log-link href="/logs?node={}">View System Log</a>
                </div>
            </div>
            <div class="aws-drawer-grid">
                <div><span class="muted">Instance Type:</span> <strong style="color: #fff;" data-drawer-type>t3.{}</strong></div>
                <div><span class="muted">Platform / AMI:</span> <span class="badge" data-drawer-ami>{}</span></div>
                <div><span class="muted">Network / Chain:</span> <span class="badge" data-drawer-net>{}</span></div>
                <div><span class="muted">Availability Zone:</span> <strong style="color: #fff;">nexus-az-1a</strong></div>
                <div><span class="muted">RPC Endpoint:</span> <span class="mono" data-drawer-rpc>:{}</span></div>
                <div><span class="muted">P2P Port:</span> <span class="mono" data-drawer-p2p>:{}</span></div>
                <div><span class="muted">IAM Signer Role:</span> <span class="mono muted" style="font-size: 11px;" data-drawer-signer>{}</span></div>
                <div><span class="muted">Consensus Role:</span> <strong style="color: var(--jade);" data-drawer-role>{}</strong></div>
            </div>
        </div>"#,
        html::escape(&first.name),
        html::escape(&first.id),
        state_badge,
        id,
        id,
        html::escape(&first.node_type.to_string()),
        html::escape(&first.runtime_version),
        html::escape(&first.network.to_string()),
        first.rpc_port,
        first.p2p_port,
        html::escape(&signer_arn),
        html::escape(role_label),
    )
}

fn status_tiles(state: &WebState, fleet: &Fleet) -> String {
    let counts = fleet.count_by_status();
    let all_hermes = state.workspace.list_hermes_agents().unwrap_or_default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let hermes_active = all_hermes.iter().filter(|h| h.is_alive(now)).count();
    let validators = fleet
        .rows
        .iter()
        .filter(|r| {
            state.workspace.load_node_role(&r.node.id).ok().flatten()
                == Some(crate::roles::NodeRole::Consensus)
        })
        .count();

    html::cards(&[
        ("Total Instances", counts.total.to_string()),
        ("Running Instances", counts.running.to_string()),
        ("Consensus Nodes", validators.to_string()),
        ("Hermes Copilots", format!("{hermes_active} active")),
    ])
}

fn add_button() -> String {
    r#"<a class="btn" href="/api/fleet/iac?format=cloudformation" download="fleet-cloudformation.yaml" title="Export entire fleet as AWS CloudFormation manifest">☁️ Export CloudFormation</a> <a class="btn" href="/api/fleet/iac?format=terraform" download="fleet-main.tf" title="Export entire fleet as Terraform manifest">🏗️ Export Terraform</a> <a class="btn" href="/api/fleet/iac?format=compose" download="docker-compose.yml" title="Export entire fleet as Docker Compose manifest">🐳 Export Compose</a> <a class="btn" href="/api/fleet/iac?format=k8s" download="k8s-fleet.yaml" title="Export entire fleet as Kubernetes Pod manifest">☸️ Export K8s Fleet</a> <a class="btn" href="/api/fleet" download="fleet-inventory.json" title="Export complete cloud fleet inventory as JSON">📥 Fleet JSON</a> <a class="btn primary" href="/nodes/new">+ Launch Instance</a>"#.to_string()
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

fn manager_table(state: &WebState, fleet: &Fleet, visible: &[NodeConfig]) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let all_signers = state
        .workspace
        .list_all_signer_bindings()
        .unwrap_or_default();
    let all_hermes = state.workspace.list_hermes_agents().unwrap_or_default();

    let rows = visible
        .iter()
        .filter_map(|node| fleet.rows.iter().find(|row| row.node.id == node.id))
        .map(|row| {
            let id = html::urlencoding_lite(&row.node.id);
            let role = state.workspace.load_node_role(&row.node.id).ok().flatten();
            let signer = all_signers.iter().find(|(nid, _)| nid == &row.node.id).map(|(_, k)| k);
            let hermes = all_hermes.iter().find(|a| a.node_id == row.node.id);

            // This read "2/2 passed" from `is_running()` alone. There are no
            // two checks: the process state and the RPC health verdict are
            // separate facts, the second of which may never have been taken.
            // The RPC column beside this one carries that verdict already.
            let status_check = html::status_badge(row.node.status.label());

            let role_badge = match role {
                Some(crate::roles::NodeRole::Consensus) => "<span class=\"badge\">⚡ Validator</span>".to_string(),
                Some(crate::roles::NodeRole::RpcApi) => "<span class=\"badge\">🌐 RPC Gateway</span>".to_string(),
                Some(crate::roles::NodeRole::Indexer) => "<span class=\"badge\">📊 Indexer</span>".to_string(),
                Some(crate::roles::NodeRole::Oracle) => "<span class=\"badge\">🔮 Oracle</span>".to_string(),
                Some(crate::roles::NodeRole::Observer) => "<span class=\"badge\">👁️ Observer</span>".to_string(),
                Some(other) => format!("<span class=\"badge\">{}</span>", html::escape(other.label())),
                None => "<span class=\"badge\">Node</span>".to_string(),
            };

            let signer_badge = match signer {
                Some(key) => format!(
                    r#"<span class="badge" title="{backend}/{key_id}">🔒 {backend}</span>"#,
                    backend = html::escape(&key.backend_id),
                    key_id = html::escape(&key.key_id),
                ),
                None => r#"<span class="muted" style="font-size: 12px;">Unbound</span>"#.to_string(),
            };

            let hermes_badge = match hermes {
                Some(h) if h.is_alive(now) => r#"<span class="badge running">🤖 Active</span>"#.to_string(),
                Some(h) if h.enabled => r#"<span class="badge stopped">🤖 Ready</span>"#.to_string(),
                _ => r#"<span class="muted" style="font-size: 12px;">—</span>"#.to_string(),
            };

            let ports_cell = if row.node.rpc_port == 0 {
                format!(r#"<span class="num">:{}</span> <span class="badge">RPC Off</span>"#, row.node.p2p_port)
            } else {
                format!(r#"<span class="num">:{}</span> <span class="muted">/</span> <span class="num">:{}</span>"#, row.node.p2p_port, row.node.rpc_port)
            };

            let quick_power = if row.node.status.is_running() {
                format!(
                    r#"<form method="post" action="/nodes/{id}/restart" style="display:inline;"><button class="btn small" type="submit" title="Restart Node Instance">🔄</button></form>
<form method="post" action="/nodes/{id}/stop" style="display:inline; margin-left:2px;"><button class="btn small danger" type="submit" title="Stop Node Instance">⏹</button></form>"#
                )
            } else {
                format!(
                    r#"<form method="post" action="/nodes/{id}/start" style="display:inline;"><button class="btn small primary" type="submit" title="Start Node Instance">▶</button></form>"#
                )
            };

            let actions = format!(
                r#"<div class="row-actions">{quick_power}<a class="btn small" href="/nodes/{id}">Studio</a><a class="btn small" href="/nodes/{id}/edit">Edit</a><a class="btn small danger" href="/nodes/{id}/delete">Delete</a></div>"#
            );

            let instance_cell = format!(
                r#"<div><a href="/nodes/{id}" style="font-weight: 600;">{name}</a></div><div class="muted mono" style="font-size: 11px;">{raw_id}</div>"#,
                name = html::escape(&row.node.name),
                raw_id = html::escape(&row.node.id),
            );

            let select_cell = format!(r#"<input type="checkbox" name="node_ids" value="{}" style="cursor: pointer;">"#, html::escape(&row.node.id));

            let cells = [
                html::raw_cell(&select_cell),
                html::raw_cell(&instance_cell),
                html::raw_cell(&html::status_badge(row.node.status.label())),
                html::raw_cell(&status_check),
                html::cell("nexus-az-1a"),
                html::raw_cell(&role_badge),
                html::raw_cell(&format!(r#"<span class="badge">{}</span> <span class="badge">{}</span>"#, html::escape(&row.node.node_type.to_string()), html::escape(&row.node.network.to_string()))),
                html::raw_cell(&ports_cell),
                html::raw_cell(&signer_badge),
                html::raw_cell(&hermes_badge),
                html::raw_cell(&actions),
            ];
            let signer_arn_attr = signer.map_or_else(|| "Unbound".to_string(), |k| format!("arn:neo:kms:mesh-1a:key/{}", k.key_id));
            let role_str = role.map_or("Node", |r| r.label());
            format!(
                r#"<tr data-node-id="{raw_id}" data-node-name="{name}" data-node-type="{node_type}" data-node-net="{net}" data-node-ver="{ver}" data-node-rpc-port="{rpc}" data-node-p2p-port="{p2p}" data-node-role="{role}" data-node-signer="{signer}" data-node-status="{status}">{}</tr>"#,
                cells.join(""),
                raw_id = html::escape(&row.node.id),
                name = html::escape(&row.node.name),
                node_type = html::escape(&row.node.node_type.to_string()),
                net = html::escape(&row.node.network.to_string()),
                ver = html::escape(&row.node.runtime_version),
                rpc = row.node.rpc_port,
                p2p = row.node.p2p_port,
                role = html::escape(role_str),
                signer = html::escape(&signer_arn_attr),
                status = html::escape(row.node.status.label()),
            )
        })
        .collect::<Vec<_>>();

    let batch_bar = r#"<div class="batch-bar aws-action-bar" style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding: 10px 14px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 6px; flex-wrap: wrap; gap: 8px;">
        <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
            <button type="button" class="btn small" data-action="toggle-all-nodes">☑ Toggle All</button>
            <span style="font-size: 12px; font-weight: 600; margin-left: 4px; text-transform: uppercase; color: var(--muted);">Instance State:</span>
            <button type="submit" name="action" value="start" class="btn small primary" title="Start all selected instances">▶ Start</button>
            <button type="submit" name="action" value="restart" class="btn small" title="Restart all selected instances">🔄 Reboot</button>
            <button type="submit" name="action" value="stop" class="btn small danger" title="Stop all selected instances">⏹ Stop</button>
            <span style="font-size: 12px; font-weight: 600; margin-left: 8px; text-transform: uppercase; color: var(--muted);">Actions:</span>
            <button type="submit" name="action" value="smoke" class="btn small" title="Run supervised smoke test across all selected instances">🩺 Run Diagnostics Sweep</button>
        </div>
        <div class="muted" style="font-size: 11px;">
            Select instances to dispatch fleet orchestration commands
        </div>
    </div>"#;

    let table = html::table(
        &[
            "Select",
            "Instance",
            "State",
            "Status Check",
            "Availability Zone",
            "Role",
            "Client / Net",
            "Ports (P2P/RPC)",
            "IAM Signer",
            "Hermes AI",
            "Actions",
        ],
        &rows,
    );

    format!(
        r#"<form method="post" action="/nodes/batch-action" id="fleet-batch-form">{batch_bar}{table}</form>"#
    )
}

fn manager_table_compact(state: &WebState, fleet: &Fleet, visible: &[NodeConfig]) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let all_hermes = state.workspace.list_hermes_agents().unwrap_or_default();

    let rows = visible
        .iter()
        .filter_map(|node| fleet.rows.iter().find(|row| row.node.id == node.id))
        .map(|row| {
            let id = html::urlencoding_lite(&row.node.id);
            let hermes = all_hermes.iter().find(|a| a.node_id == row.node.id);
            let hermes_badge = if hermes.is_some_and(|h| h.is_alive(now)) {
                r#"<span class="badge running">🤖 Active</span>"#
            } else {
                ""
            };
            let quick_power = if row.node.status.is_running() {
                format!(
                    r#"<form method="post" action="/nodes/{id}/restart" style="display:inline;"><button class="btn small" type="submit" title="Restart">🔄</button></form>"#
                )
            } else {
                format!(
                    r#"<form method="post" action="/nodes/{id}/start" style="display:inline;"><button class="btn small primary" type="submit" title="Start">▶</button></form>"#
                )
            };
            let actions = format!(
                r#"<div class="row-actions">{quick_power}<a class="btn small" href="/nodes/{id}">View</a><a class="btn small" href="/nodes/{id}/edit">Edit</a><a class="btn small danger" href="/nodes/{id}/delete">Delete</a></div>"#
            );
            let rpc_text = if row.node.rpc_port == 0 {
                "RPC off".to_string()
            } else {
                format!("RPC {}", row.node.rpc_port)
            };
            let line = format!(
                r#"<div class="node-line">{dot}<a class="node-name" href="/nodes/{id}">{name}</a><span class="badge">{node_type}</span><span class="badge">{network}</span><span class="num node-port">{rpc}</span>{hermes}{pill}</div>"#,
                dot = html::status_dot(row.node.status.label()),
                name = html::escape(&row.node.name),
                node_type = html::escape(&row.node.node_type.to_string()),
                network = html::escape(&row.node.network.to_string()),
                rpc = html::escape(&rpc_text),
                hermes = hermes_badge,
                pill = html::status_badge(row.node.status.label()),
            );
            html::row(&[html::raw_cell(&line), html::raw_cell(&actions)])
        })
        .collect::<Vec<_>>();
    html::table(&["Node", "Actions"], &rows)
}
