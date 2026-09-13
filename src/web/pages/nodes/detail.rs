//! Node detail studio view: configuration facts, health history, plugins, and control bar.

use axum::{
    extract::{Path, RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    core::node::NodeConfig,
    web::{fleet::Fleet, html, WebState},
};

use super::list::resolve_density;

pub async fn node_detail(
    State(state): State<WebState>,
    Path(id): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let density = resolve_density(&state);
    let new_token = query.as_deref().and_then(|q| {
        q.split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(k, _)| *k == "hermes_token")
            .map(|(_, v)| v)
    });
    let body = match render_detail(&state, &id, new_token) {
        Ok(body) => body,
        Err(err) => {
            eprintln!("Error rendering node detail for '{id}': {err:?}");
            return Redirect::to(&format!(
                "/nodes?flash={}",
                html::urlencoding_lite(&format!("Node error: {err}"))
            ))
            .into_response();
        }
    };
    Html(html::layout_with_density(
        "Node",
        "nodes",
        &html::flash(query.as_deref()),
        &body,
        density,
    ))
    .into_response()
}

pub async fn toggle_hermes_healing(
    State(state): State<WebState>,
    Path(id): Path<String>,
) -> Response {
    let mut assoc = state
        .workspace
        .load_hermes_agent(&id)
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::agents::HermesAgentAssociation::new(&id));
    assoc.autonomous_healing = !assoc.autonomous_healing;
    let label = if assoc.autonomous_healing { "enabled" } else { "disabled" };
    match state.commands.save_hermes_agent(&assoc) {
        Ok(()) => {
            let _ = state.commands.record_event(crate::core::operations::NewRuntimeEvent {
                node_id: Some(id.clone()),
                node_name: None,
                kind: crate::core::operations::EventKind::NodeUpdated,
                severity: crate::core::operations::EventSeverity::Info,
                message: format!("Hermes autonomous healing {label} for node {id}"),
            });
            Redirect::to(&format!(
                "/nodes/{}?flash=Autonomous%20self-healing%20{}",
                html::urlencoding_lite(&id),
                html::urlencoding_lite(label),
            ))
            .into_response()
        }
        Err(e) => Redirect::to(&format!(
            "/nodes/{}?flash=Failed%20to%20update%20healing%20state%3A%20{}",
            html::urlencoding_lite(&id),
            html::urlencoding_lite(&e.to_string()),
        ))
        .into_response(),
    }
}

pub async fn test_hermes_ping(
    State(state): State<WebState>,
    Path(id): Path<String>,
) -> Response {
    match state.commands.record_hermes_heartbeat(&id, Some("0.5.0-copilot")) {
        Ok(()) => Redirect::to(&format!(
            "/nodes/{}?flash=Hermes%20guest%20heartbeat%20ping%20recorded",
            html::urlencoding_lite(&id),
        ))
        .into_response(),
        Err(e) => Redirect::to(&format!(
            "/nodes/{}?flash=Failed%20to%20record%20heartbeat%3A%20{}",
            html::urlencoding_lite(&id),
            html::urlencoding_lite(&e.to_string()),
        ))
        .into_response(),
    }
}

pub async fn provision_hermes_token(
    State(state): State<WebState>,
    Path(id): Path<String>,
) -> Response {
    let node = match state
        .workspace
        .list_nodes()
        .ok()
        .and_then(|nodes| nodes.into_iter().find(|n| n.id == id))
    {
        Some(n) => n,
        None => return Redirect::to("/nodes").into_response(),
    };
    let token_name = format!("hermes-agent-{}", node.name);
    let permissions = vec![crate::wallet::TokenPermission::HermesAgent(node.id.clone())];
    match state.commands.create_api_token(&token_name, permissions, None) {
        Ok((_token, secret)) => {
            let _ = state.commands.record_event(crate::core::operations::NewRuntimeEvent {
                node_id: Some(node.id.clone()),
                node_name: Some(node.name.clone()),
                kind: crate::core::operations::EventKind::ApiTokenCreated,
                severity: crate::core::operations::EventSeverity::Info,
                message: format!("Provisioned scoped Hermes Agent token for {}", node.name),
            });
            Redirect::to(&format!(
                "/nodes/{}?hermes_token={}&flash=Hermes%20Agent%20scoped%20token%20provisioned",
                html::urlencoding_lite(&id),
                html::urlencoding_lite(&secret),
            ))
            .into_response()
        }
        Err(e) => Redirect::to(&format!(
            "/nodes/{}?flash=Failed%20to%20provision%20token%3A%20{}",
            html::urlencoding_lite(&id),
            html::urlencoding_lite(&e.to_string()),
        ))
        .into_response(),
    }
}

