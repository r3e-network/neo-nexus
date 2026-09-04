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
    core::operations::{EventKind, EventSeverity, NewRuntimeEvent},
    plugins::{
        installed_plugin_release, PluginPackageManager, PluginPackageManifest,
        PluginReleaseMetadata,
    },
    types::{NodeConfig, NodeType},
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
    let installations = state
        .repository
        .list_plugin_installations(&node.id)
        .unwrap_or_default();
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
                html::cell(
                    &installations
                        .iter()
                        .find(|installation| installation.plugin_id == definition.id)
                        .map(|installation| {
                            match installed_plugin_release(&installation.manifest_path) {
                                Ok(Some(release)) => format!(
                                    "{}; neo-cli {}; {}",
                                    release.version,
                                    release.compatible_runtime_versions.join(", "),
                                    if release.validate_for(node).is_ok() {
                                        "compatible"
                                    } else {
                                        "incompatible"
                                    }
                                ),
                                Ok(None) => format!(
                                    "unversioned; sha256 {}",
                                    installation.sha256.chars().take(12).collect::<String>()
                                ),
                                Err(error) => format!("manifest unavailable: {error}"),
                            }
                        })
                        .unwrap_or_else(|| {
                            if node.node_type == NodeType::NeoCli {
                                "not installed".into()
                            } else {
                                "built in".into()
                            }
                        }),
                ),
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
{table}
{install}"#,
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
        install = format!("{}{}", install_form(node), signclient_form(state, node)),
        table = html::table(
            &[
                "Plugin",
                "Category",
                "Purpose",
                "Installed version",
                "Reload",
                "State",
                "Control"
            ],
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
        let _supervisor = state.supervisor();
        if node.node_type == NodeType::NeoCli {
            if node.status.is_running() || node.pid.is_some() || _supervisor.is_managing(&node.id) {
                anyhow::bail!("stop the neo-cli node before enabling or disabling plugin packages");
            }
            let work = state.workspace_child_dir("nodes").join(&node.id);
            PluginPackageManager::set_enabled(&work, plugin, wanted)?;
            PluginPackageManager::refresh_installation_paths(&state.repository, &work, &node.id)?;
        }
        state
            .repository
            .set_plugin_enabled(&node.id, plugin, wanted)?;
        let message = format!(
            "{} {} on {}",
            plugin,
            if wanted { "enabled" } else { "disabled" },
            node.name
        );
        // Enabling a governance plugin is an on-chain commitment, so the record
        // of who flipped it matters more here than for most toggles.
        let _ = state.repository.record_event(NewRuntimeEvent {
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

fn install_form(node: &NodeConfig) -> String {
    if node.node_type != NodeType::NeoCli {
        return String::new();
    }
    let options = PluginCatalog
        .for_node_type(node.node_type)
        .iter()
        .map(|plugin| {
            format!(
                r#"<option value="{}">{}</option>"#,
                plugin.id,
                html::escape(plugin.name)
            )
        })
        .collect::<String>();
    format!(
        r#"<h2>Install or change plugin version</h2>
<p>Stop the node first. Choose a verified local ZIP package and its declared compatible neo-cli version. Existing configuration changes require review in <a href="/config">Config</a>. Installing an older compatible package restores that version.</p>
<form method="post" action="/plugins/{id}/install">
<label>Plugin<select name="plugin">{options}</select></label>
<label>Plugin version<input name="version" required></label>
<label>Compatible neo-cli version<input name="runtime_version" value="{version}" required></label>
<label>Local ZIP path<input name="source" required></label>
<label>Expected SHA-256<input name="sha256" required minlength="64" maxlength="64"></label>
<button type="submit">Verify and install plugin</button></form>"#,
        id = html::escape(&node.id),
        version = html::escape(&node.runtime_version)
    )
}

#[derive(Deserialize)]
pub struct InstallPluginForm {
    plugin: String,
    version: String,
    runtime_version: String,
    source: String,
    sha256: String,
}

pub async fn install(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(input): Form<InstallPluginForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        // Serialize package publication with node starts and watchdog restarts.
        let supervisor = state.supervisor();
        let node = state
            .repository
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == id)
            .ok_or_else(|| anyhow::anyhow!("node not found"))?;
        if supervisor.is_managing(&id) {
            anyhow::bail!("stop the node before installing plugins");
        }
        let plugin_id = input.plugin.parse::<PluginId>()?;
        let release = PluginReleaseMetadata {
            version: input.version.trim().into(),
            compatible_runtime_versions: vec![input.runtime_version.trim().into()],
        };
        let installation = PluginPackageManager::install_with_release(
            &PluginPackageManifest {
                plugin_id,
                label: format!("{plugin_id} {}", release.version),
                source_path: input.source.trim().into(),
                expected_sha256: input.sha256,
            },
            &node,
            state.workspace_child_dir("nodes").join(&node.id),
            Some(&release),
        )?;
        state.repository.upsert_plugin_installation(&installation)?;
        let message = format!("installed {plugin_id} {} on {}", release.version, node.name);
        let _ = state.repository.record_event(NewRuntimeEvent {
            node_id: Some(node.id),
            node_name: Some(node.name),
            kind: EventKind::PluginUpdated,
            severity: EventSeverity::Info,
            message: message.clone(),
        });
        Ok(message)
    })();
    let message = outcome.unwrap_or_else(|error| format!("plugin not installed: {error}"));
    Redirect::to(&format!(
        "/plugins?node={}&flash={}",
        html::urlencoding_lite(&id),
        html::urlencoding_lite(&message)
    ))
    .into_response()
}

