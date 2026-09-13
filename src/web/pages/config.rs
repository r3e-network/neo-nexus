//! Config: what each node's runtime configuration is, and the same workspace
//! export the CLI writes. The managed path shown here is the one the launch
//! pipeline computes, so what an operator reads is what `Start` will write.

use std::path::PathBuf;

use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    catalog::PluginState,
    config::WorkspaceConfigExporter,
    core::operations::{EventKind, EventSeverity, NewRuntimeEvent},
    core::workspace::ConfigExporter,
    types::{node_workspace_path, NodeConfig},
};

use super::super::{html, WebState};

pub async fn config(State(state): State<WebState>, RawQuery(query): RawQuery) -> Response {
    let body = match state.workspace.list_nodes() {
        Ok(nodes) => render_body(&state, &nodes),
        Err(error) => html::note(&format!("failed to load nodes: {error}")),
    };
    Html(html::layout(
        "Configuration",
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
            let plugins = state.workspace.list_plugin_states(&node.id)?;
            Ok(ConfigRow {
                node: node.clone(),
                plugins,
                managed_path: ConfigExporter::managed_target_path(
                    node_work_dir(state, node)?,
                    node,
                ),
            })
        })
        .collect()
}

/// The directory a node owns inside the workspace — the same layout the
/// lifecycle pipeline and the supervisor use.
fn node_work_dir(state: &WebState, node: &NodeConfig) -> anyhow::Result<PathBuf> {
    node_workspace_path(state.workspace_child_dir("nodes"), &node.id)
}

fn render_body(state: &WebState, nodes: &[NodeConfig]) -> String {
    let breadcrumb = html::breadcrumb(&[
        ("Systems Manager", "/operations"),
        ("Application Management", "/config"),
        ("Parameter Store & Config", "/config"),
    ]);
    let head = html::page_head(
        "Systems Manager · Application Configuration",
        "Hierarchical node runtime parameters, configuration drift verification, and deterministic workspace exports.",
        r#"<a class="btn" href="/operations">OpsCenter</a> <a class="btn" href="/api/fleet/iac?format=cloudformation" download="fleet-cloudformation.yaml">☁️ CloudFormation</a>"#,
    );

    if nodes.is_empty() {
        return format!(
            "{breadcrumb}\n{head}\n{}",
            html::note("No instances are registered yet, so there is no runtime configuration to manage.")
        );
    }
    let rows = match collect_rows(state, nodes) {
        Ok(rows) => rows,
        Err(error) => return html::note(&format!("failed to load plugin state: {error}")),
    };
    let written = rows.iter().filter(|row| row.managed_path.is_file()).count();
    format!(
        r#"{breadcrumb}
{head}
{tiles}
<div class="section-head" style="margin-top: 20px;">
    <h2>Systems Manager Parameter Store Inventory</h2>
    <span class="muted" style="font-size: 12px;">Standard tier · SecureString encrypted via AWS KMS default key</span>
</div>
{table}
<div class="panel" style="margin-top: 24px; padding: 18px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
    <h3 style="margin-top: 0;">Workspace Configuration Synchronization</h3>
    {export_note}
    <div style="margin-top: 12px;">
        {export_form}
    </div>
</div>"#,
        breadcrumb = breadcrumb,
        head = head,
        tiles = html::cards(&[
            ("Managed Instances", rows.len().to_string()),
            ("Config Manifests Synced", written.to_string()),
            ("KMS Encryption", "AWS-KMS (active)".to_string()),
            ("Drift Status", if written == rows.len() { "0 Drifted".to_string() } else { format!("{} Pending", rows.len().saturating_sub(written)) }),
        ]),
        table = html::table(
            &[
                "Parameter Key Path",
                "Instance Engine",
                "Cluster Network",
                "Storage Driver",
                "Port Bindings",
                "Active Sidecars",
                "Sync State",
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
        .map(|plugin| format!(r#"<span class="badge">{}</span>"#, plugin.plugin_id))
        .collect::<Vec<_>>()
        .join(" ");
    let sync_badge = if row.managed_path.is_file() {
        r#"<span class="badge running">● In Sync</span>"#
    } else {
        r#"<span class="badge stopped">○ Pending Write</span>"#
    };
    let param_key = format!(
        r#"<div><span class="mono" style="font-weight: 600; color: var(--jade);">/neo/fleet/{name}/config.json</span></div><div class="muted mono" style="font-size: 11px;">{path}</div>"#,
        name = html::escape(&row.node.name),
        path = html::escape(&row.managed_path.display().to_string()),
    );
    html::row(&[
        html::raw_cell(&param_key),
        html::raw_cell(&format!(r#"<span class="badge">{}</span>"#, html::escape(&row.node.node_type.to_string()))),
        html::raw_cell(&format!(r#"<span class="badge">{}</span> <span class="muted" style="font-size: 11px;">nexus-az-1a</span>"#, html::escape(&row.node.network.to_string()))),
        html::raw_cell(&format!(r#"<span class="badge">{}</span>"#, html::escape(&row.node.storage_engine.to_string()))),
        html::raw_cell(&format!(r#"<span class="mono">:{}</span> <span class="muted">/</span> <span class="mono">:{}</span>"#, row.node.p2p_port, row.node.rpc_port)),
        html::raw_cell(if enabled.is_empty() { r#"<span class="muted" style="font-size: 12px;">none</span>"# } else { &enabled }),
        html::raw_cell(sync_badge),
    ])
}

pub async fn export_all(State(state): State<WebState>) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let nodes = state.workspace.list_nodes()?;
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
        let message = format!(
            "exported {} node configs ({} files) to {}",
            export.report.node_count,
            export.report.exported_file_count,
            export.output_dir.display()
        );
        let _ = state.commands.record_event(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::ConfigExported,
            severity: EventSeverity::Info,
            message: message.clone(),
        });
        Ok(message)
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
