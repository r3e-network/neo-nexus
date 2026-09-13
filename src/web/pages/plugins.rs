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
    catalog::{PluginCatalog, PluginDefinition, PluginId},
    core::operations::{EventKind, EventSeverity, NewRuntimeEvent},
    plugins::{
        ensure_plugin_configuration_supported, ensure_plugin_installable, plugin_support_guidance,
    },
    types::{NodeConfig, NodeType, NodeTypeTraits},
};

use super::super::{html, jobs::JobStatus, plugin_ops, WebState};

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
    let body = match state.workspace.list_nodes() {
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
    let enabled = match state.workspace.list_plugin_states(&node.id) {
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
                    "restart required"
                } else {
                    "see runtime documentation"
                }),
                html::raw_cell(&state_badge(is_enabled)),
                html::raw_cell(&toggle_form(node, definition.id, is_enabled)),
            ])
        })
        .collect::<Vec<_>>();

    format!(
        r#"<h1>Plugins</h1>
<nav class="actions" aria-label="Select a node for plugin management">{picker}</nav>
<h2>Runtime compatibility</h2>
{compatibility}
{tiles}
<h2>Launch configuration</h2>
{configuration_note}
{table}
{install}
{jobs}"#,
        picker = node_picker(nodes, node),
        compatibility = html::notice(
            if node.node_type.supports_plugins() { "ok" } else { "warn" },
            plugin_support_guidance(node.node_type),
        ),
        tiles = html::cards(&[
            ("Node", node.name.clone()),
            ("Runtime", node.node_type.to_string()),
            ("Configurable entries", applicable.len().to_string()),
            (
                "Configured enabled",
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
        configuration_note = html::note(
            "These controls save launch configuration, not live runtime state. Stop and settle the node before changing them; start it again to apply changes. Enabled does not confirm that a package is installed or loaded.",
        ),
        table = if rows.is_empty() {
            html::note("No plugin configuration controls are available for this runtime in the managed catalog.")
        } else {
            html::table(
                &["Capability", "Category", "Purpose", "Applies", "Configuration", "Control"],
                &rows,
            )
        },
        install = install_form(node, &applicable),
        jobs = job_panel(state, node),
    )
}

/// The upload form that installs a plugin package onto the selected node.
///
/// Plugin packages are a neo-cli capability, so for any other runtime the form
/// is replaced by a plain note rather than offering an action that can only
/// fail. The node is fixed to the one the page is showing, and the plugin choice
/// is confined to the catalogue entries that node's runtime can actually load.
fn install_form(node: &NodeConfig, applicable: &[&PluginDefinition]) -> String {
    if let Err(error) = ensure_plugin_installable(node) {
        return format!(
            "<h2>Install a package</h2>\n{}\n{}",
            support_badge(node.node_type),
            html::notice("warn", &error.to_string()),
        );
    }
    let options = applicable
        .iter()
        .map(|definition| {
            format!(
                r#"<option value="{}">{}</option>"#,
                html::escape(&definition.id.to_string()),
                html::escape(definition.name),
            )
        })
        .collect::<String>();
    format!(
        r#"<h2>Install a package</h2>
{support}
<p class="muted">Upload a verified C# DLL ZIP package. Installation writes files only; save the required configuration and start the node to load them. Managed entries require restart.</p>
<form class="filters" method="post" action="/plugins/install" enctype="multipart/form-data">
<input type="hidden" name="node_id" value="{node_id}">
<label class="field"><span>Plugin</span><select name="plugin_id">{options}</select></label>
<label class="field"><span>Label</span><input name="label" required></label>
<label class="field"><span>Package (.zip)</span><input type="file" name="package" accept=".zip" required></label>
<label class="field"><span>Expected SHA-256</span><input name="expected_sha256" class="mono" required></label>
<button type="submit">Install plugin</button>
</form>"#,
        support = support_badge(node.node_type),
        node_id = html::escape(&node.id),
        options = options,
    )
}

fn support_badge(node_type: NodeType) -> String {
    let (class, label) = if node_type.supports_plugins() {
        ("badge running", "✅ Plugin packages supported")
    } else {
        ("badge event-warning", "⚠️ Plugin packages not supported")
    };
    format!(r#"<span class="{class}">{label}</span>"#)
}

fn job_panel(state: &WebState, node: &NodeConfig) -> String {
    let rows = state
        .jobs
        .recent()
        .iter()
        .filter(|job| job.lane == plugin_ops::LANE)
        .take(5)
        .map(|job| {
            let class = match job.status {
                JobStatus::Running => "badge starting",
                JobStatus::Succeeded => "badge running",
                JobStatus::Failed => "badge error",
            };
            html::row(&[
                html::raw_cell(&format!(
                    r#"<span class="{class}">{}</span>"#,
                    html::escape(job.status.label()),
                )),
                html::cell(&job.description),
                html::cell(&job.detail),
            ])
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return String::new();
    }
    format!(
        r#"<h2>Recent package jobs (workspace)</h2><p class="muted">Started means queued, not installed or loaded. Refresh to see the final result.</p>{}<a class="btn small" href="/plugins?node={}">Refresh package status</a>"#,
        html::table(&["State", "Work", "Result"], &rows),
        html::urlencoding_lite(&node.id),
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
        if enabled {
            "enabled for next launch"
        } else {
            "off"
        }
    )
}

fn toggle_form(node: &NodeConfig, plugin_id: PluginId, enabled: bool) -> String {
    let label = if enabled { "Disable" } else { "Enable" };
    if node.status.is_active() || node.pid.is_some() {
        return format!(
            r#"<button type="button" disabled title="Stop and settle the node before changing its launch configuration">{}</button>"#,
            html::escape(label),
        );
    }
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
                r#"<a class="btn{current}" href="/plugins?node={}">{} · {} {}</a>"#,
                html::urlencoding_lite(&node.id),
                html::escape(&node.name),
                html::escape(&node.node_type.to_string()),
                support_badge(node.node_type),
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
            .workspace
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == id)
            .ok_or_else(|| anyhow::anyhow!("node {id} was not found"))?;
        let plugin: PluginId = input
            .plugin
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("{} is not a plugin", input.plugin))?;
        ensure_plugin_configuration_supported(node.node_type, plugin)?;
        let currently_enabled = state
            .workspace
            .list_plugin_states(&node.id)?
            .into_iter()
            .any(|record| record.plugin_id == plugin && record.enabled);
        let wanted = !currently_enabled;
        state
            .commands
            .set_plugin_enabled(&node.id, plugin, wanted)?;
        let message = format!(
            "{} {} in launch configuration for {}; takes effect on the next launch, not a live plugin load",
            plugin,
            if wanted { "enabled" } else { "disabled" },
            node.name
        );
        // Enabling a governance plugin is an on-chain commitment, so the record
        // of who flipped it matters more here than for most toggles.
        let _ = state.commands.record_event(NewRuntimeEvent {
            node_id: Some(node.id.clone()),
            node_name: Some(node.name.clone()),
            kind: EventKind::PluginUpdated,
            severity: EventSeverity::Info,
            message: message.clone(),
        });
        Ok(message)
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
