//! Roles: which duties each client actually supports, and what adopting a role
//! would change on a given node. The matrix is the same `role_availability`
//! table the launch planner consults, so an operator is never shown a duty the
//! planner would later refuse.

use axum::{
    extract::{Form, Path, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    core::workspace::role_availability,
    events::{EventKind, EventSeverity, NewRuntimeEvent},
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
    let body = match state.workspace.list_nodes() {
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
        r#"<h1>Private network</h1>
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
    let filters = planner_form(nodes, node, &params.role);
    let Some(role) = pick_role(&params.role) else {
        return format!(
            "<h2>Role planner</h2>\n{filters}\n{}",
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
    let apply = if availability.is_supported() {
        if node.status.is_active() || node.pid.is_some() {
            html::note("Stop and settle this node before applying a different duty.")
        } else {
            format!(
                r#"<form method="post" action="/nodes/{}/role"><input type="hidden" name="role" value="{}"><button class="primary" type="submit">Apply {} duty</button></form>"#,
                html::urlencoding_lite(&node.id),
                html::escape(role.persist_key()),
                html::escape(role.label()),
            )
        }
    } else {
        String::new()
    };
    format!(
        r#"<h2>Plan for {node} as {role}</h2>
{warning}
{filters}
{changes}
{notes}
{apply}"#,
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
        filters = filters,
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
        apply = apply,
    )
}

#[derive(Deserialize)]
pub struct ApplyRoleForm {
    role: String,
}

pub async fn apply_role(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(input): Form<ApplyRoleForm>,
) -> Response {
    let result = (|| -> anyhow::Result<String> {
        let node = state
            .workspace
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == id)
            .ok_or_else(|| anyhow::anyhow!("node {id} was not found"))?;
        let role = NodeRole::from_persist_key(input.role.trim())
            .ok_or_else(|| anyhow::anyhow!("{} is not a supported duty", input.role))?;
        let availability = role_availability(node.node_type, role);
        if !availability.is_supported() {
            anyhow::bail!(
                "{} cannot apply {}: {}",
                node.node_type,
                role.label(),
                availability.reason().unwrap_or("duty is unavailable")
            );
        }
        let plan = RolePlanner::plan(&node, role);
        state.commands.apply_node_role_plan(&node.id, &plan)?;
        let message = format!("{} duty set to {}", node.name, role.label());
        let _ = state.commands.record_event(NewRuntimeEvent {
            node_id: Some(node.id),
            node_name: Some(node.name),
            kind: EventKind::RoleApplied,
            severity: EventSeverity::Info,
            message: message.clone(),
        });
        Ok(message)
    })();
    let message = result.unwrap_or_else(|error| format!("duty not applied: {error}"));
    Redirect::to(&format!(
        "/roles?node={}&role={}&flash={}",
        crate::web::html::urlencoding_lite(&id),
        crate::web::html::urlencoding_lite(input.role.trim()),
        crate::web::html::urlencoding_lite(&message),
    ))
    .into_response()
}

fn planner_form(nodes: &[NodeConfig], selected_node: &NodeConfig, selected_role: &str) -> String {
    let node_options = nodes
        .iter()
        .map(|node| {
            let selected = if node.id == selected_node.id {
                " selected"
            } else {
                ""
            };
            format!(
                r#"<option value="{}"{selected}>{}</option>"#,
                html::escape(&node.id),
                html::escape(&node.name)
            )
        })
        .collect::<String>();
    let role_options = std::iter::once(r#"<option value="">Choose a duty</option>"#.to_string())
        .chain(NodeRole::ALL.iter().map(|role| {
            let label = role.label();
            let selected = if pick_role(selected_role) == Some(*role) {
                " selected"
            } else {
                ""
            };
            format!(
                r#"<option value="{}"{selected}>{}</option>"#,
                html::escape(label),
                html::escape(label)
            )
        }))
        .collect::<String>();
    format!(
        r#"<form class="filters" method="get" action="/roles">
<label class="field" for="role-node"><span>Node</span><select id="role-node" name="node">{node_options}</select></label>
<label class="field" for="role-duty"><span>Duty</span><select id="role-duty" name="role">{role_options}</select></label>
<button type="submit">Show plan</button>
</form>"#
    )
}

fn pick_node<'a>(nodes: &'a [NodeConfig], wanted: &str) -> Option<&'a NodeConfig> {
    let wanted = wanted.trim();
    nodes
        .iter()
        .find(|node| !wanted.is_empty() && (node.name == wanted || node.id == wanted))
        .or_else(|| nodes.first())
}

/// Accept both the human label used by the planner form and the stable key used
/// by the apply form/redirect. Keeping the redirect on the stable key avoids a
/// URL contract tied to presentation text, while still accepting bookmarked
/// label URLs from earlier versions.
fn pick_role(raw: &str) -> Option<NodeRole> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    NodeRole::ALL.iter().copied().find(|role| {
        role.label().eq_ignore_ascii_case(raw) || role.persist_key().eq_ignore_ascii_case(raw)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_query_round_trips_labels_and_persisted_keys() {
        for role in NodeRole::ALL {
            assert_eq!(pick_role(role.label()), Some(role));
            assert_eq!(pick_role(role.persist_key()), Some(role));
        }
        assert_eq!(pick_role("  VALIDATOR  "), Some(NodeRole::Consensus));
        assert_eq!(pick_role(""), None);
        assert_eq!(pick_role("unknown"), None);
    }
}
