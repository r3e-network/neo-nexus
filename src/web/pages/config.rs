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
    config::{ConfigDriftDetector, ConfigDriftReport, ConfigDriftStatus, WorkspaceConfigExporter},
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
    /// What the real comparator found, not whether a file happens to exist.
    ///
    /// `Err` when the check itself could not run — a node whose config cannot
    /// be rendered, most often because its runtime is unbound. That is its own
    /// answer and must not be shown as "in sync".
    drift: Result<ConfigDriftReport, String>,
}

fn collect_rows(state: &WebState, nodes: &[NodeConfig]) -> anyhow::Result<Vec<ConfigRow>> {
    nodes
        .iter()
        .map(|node| {
            let plugins = state.workspace.list_plugin_states(&node.id)?;
            let managed_path =
                ConfigExporter::managed_target_path(node_work_dir(state, node)?, node);
            // The page promised "configuration drift verification" and delivered
            // `Path::is_file()`. The real comparator hashes the rendered config
            // against the file and re-validates the file semantically; it was
            // reachable only from `--check-config-drift`. It runs here now, so a
            // node edited by hand since its last launch says so.
            let drift = ConfigDriftDetector::check(node, &managed_path)
                .map_err(|error| format!("{error:#}"));
            Ok(ConfigRow {
                node: node.clone(),
                plugins,
                managed_path,
                drift,
            })
        })
        .collect()
}

/// What the comparator found, with the first difference an operator would act
/// on.
///
/// Every outcome is its own badge. In particular a check that could not run is
/// neither drift nor agreement — treating it as either is how a page ends an
/// investigation early.
fn drift_badge(row: &ConfigRow) -> String {
    match &row.drift {
        Ok(report) => match report.status {
            ConfigDriftStatus::InSync => {
                r#"<span class="badge running">Matches</span>"#.to_string()
            }
            ConfigDriftStatus::Drifted => format!(
                r#"<span class="badge error">Drifted</span><div class="muted" style="font-size: 11px;">{}</div>"#,
                html::escape(
                    &report
                        .differences
                        .first()
                        .map_or_else(|| "differs from what we would generate".to_string(), |difference| difference.detail.clone())
                )
            ),
            ConfigDriftStatus::Missing => {
                r#"<span class="badge stopped">Not written yet</span><div class="muted" style="font-size: 11px;">it will be written on the next start</div>"#.to_string()
            }
        },
        Err(error) => format!(
            r#"<span class="badge">Cannot check</span><div class="muted" style="font-size: 11px;">{}</div>"#,
            html::escape(error)
        ),
    }
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
            html::note(
                "No instances are registered yet, so there is no runtime configuration to manage."
            )
        );
    }
    let rows = match collect_rows(state, nodes) {
        Ok(rows) => rows,
        Err(error) => return html::note(&format!("failed to load plugin state: {error}")),
    };
    let in_sync = rows
        .iter()
        .filter(|row| {
            row.drift
                .as_ref()
                .is_ok_and(|report| report.status.is_in_sync())
        })
        .count();
    let drifted = rows
        .iter()
        .filter(|row| {
            row.drift
                .as_ref()
                .is_ok_and(|report| report.status == ConfigDriftStatus::Drifted)
        })
        .count();
    let unknown = rows.len() - in_sync - drifted;
    format!(
        r#"{breadcrumb}
{head}
{tiles}
<div class="section-head" style="margin-top: 20px;">
    <h2>Managed configuration</h2>
    <span class="muted" style="font-size: 12px;">Each file is hashed against what this workspace would generate now.</span>
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
        // "KMS Encryption: AWS-KMS (active)" asserted encryption that does not
        // exist: there is no AWS SDK in this project and no encryption anywhere
        // under src/config/. The files *are* written 0600, which is a real and
        // different guarantee — and one worth stating, because the generated
        // configs embed plaintext wallet unlock passwords.
        tiles = html::cards(&[
            ("Nodes", rows.len().to_string()),
            ("Match what we would generate", in_sync.to_string()),
            ("Drifted", drifted.to_string()),
            ("Not checked", unknown.to_string()),
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
    let sync_badge = drift_badge(row);
    let param_key = format!(
        r#"<div><span class="mono" style="font-weight: 600; color: var(--jade);">/neo/fleet/{name}/config.json</span></div><div class="muted mono" style="font-size: 11px;">{path}</div>"#,
        name = html::escape(&row.node.name),
        path = html::escape(&row.managed_path.display().to_string()),
    );
    html::row(&[
        html::raw_cell(&param_key),
        html::raw_cell(&format!(
            r#"<span class="badge">{}</span>"#,
            html::escape(&row.node.node_type.to_string())
        )),
        html::raw_cell(&format!(
            r#"<span class="badge">{}</span>"#,
            html::escape(&row.node.network.to_string())
        )),
        html::raw_cell(&format!(
            r#"<span class="badge">{}</span>"#,
            html::escape(&row.node.storage_engine.to_string())
        )),
        html::raw_cell(&format!(
            r#"<span class="mono">:{}</span> <span class="muted">/</span> <span class="mono">:{}</span>"#,
            row.node.p2p_port, row.node.rpc_port
        )),
        html::raw_cell(if enabled.is_empty() {
            r#"<span class="muted" style="font-size: 12px;">none</span>"#
        } else {
            &enabled
        }),
        html::raw_cell(&sync_badge),
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