fn render_detail(state: &WebState, id: &str, new_token: Option<&str>) -> anyhow::Result<String> {
    let fleet = Fleet::load(&state.workspace)?;
    let row = fleet
        .rows
        .iter()
        .find(|row| row.node.id == id)
        .ok_or_else(|| anyhow::anyhow!("node {id} was not found"))?;
    let node = &row.node;
    let history = state.workspace.node_rpc_health_history(&node.id, 10)?;
    let signer = state.workspace.load_node_signer_key(&node.id)?;
    let encoded = html::urlencoding_lite(id);

    let header_actions = format!(
        r#"<form method="post" action="/nodes/{encoded}/smoke" style="display:inline;"><button class="btn" type="submit" title="Run supervised binary & RPC smoke test">🩺 SRE Health Sweep</button></form><a class="btn" href="/nodes/{encoded}/edit">Edit</a><a class="btn" href="/logs?node={encoded}">Logs</a><a class="btn" href="/plugins?node={encoded}">Plugins</a>"#
    );

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let assoc = state
        .workspace
        .load_hermes_agent(&node.id)
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::agents::HermesAgentAssociation::new(&node.id));
    let role = state.workspace.load_node_role(&node.id).ok().flatten();

    let summary = super::detail_tabs::summary_banner(node, role, signer.as_ref());
    let tab1 = super::detail_tabs::render_tab_details(node);
    let tab2 = super::detail_tabs::render_tab_status_checks(node, role, signer.as_ref(), &history, &assoc, now);
    let tab3 = super::detail_tabs::render_tab_monitoring(state, node, &history);
    let tab4 = super::detail_tabs::render_tab_networking(state, node);
    let tab5 = super::detail_tabs::render_tab_security(state, node, signer.as_ref(), new_token);
    let tab6 = super::detail_tabs::render_tab_storage(node);
    let tab7 = super::detail_tabs::render_tab_tags(node, role);
    let tab8 = super::detail_tabs::render_tab_iac(node, role, signer.as_ref(), &assoc);
    let activity = super::activity::instance_activity_card(state, node);

    let tab_container = format!(
        r#"<div class="aws-tab-container" style="margin-top: 16px;">
            <div class="aws-tab-bar" role="tablist">
                <button type="button" class="aws-tab-btn active" data-tab-target="tab-details" role="tab" aria-selected="true">Details</button>
                <button type="button" class="aws-tab-btn" data-tab-target="tab-status" role="tab" aria-selected="false">Status checks</button>
                <button type="button" class="aws-tab-btn" data-tab-target="tab-monitoring" role="tab" aria-selected="false">Monitoring</button>
                <button type="button" class="aws-tab-btn" data-tab-target="tab-networking" role="tab" aria-selected="false">Networking</button>
                <button type="button" class="aws-tab-btn" data-tab-target="tab-security" role="tab" aria-selected="false">Security &amp; IAM</button>
                <button type="button" class="aws-tab-btn" data-tab-target="tab-storage" role="tab" aria-selected="false">Storage</button>
                <button type="button" class="aws-tab-btn" data-tab-target="tab-tags" role="tab" aria-selected="false">Tags</button>
                <button type="button" class="aws-tab-btn" data-tab-target="tab-iac" role="tab" aria-selected="false">Launch Template &amp; IaC</button>
            </div>
            <div class="aws-tab-panel active" data-tab-panel="tab-details" role="tabpanel">{tab1}</div>
            <div class="aws-tab-panel" data-tab-panel="tab-status" role="tabpanel" style="display: none;">{tab2}</div>
            <div class="aws-tab-panel" data-tab-panel="tab-monitoring" role="tabpanel" style="display: none;">{tab3}</div>
            <div class="aws-tab-panel" data-tab-panel="tab-networking" role="tabpanel" style="display: none;">{tab4}</div>
            <div class="aws-tab-panel" data-tab-panel="tab-security" role="tabpanel" style="display: none;">{tab5}</div>
            <div class="aws-tab-panel" data-tab-panel="tab-storage" role="tabpanel" style="display: none;">{tab6}</div>
            <div class="aws-tab-panel" data-tab-panel="tab-tags" role="tabpanel" style="display: none;">{tab7}</div>
            <div class="aws-tab-panel" data-tab-panel="tab-iac" role="tabpanel" style="display: none;">{tab8}</div>
        </div>"#,
        tab1 = tab1,
        tab2 = tab2,
        tab3 = tab3,
        tab4 = tab4,
        tab5 = tab5,
        tab6 = tab6,
        tab7 = tab7,
        tab8 = tab8,
    );

    Ok(format!(
        r#"{breadcrumb}
{head}
{controls}
{summary}
{tab_container}
{activity}"#,
        breadcrumb = html::breadcrumb(&[("EC2", "/nodes"), ("Instances", "/nodes"), (&node.name, "")]),
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
        controls = control_bar(node),
        summary = summary,
        tab_container = tab_container,
        activity = activity,
    ))
}

