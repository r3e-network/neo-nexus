//! Plugins: the capabilities a node's runtime can load, and which of them this
//! node has switched on. The catalogue comes from `PluginCatalog::for_node_type`,
//! so a neo-go node is never offered a NeoFS plugin, and toggling writes through
//! the same `set_plugin_enabled` the readiness evaluation reads back.

use axum::{
    extract::{Form, Path, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    catalog::{PluginCatalog, PluginId},
    types::NodeConfig,
};

use super::super::{html, WebState};

#[derive(Default, Deserialize)]
pub struct PluginQuery {
    #[serde(default)]
    node: String,
}

pub async fn plugins(
    State(state): State<WebState>,
    RawQuery(raw): RawQuery,
    Query(params): Query<PluginQuery>,
) -> Response {
    let body = match state.repository.list_nodes() {
        Ok(nodes) => render_body(&state, &nodes, &params.node),
        Err(error) => html::note(&format!("failed to load nodes: {error}")),
    };
    Html(html::layout(
        "Plugins",
        "plugins",
        &html::flash(raw.as_deref()),
        &body,
    ))
    .into_response()
}

fn render_body(state: &WebState, nodes: &[NodeConfig], wanted: &str) -> String {
    let Some(node) = pick_node(nodes, wanted) else {
        return format!(
            "<h1>Plugins</h1>\n{}",
            html::note("No nodes are registered yet, so there is nothing to enable.")
        );
    };

    let catalog = PluginCatalog;
    let applicable = catalog.for_node_type(node.node_type);
    let enabled = match state.repository.list_plugin_states(&node.id) {
        Ok(states) => states,
        Err(error) => return html::note(&format!("failed to load plugin state: {error}")),
    };
    let rows = applicable
        .iter()
        .map(|definition| {
            let is_enabled = enabled
                .iter()
                .any(|state| state.plugin_id == definition.id && state.enabled);
            html::row(&[
                html::cell(definition.name),
                html::cell(&definition.category.to_string()),
                html::cell(definition.description),
                html::cell(if definition.requires_restart {
                    "restart"
                } else {
                    "hot"
                }),
                html::raw_cell(&state_badge(is_enabled)),
                html::raw_cell(&toggle_form(node, definition.id, is_enabled)),
            ])
        })
        .collect::<Vec<_>>();

    format!(
        r#"<h1>Plugins</h1>
<div class="actions">{picker}</div>
{tiles}
{table}"#,
        picker = node_picker(nodes, node),
        tiles = html::cards(&[
            ("Node", node.name.clone()),
            ("Runtime", node.node_type.to_string()),
            ("Available", applicable.len().to_string()),
            (
                "Enabled",
                applicable
                    .iter()
                    .filter(|definition| {
                        enabled
                            .iter()
                            .any(|state| state.plugin_id == definition.id && state.enabled)
                    })
                    .count()
                    .to_string(),
            ),
        ]),
        table = html::table(
            &["Plugin", "Category", "Purpose", "Reload", "State", "Control"],
            &rows,
        ),
    )
}

fn state_badge(enabled: bool) -> String {
    let class = if enabled {
        "badge running"
    } else {
        "badge stopped"
    };
    format!(
        r#"<span class="{class}">{}</span>"#,
        if enabled { "enabled" } else { "off" }
    )
}

fn toggle_form(node: &NodeConfig, plugin_id: PluginId, enabled: bool) -> String {
    let label = if enabled { "Disable" } else { "Enable" };
    html::control_form(
        &format!("/plugins/{}/toggle", node.id),
        &[("plugin", &plugin_id.to_string())],
        label,
    )
}

fn node_picker(nodes: &[NodeConfig], selected: &NodeConfig) -> String {
    nodes
        .iter()
        .map(|node| {
            let current = if node.id == selected.id {
                " primary"
            } else {
                ""
            };
            format!(
                r#"<a class="btn{current}" href="/plugins?node={}">{}</a>"#,
                html::urlencoding_lite(&node.id),
                html::escape(&node.name)
            )
        })
        .collect()
}

fn pick_node<'a>(nodes: &'a [NodeConfig], wanted: &str) -> Option<&'a NodeConfig> {
    let wanted = wanted.trim();
    nodes
        .iter()
        .find(|node| !wanted.is_empty() && (node.name == wanted || node.id == wanted))
        .or_else(|| nodes.first())
}

#[derive(Deserialize)]
pub struct ToggleForm {
    #[serde(default)]
    plugin: String,
}

/// Toggling is a plain post so it works without JavaScript. The new state is
/// derived by negating what the workspace records, never taken from the form,
/// and the plugin must be one this node's runtime can actually load.
pub async fn toggle(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(input): Form<ToggleForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let node = state
            .repository
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == id)
            .ok_or_else(|| anyhow::anyhow!("node {id} was not found"))?;
        let plugin: PluginId = input
            .plugin
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("{} is not a plugin", input.plugin))?;
        let catalog = PluginCatalog;
        if !catalog
            .for_node_type(node.node_type)
            .iter()
            .any(|definition| definition.id == plugin)
        {
            anyhow::bail!("{plugin} does not apply to a {} node", node.node_type);
        }
        let currently_enabled = state
            .repository
            .list_plugin_states(&node.id)?
            .into_iter()
            .any(|record| record.plugin_id == plugin && record.enabled);
        let wanted = !currently_enabled;
        state
            .repository
            .set_plugin_enabled(&node.id, plugin, wanted)?;
        Ok(format!(
            "{} {} on {}",
            plugin,
            if wanted { "enabled" } else { "disabled" },
            node.name
        ))
    })();
    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("not changed: {error}"),
    };
    Redirect::to(&format!(
        "/plugins?node={}&flash={}",
        html::urlencoding_lite(&id),
        html::urlencoding_lite(&message)
    ))
    .into_response()
}
