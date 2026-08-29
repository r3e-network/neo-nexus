//! Roles: which duties each client actually supports, and what adopting a role
//! would change on a given node. The matrix is the same `role_availability`
//! table the launch planner consults, so an operator is never shown a duty the
//! planner would later refuse.

use axum::{
    extract::{Query, RawQuery, State},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    core::workspace::role_availability,
    roles::{NodeRole, RoleAvailability, RolePlanner},
    types::{NodeConfig, NodeType},
};

use super::super::{html, WebState};

#[derive(Default, Deserialize)]
pub struct RoleQuery {
    #[serde(default)]
    node: String,
    #[serde(default)]
    role: String,
}

pub async fn roles(
    State(state): State<WebState>,
    RawQuery(raw): RawQuery,
    Query(params): Query<RoleQuery>,
) -> Response {
    let body = match state.repository.list_nodes() {
        Ok(nodes) => render_body(&nodes, &params),
        Err(error) => html::note(&format!("failed to load nodes: {error}")),
    };
    Html(html::layout(
        "Private network",
        "roles",
        &html::flash(raw.as_deref()),
        &body,
    ))
    .into_response()
}

fn render_body(nodes: &[NodeConfig], params: &RoleQuery) -> String {
    format!(
        r#"<h1>Roles and duties</h1>
<h2>Support matrix</h2>
{matrix}
{planner}"#,
        matrix = support_matrix(),
        planner = role_planner(nodes, params),
    )
}

/// Every cell is stated, so a new client or duty cannot default to "supported".
fn support_matrix() -> String {
    let head = std::iter::once("Duty".to_string())
        .chain(NodeType::ALL.iter().map(|node_type| node_type.to_string()))
        .collect::<Vec<_>>();
    let rows = NodeRole::ALL
        .iter()
        .map(|role| {
            html::row(
                &std::iter::once(html::cell(role.label()))
                    .chain(NodeType::ALL.iter().map(|node_type| {
                        html::raw_cell(&availability_cell(role_availability(*node_type, *role)))
                    }))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let headers = head.iter().map(String::as_str).collect::<Vec<_>>();
    html::table(&headers, &rows)
}

fn availability_cell(availability: RoleAvailability) -> String {
    if availability.is_supported() {
        return r#"<span class="badge running">supported</span>"#.to_string();
    }
    let class = match availability {
        RoleAvailability::Unverified(_) => "badge starting",
        _ => "badge error",
    };
    let reason = availability.reason().unwrap_or("unavailable");
    format!(
        r#"<span class="{class}" title="{}">{}</span>"#,
        html::escape(reason),
        html::escape(short_reason(reason))
    )
}

/// The matrix is a scan surface; the full sentence belongs in the tooltip and on
/// the planner below.
fn short_reason(reason: &str) -> &str {
    reason.split([':', '.']).next().unwrap_or(reason).trim()
}

fn role_planner(nodes: &[NodeConfig], params: &RoleQuery) -> String {
    let Some(node) = pick_node(nodes, &params.node) else {
        return html::note("No nodes are registered yet, so there is no node to plan a role for.");
    };
    let Some(role) = pick_role(&params.role) else {
        return format!(
            "<h2>Plan for {}</h2>\n{}",
            html::escape(&node.name),
            html::note("Choose a duty to see what adopting it would change.")
        );
    };
    let plan = RolePlanner::plan(node, role);
    let availability = role_availability(node.node_type, role);
    let changes = plan
        .plugin_changes
        .iter()
        .map(|change| {
            html::row(&[
                html::cell(&change.plugin_id.to_string()),
                html::cell(if change.enabled { "enable" } else { "disable" }),
                html::cell(change.reason),
            ])
        })
        .collect::<Vec<_>>();
    let notes = plan
        .notes
        .iter()
        .map(|note| format!("<li>{}</li>", html::escape(note)))
        .collect::<String>();
    format!(
        r#"<h2>Plan for {node} as {role}</h2>
{warning}
{filters}
{changes}
{notes}"#,
        node = html::escape(&node.name),
        role = html::escape(role.label()),
        warning = availability
            .reason()
            .map(|reason| {
                html::note(&format!(
                    "this duty is not available on {}: {reason}",
                    node.node_type
                ))
            })
            .unwrap_or_default(),
        filters = html::filter_form_with_hidden(
            "/roles",
            &[("node", &node.id)],
            &[("role", role.label())],
        ),
        changes = if changes.is_empty() {
            html::note("Adopting this duty changes no plugins on this runtime.")
        } else {
            html::table(&["Plugin", "Change", "Why"], &changes)
        },
        notes = if notes.is_empty() {
            String::new()
        } else {
            format!("<ul class=\"muted\">{notes}</ul>")
        },
    )
}

fn pick_node<'a>(nodes: &'a [NodeConfig], wanted: &str) -> Option<&'a NodeConfig> {
    let wanted = wanted.trim();
    nodes
        .iter()
        .find(|node| !wanted.is_empty() && (node.name == wanted || node.id == wanted))
        .or_else(|| nodes.first())
}

/// Duty labels contain separators (`RPC/API`, `State Validator`), so the page
/// round-trips the label itself and matches it case-insensitively rather than
/// depending on an exact string comparison.
fn pick_role(raw: &str) -> Option<NodeRole> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    NodeRole::ALL
        .iter()
        .copied()
        .find(|role| role.label().eq_ignore_ascii_case(raw))
}