fn control_bar(node: &NodeConfig) -> String {
    let encoded = html::urlencoding_lite(&node.id);
    let start_disabled = if node.status.is_active() || node.pid.is_some() {
        " disabled"
    } else {
        ""
    };
    let stop_disabled = if node.status.is_active() || node.pid.is_some() {
        ""
    } else {
        " disabled"
    };
    let restart_disabled = if node.status.is_running() {
        ""
    } else {
        " disabled"
    };
    format!(
        r#"<div class="aws-action-bar" style="display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 6px; margin-bottom: 14px; flex-wrap: wrap; gap: 8px;">
            <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                <span class="muted" style="font-size: 11px; font-weight: 600; text-transform: uppercase;">Instance State:</span>
                <form method="post" action="/nodes/{encoded}/start" style="display: inline;">
                    <button type="submit" class="btn small primary"{start_disabled} title="Start virtual node instance">▶ Start</button>
                </form>
                <form method="post" action="/nodes/{encoded}/restart" style="display: inline;">
                    <button type="submit" class="btn small"{restart_disabled} title="Reboot virtual node instance">🔄 Reboot</button>
                </form>
                <form method="post" action="/nodes/{encoded}/stop" style="display: inline;">
                    <button type="submit" class="btn small danger"{stop_disabled} title="Quiesce virtual node instance process">⏹ Stop</button>
                </form>
                <span class="muted" style="font-size: 11px; font-weight: 600; text-transform: uppercase; margin-left: 8px;">Troubleshoot:</span>
                <form method="post" action="/nodes/{encoded}/smoke" style="display: inline;">
                    <button type="submit" class="btn small" title="Run supervised SRE smoke test">🩺 SRE Health Sweep</button>
                </form>
                <a class="btn small" href="/logs?node={encoded}" title="View live CloudWatch stream">📋 System Log</a>
            </div>
            <div style="display: flex; align-items: center; gap: 8px;">
                <a class="btn small" href="/nodes/{encoded}/edit" title="Edit instance configuration">⚙️ Edit</a>
                <a class="btn small danger" href="/nodes/{encoded}/delete" title="Terminate instance">🗑️ Terminate</a>
            </div>
        </div>"#,
        encoded = encoded,
        start_disabled = start_disabled,
        stop_disabled = stop_disabled,
        restart_disabled = restart_disabled,
    )
}

