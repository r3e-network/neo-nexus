//! Config: what each node's runtime configuration is, and the same workspace
//! export the CLI writes. The managed path shown here is the one the launch
//! pipeline computes, so what an operator reads is what `Start` will write.

use std::path::PathBuf;

use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    catalog::PluginState, config::WorkspaceConfigExporter, core::workspace::ConfigExporter,
    types::NodeConfig,
};

use super::super::{html, WebState};

pub async fn config(State(state): State<WebState>, RawQuery(query): RawQuery) -> Response {
    let body = match state.repository.list_nodes() {
        Ok(nodes) => render_body(&state, &nodes),
        Err(error) => html::note(&format!("failed to load nodes: {error}")),
    };
    Html(html::layout(
        "Config",
        "config",
        &html::flash(query.as_deref()),
        &body,
    ))
    .into_response()
}

/// A node, its plugin state, and the config path its launch would use.
struct ConfigRow {
    node: NodeConfig,
    plugins: Vec<PluginState>,
    managed_path: PathBuf,
}

fn collect_rows(state: &WebState, nodes: &[NodeConfig]) -> anyhow::Result<Vec<ConfigRow>> {
    nodes
        .iter()
        .map(|node| {
            let plugins = state.repository.list_plugin_states(&node.id)?;
            Ok(ConfigRow {
                node: node.clone(),
                plugins,
                managed_path: ConfigExporter::managed_target_path(node_work_dir(state, node), node),
            })
        })
        .collect()
}

/// The directory a node owns inside the workspace — the same layout the
/// lifecycle pipeline and the supervisor use.
fn node_work_dir(state: &WebState, node: &NodeConfig) -> PathBuf {
    state.workspace_child_dir("nodes").join(&node.id)
}

fn render_body(state: &WebState, nodes: &[NodeConfig]) -> String {
    if nodes.is_empty() {
        return format!(
            "<h1>Config</h1>\n{}",
            html::note("No nodes are registered yet, so there is no configuration to report.")
        );
    }
    let rows = match collect_rows(state, nodes) {
        Ok(rows) => rows,
        Err(error) => return html::note(&format!("failed to load plugin state: {error}")),
    };
    let written = rows.iter().filter(|row| row.managed_path.is_file()).count();
    format!(
        r#"<h1>Config</h1>
{tiles}
{table}
<h2>Workspace export</h2>
{export_note}
{export_form}"#,
        tiles = html::cards(&[
            ("Nodes", rows.len().to_string()),
            ("Configs written", written.to_string()),
        ]),
        table = html::table(
            &[
                "Node",
                "Runtime",
                "Network",
                "Storage",
                "RPC/P2P",
                "Plugins enabled",
                "Managed config",
                "Written",
            ],
            &rows.iter().map(config_row).collect::<Vec<_>>(),
        ),
        export_note = html::note(
            "Export writes every node runtime config plus report files — the same artifact --export-node-configs produces.",
        ),
        export_form = html::control_form("/config/export", &[], "Export workspace configs"),
    )
}

fn config_row(row: &ConfigRow) -> String {
    let enabled = row
        .plugins
        .iter()
        .filter(|plugin| plugin.enabled)
        .map(|plugin| plugin.plugin_id.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    html::row(&[
        html::cell(&row.node.name),
        html::cell(&row.node.node_type.to_string()),
        html::cell(&row.node.network.to_string()),
        html::cell(&row.node.storage_engine.to_string()),
        html::cell(&format!("{}/{}", row.node.rpc_port, row.node.p2p_port)),
        html::cell(if enabled.is_empty() { "none" } else { &enabled }),
        html::cell(&row.managed_path.display().to_string()),
        html::cell(if row.managed_path.is_file() {
            "yes"
        } else {
            "no"
        }),
    ])
}

pub async fn export_all(State(state): State<WebState>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let nodes = state.repository.list_nodes()?;
        let rows = collect_rows(&state, &nodes)?;
        let paired = rows
            .iter()
            .map(|row| (row.node.clone(), row.plugins.clone()))
            .collect::<Vec<_>>();
        let output = state.workspace_child_dir("export").join("configs");
        let export = WorkspaceConfigExporter::write(
            &output,
            database_path(&state),
            &paired,
            env!("CARGO_PKG_VERSION"),
        )?;
        Ok(format!(
            "exported {} node configs ({} files) to {}",
            export.report.node_count,
            export.report.exported_file_count,
            export.output_dir.display()
        ))
    })();
    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("export failed: {error}"),
    };
    Redirect::to(&format!(
        "/config?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}

fn database_path(state: &WebState) -> PathBuf {
    state.data_dir.join("neonexus.db")
}