fn signclient_form(state: &WebState, node: &NodeConfig) -> String {
    if node.node_type != NodeType::NeoCli {
        return String::new();
    }
    let settings = crate::plugins::SignClientSettings::read_for_node(
        &state.workspace_child_dir("nodes").join(&node.id),
    )
    .unwrap_or_default();
    format!(
        r#"<h2>SignClient bridge</h2>
<p>Install and enable a compatible SignClient package first. This connection uses the signer service's local gRPC bridge. The bridge selects the custody key; no private key or caller token is written into node configuration. Start consensus explicitly with <code>start consensus SignClient</code> (substitute the configured signer name); enabling this plugin does not start consensus automatically.</p>
<form method="post" action="/plugins/{id}/signclient">
<label>Signer name<input name="name" value="{name}" required></label>
<label>Local bridge endpoint<input name="endpoint" value="{endpoint}" required></label>
<button type="submit">Save SignClient configuration</button></form>"#,
        id = html::escape(&node.id),
        name = html::escape(&settings.name),
        endpoint = html::escape(&settings.endpoint)
    )
}

#[derive(Deserialize)]
pub struct SignClientForm {
    name: String,
    endpoint: String,
}

pub async fn configure_signclient(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(input): Form<SignClientForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let supervisor = state.supervisor();
        let node = state
            .repository
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == id)
            .ok_or_else(|| anyhow::anyhow!("node not found"))?;
        if node.status.is_running() || node.pid.is_some() || supervisor.is_managing(&id) {
            anyhow::bail!("stop the node before changing its SignClient settings");
        }
        let settings = crate::plugins::SignClientSettings {
            name: input.name.trim().into(),
            endpoint: input.endpoint.trim().into(),
        };
        let backup =
            settings.write_for_node(&state.workspace_child_dir("nodes").join(&id), &node)?;
        let message = format!(
            "SignClient endpoint configured for {}; start consensus explicitly with signer {}{}",
            node.name,
            settings.name,
            backup
                .map(|path| format!("; previous config backed up to {}", path.display()))
                .unwrap_or_default()
        );
        let _ = state.repository.record_event(NewRuntimeEvent {
            node_id: Some(node.id),
            node_name: Some(node.name),
            kind: EventKind::PluginUpdated,
            severity: EventSeverity::Info,
            message: message.clone(),
        });
        Ok(message)
    })();
    let message =
        outcome.unwrap_or_else(|error| format!("SignClient configuration not changed: {error}"));
    Redirect::to(&format!(
        "/plugins?node={}&flash={}",
        html::urlencoding_lite(&id),
        html::urlencoding_lite(&message)
    ))
    .into_response()
}