pub(crate) fn hermes_agent_card(state: &WebState, node: &NodeConfig, new_token: Option<&str>) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let assoc = state
        .workspace
        .load_hermes_agent(&node.id)
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::agents::HermesAgentAssociation::new(&node.id));

    let is_alive = assoc.is_alive(now);
    let status_badge = if is_alive {
        r#"<span class="badge running">🟢 Hermes Guest Agent Active</span>"#
    } else if assoc.last_heartbeat_unix.is_some() {
        r#"<span class="badge stopped">🟡 Hermes Stale (Last ping > 2m ago)</span>"#
    } else {
        r#"<span class="badge stopped">⚪ Hermes Ready to Connect</span>"#
    };

    let healing_badge = if assoc.autonomous_healing {
        r#"<span class="badge running">Autonomous Self-Healing: Active</span>"#
    } else {
        r#"<span class="badge stopped">Autonomous Self-Healing: Off</span>"#
    };

    let hour_ago = now.saturating_sub(3600);
    let recent_events = state
        .workspace
        .list_events(crate::events::RuntimeEventFilter::new(None, &node.name, 50))
        .unwrap_or_default();
    let recent_restarts = recent_events
        .iter()
        .filter(|e| {
            (e.occurred_at_unix as i64) >= hour_ago
                && (e.kind == crate::events::EventKind::NodeRestarted
                    || e.kind == crate::events::EventKind::NodeStartFailed)
                && (e.node_id.as_deref() == Some(node.id.as_str()) || e.message.contains(&node.id))
        })
        .count();

    let circuit_breaker_badge = if recent_restarts >= 5 {
        r#"<span class="badge stopped" title="Circuit breaker tripped: restart rate limit exceeded (5/5 in last hour)">🔴 Breaker Tripped (5/5)</span>"#
    } else {
        r#"<span class="badge running" title="Circuit breaker armed: autonomous recovery permitted">🟢 Breaker Armed</span>"#
    };

    let mcp_url = format!("http://127.0.0.1:8080/api/nodes/{}/mcp", node.id);
    let token_placeholder = new_token.unwrap_or("${NEONEXUS_AGENT_TOKEN}");
    let mut config_snippet = crate::agents::generate_hermes_config_snippet(&node.id, &node.name, &mcp_url);
    if let Some(secret) = new_token {
        config_snippet = config_snippet.replace("${NEONEXUS_AGENT_TOKEN}", secret);
    }

    let token_reveal = if let Some(secret) = new_token {
        format!(
            r#"<div class="notice" style="border-left: 3px solid #3584e4; background: rgba(53, 132, 228, 0.08); margin-bottom: 14px; padding: 12px 14px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                    <strong>🔑 Hermes Agent Scoped Token Provisioned</strong>
                    <span class="badge running">Instance-Scoped</span>
                </div>
                <p class="muted" style="margin: 4px 0 8px 0; font-size: 13px;">
                    Copy this token now. It has been pre-injected into the configuration snippet below and grants scoped MCP execution solely to <strong>{node_name}</strong>.
                </p>
                <div style="display: flex; gap: 8px;">
                    <input class="mono" readonly value="{secret}" style="flex: 1; padding: 6px 10px; background: var(--bg-card); border: 1px solid var(--line);" id="hermes-secret-box">
                </div>
            </div>"#,
            node_name = html::escape(&node.name),
            secret = html::escape(secret),
        )
    } else {
        String::new()
    };

    let encoded_id = html::urlencoding_lite(&node.id);
    let last_hb_text = match assoc.last_heartbeat_unix {
        Some(t) => {
            let diff = now.saturating_sub(t);
            if diff < 60 {
                format!("{diff}s ago")
            } else {
                format!("{}m ago", diff / 60)
            }
        }
        None => "Never (pending connection)".to_string(),
    };
    let healing_button = if assoc.autonomous_healing {
        format!(
            r#"<form method="post" action="/nodes/{encoded_id}/agent/toggle-healing" style="display: inline;">
                <button type="submit" class="btn small" title="Temporarily disable autonomous self-healing restarts">Pause Healing</button>
            </form>"#
        )
    } else {
        format!(
            r#"<form method="post" action="/nodes/{encoded_id}/agent/toggle-healing" style="display: inline;">
                <button type="submit" class="btn small primary" title="Enable autonomous recovery restarts">Enable Healing</button>
            </form>"#
        )
    };
    let ping_button = format!(
        r#"<form method="post" action="/nodes/{encoded_id}/agent/test-ping" style="display: inline;">
            <button type="submit" class="btn small" title="Simulate an agent heartbeat ping to verify active status">💓 Test Ping</button>
        </form>"#
    );

    let curl_example = format!(
        r#"curl -s -X POST http://127.0.0.1:8080/api/nodes/{}/mcp \
  -H "Authorization: Bearer {}" \
  -H "Content-Type: application/json" \
  -d '{{"jsonrpc":"2.0","id":1,"method":"tools/list"}}'"#,
        node.id, token_placeholder
    );

    format!(
        r#"<div class="panel hermes-agent-card">
            <div class="role-card-top" style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span style="font-size: 20px;">🤖</span>
                    <strong>Hermes AI Guest Agent</strong>
                </div>
                <div>{status_badge} {healing_badge} {circuit_breaker_badge}</div>
            </div>
            {token_reveal}
            <p class="muted" style="margin-bottom: 12px; font-size: 13px;">
                Hermes Agent operates as the guest AI copilot inside this virtual node instance, monitoring mempool health, tailing logs, and executing automated recovery restarts via scoped Model Context Protocol (MCP).
            </p>
            <div class="grid" style="grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 12px;">
                <div>
                    <span class="help">Scoped Node MCP Endpoint:</span>
                    <input class="mono" style="width: 100%; margin-top: 4px;" readonly value="{mcp_url}">
                </div>
                <div>
                    <span class="help">Heartbeat & Telemetry:</span>
                    <input class="mono" style="width: 100%; margin-top: 4px;" readonly value="{last_hb_text} (v{agent_version})">
                </div>
            </div>
            <div style="margin-bottom: 12px; display: flex; gap: 8px; flex-wrap: wrap;">
                <form method="post" action="/nodes/{encoded_id}/agent/token" style="display: inline;">
                    <button type="submit" class="btn small primary">🔑 Provision Scoped Hermes Token</button>
                </form>
                {healing_button}
                {ping_button}
            </div>
            <details class="panel" style="background: rgba(0,0,0,0.2); margin-top: 10px;">
                <summary style="cursor: pointer; font-weight: 500;">📋 View Hermes profile config.yaml setup snippet</summary>
                <pre style="margin-top: 8px; overflow-x: auto;"><code>{config_snippet}</code></pre>
            </details>
            <details class="panel" style="background: rgba(0,0,0,0.2); margin-top: 8px;">
                <summary style="cursor: pointer; font-weight: 500;">🧪 View MCP test curl command</summary>
                <pre style="margin-top: 8px; overflow-x: auto;"><code>{curl_example}</code></pre>
            </details>
        </div>"#,
        status_badge = status_badge,
        healing_badge = healing_badge,
        token_reveal = token_reveal,
        mcp_url = html::escape(&mcp_url),
        last_hb_text = html::escape(&last_hb_text),
        agent_version = html::escape(&assoc.agent_version),
        healing_button = healing_button,
        ping_button = ping_button,
        encoded_id = encoded_id,
        config_snippet = html::escape(&config_snippet),
        curl_example = html::escape(&curl_example),
    )
}

pub(crate) fn endpoints_card(state: &WebState, node: &NodeConfig) -> String {
    let rpc_text = if node.rpc_port == 0 {
        "Disabled (Zero RPC attack surface)".to_string()
    } else {
        format!("http://127.0.0.1:{}", node.rpc_port)
    };
    let ws_text = node
        .ws_port
        .map_or_else(|| "Not configured".to_string(), |p| format!("ws://127.0.0.1:{}", p));
    let p2p_text = format!("127.0.0.1:{}", node.p2p_port);
    let metrics_text = format!("http://127.0.0.1:8080/api/nodes/{}/metrics", node.id);
    let p2p_multiaddr = format!("/ip4/127.0.0.1/tcp/{}", node.p2p_port);

    let all_nodes = state.workspace.list_nodes().unwrap_or_default();
    let peers_on_net = all_nodes
        .iter()
        .filter(|n| n.id != node.id && n.network == node.network)
        .map(|n| {
            format!(
                r#"<span class="badge" title="P2P 127.0.0.1:{port}">📡 {name} (:{port})</span>"#,
                name = html::escape(&n.name),
                port = n.p2p_port
            )
        })
        .collect::<Vec<_>>();

    let peer_section = if peers_on_net.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div style="margin-top: 12px; padding-top: 10px; border-top: 1px solid var(--line);">
                <span class="help">Discovered Fleet Peers on {network} mesh:</span>
                <div style="display: flex; gap: 6px; flex-wrap: wrap; margin-top: 6px;">
                    {peers}
                </div>
            </div>"#,
            network = node.network,
            peers = peers_on_net.join(" ")
        )
    };

    format!(
        r#"<div class="panel endpoints-card">
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 12px;">
                <span style="font-size: 18px;">🌐</span>
                <strong>Networking & Instance Endpoints</strong>
            </div>
            <div class="grid" style="grid-template-columns: 1fr 1fr; gap: 12px;">
                <div>
                    <span class="help">P2P Mesh Peering:</span>
                    <input class="mono" readonly value="{p2p_text}" style="width: 100%; margin-top: 4px;">
                </div>
                <div>
                    <span class="help">JSON-RPC 2.0 API:</span>
                    <input class="mono" readonly value="{rpc_text}" style="width: 100%; margin-top: 4px;">
                </div>
                <div>
                    <span class="help">WebSocket Subscriptions:</span>
                    <input class="mono" readonly value="{ws_text}" style="width: 100%; margin-top: 4px;">
                </div>
                <div>
                    <span class="help">Prometheus Scrape:</span>
                    <input class="mono" readonly value="{metrics_text}" style="width: 100%; margin-top: 4px;">
                </div>
                <div style="grid-column: span 2;">
                    <span class="help">P2P Node Multiaddress (Fleet Interconnect):</span>
                    <input class="mono" readonly value="{p2p_multiaddr}" style="width: 100%; margin-top: 4px;">
                </div>
            </div>
            {peer_section}
        </div>"#,
        p2p_text = html::escape(&p2p_text),
        rpc_text = html::escape(&rpc_text),
        ws_text = html::escape(&ws_text),
        metrics_text = html::escape(&metrics_text),
        p2p_multiaddr = html::escape(&p2p_multiaddr),
        peer_section = peer_section,
    )
}
