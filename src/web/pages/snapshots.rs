//! Snapshots: the fast-sync archives the workspace knows about, and how far
//! along each one is — declared, downloaded, hash-verified, applied. Verification
//! is a real hash check against the archive, so this page reports the recorded
//! result instead of offering to recompute it on a page load.

use axum::{
    extract::{Query, RawQuery, State},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

use crate::core::{
    operations::format_bytes,
    runtime::{filter_snapshots, FastSyncSnapshot, SnapshotFilter},
};
use crate::types::NodeConfig;

use super::super::{html, WebState};

#[derive(Default, Deserialize)]
pub struct SnapshotQuery {
    #[serde(default)]
    network: String,
    #[serde(default)]
    runtime: String,
    #[serde(default)]
    verified: String,
    #[serde(default)]
    cached: String,
    #[serde(default)]
    q: String,
}

pub async fn snapshots(
    State(state): State<WebState>,
    RawQuery(flash): RawQuery,
    Query(params): Query<SnapshotQuery>,
) -> Response {
    let (snapshots, nodes) = match (
        state.repository.list_fast_sync_snapshots(),
        state.repository.list_nodes(),
    ) {
        (Ok(snapshots), Ok(nodes)) => (snapshots, nodes),
        (Err(error), _) => {
            return Html(html::layout(
                "Snapshots",
                "snapshots",
                &html::flash(flash.as_deref()),
                &html::note(&format!("failed to load snapshots: {error}")),
            ))
            .into_response()
        }
        (_, Err(error)) => {
            return Html(html::layout(
                "Snapshots",
                "snapshots",
                &html::flash(flash.as_deref()),
                &html::note(&format!("failed to load nodes: {error}")),
            ))
            .into_response()
        }
    };

    let visible = filter_snapshots(&snapshots, &snapshot_filter(&params));
    let body = render_body(&snapshots, &visible, &nodes, &params);

    Html(html::layout(
        "Snapshots",
        "snapshots",
        &html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

/// An unparseable narrowing is simply no narrowing: the page should still show
/// the inventory rather than refuse to render.
fn snapshot_filter(params: &SnapshotQuery) -> SnapshotFilter {
    SnapshotFilter::new(
        params.network.trim().parse().ok(),
        params.runtime.trim().parse().ok(),
        tri_state(&params.verified),
        tri_state(&params.cached),
        params.q.trim(),
    )
}

fn tri_state(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" => Some(true),
        "no" | "false" => Some(false),
        _ => None,
    }
}

fn render_body(
    all: &[FastSyncSnapshot],
    visible: &[FastSyncSnapshot],
    nodes: &[NodeConfig],
    params: &SnapshotQuery,
) -> String {
    format!(
        r#"<h1>Snapshots</h1>
{tiles}
{register}
{filters}
{table}"#,
        tiles = html::cards(&[
            ("Known", all.len().to_string()),
            (
                "Cached",
                count(all, |snapshot| snapshot.cached_path.is_some())
            ),
            (
                "Verified",
                count(all, |snapshot| snapshot.verified_sha256.is_some()),
            ),
            ("Matching", visible.len().to_string()),
        ]),
        register = register_form(),
        filters = html::typed_filter_form(
            "/snapshots",
            &[],
            &[
                html::FilterControl::Text {
                    label: "Network",
                    name: "network",
                    value: &params.network,
                },
                html::FilterControl::Text {
                    label: "Runtime",
                    name: "runtime",
                    value: &params.runtime,
                },
                html::FilterControl::Select {
                    label: "Verification",
                    name: "verified",
                    selected: &params.verified,
                    options: &[
                        ("", "Any verification"),
                        ("yes", "Verified"),
                        ("no", "Not verified")
                    ],
                },
                html::FilterControl::Select {
                    label: "Cache",
                    name: "cached",
                    selected: &params.cached,
                    options: &[
                        ("", "Any cache state"),
                        ("yes", "Cached"),
                        ("no", "Not cached")
                    ],
                },
                html::FilterControl::Search {
                    label: "Search",
                    name: "q",
                    value: &params.q,
                    placeholder: "Snapshot, source, or checksum",
                },
            ],
        ),
        table = snapshot_table(visible, nodes),
    )
}

fn count(snapshots: &[FastSyncSnapshot], wanted: impl Fn(&FastSyncSnapshot) -> bool) -> String {
    snapshots
        .iter()
        .filter(|snapshot| wanted(snapshot))
        .count()
        .to_string()
}

fn snapshot_table(snapshots: &[FastSyncSnapshot], nodes: &[NodeConfig]) -> String {
    if snapshots.is_empty() {
        return html::note("No fast-sync snapshots are registered in this workspace.");
    }
    let rows = snapshots
        .iter()
        .map(|snapshot| {
            // Find nodes that can use this snapshot (matching node_type)
            let compatible_nodes: Vec<&NodeConfig> = nodes
                .iter()
                .filter(|node| node.node_type == snapshot.node_type)
                .collect();

            // Build apply buttons HTML for compatible nodes
            let apply_actions = if snapshot.verified_sha256.is_some()
                && snapshot.cached_path.is_some()
                && !compatible_nodes.is_empty()
            {
                let buttons: Vec<String> = compatible_nodes
                    .iter()
                    .map(|node| {
                        format!(
                        "<form method='POST' action='/snapshots/{}/apply/{}' style='display:inline'>
                            <button type='submit' class='btn btn-sm'>Apply to {}</button>
                        </form>",
                        html::urlencoding_lite(&snapshot.id),
                        html::urlencoding_lite(&node.id),
                        html::urlencoding_lite(&node.name)
                    )
                    })
                    .collect();
                format!("<div class='actions'>{}</div>", buttons.join(" "))
            } else if !compatible_nodes.is_empty() {
                "<span class='badge badge-warning'>Not ready</span>".to_string()
            } else {
                "<span class='badge badge-muted'>No compatible nodes</span>".to_string()
            };

            html::row(&[
                html::cell(&snapshot.label),
                html::cell(&snapshot.network.to_string()),
                html::cell(&snapshot.node_type.to_string()),
                html::cell(&snapshot.source_path.display().to_string()),
                html::cell(snapshot.source_url.as_deref().unwrap_or("local only")),
                html::cell(stage(snapshot)),
                html::cell(
                    &snapshot
                        .expected_sha256
                        .chars()
                        .take(12)
                        .collect::<String>(),
                ),
                html::cell(
                    &snapshot
                        .verified_sha256
                        .as_deref()
                        .map_or("—".to_string(), |hash| {
                            hash.chars().take(12).collect::<String>()
                        }),
                ),
                html::cell(&snapshot.bytes.map_or("—".to_string(), format_bytes)),
                html::raw_cell(&format!("{}{}", lifecycle_actions(snapshot), apply_actions)),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Label",
            "Network",
            "Runtime",
            "Source path",
            "Download",
            "Stage",
            "Expected",
            "Verified",
            "Size",
            "Actions",
        ],
        &rows,
    )
}

/// The most advanced stage a snapshot has reached. Order matters: a cached file
/// that has not been hashed is not the same as one that has.
fn stage(snapshot: &FastSyncSnapshot) -> &'static str {
    if snapshot.verified_sha256.is_some() {
        "verified"
    } else if snapshot.cached_path.is_some() {
        "cached"
    } else if snapshot.source_url.is_some() {
        "downloadable"
    } else {
        "declared"
    }
}

/// The stage-advancing controls for one row, shown only where they can do
/// something: a download needs an HTTPS source and an empty cache; a local cache
/// needs a source file and an empty cache; a verify needs a cached file whose
/// hash has not been recorded yet. This mirrors the readiness `stage()` reports.
fn lifecycle_actions(snapshot: &FastSyncSnapshot) -> String {
    let cached = snapshot.cached_path.is_some();
    let verified = snapshot.verified_sha256.is_some();
    let has_source_path = !snapshot.source_path.as_os_str().is_empty();

    let mut buttons: Vec<String> = Vec::new();
    if snapshot.source_url.is_some() && !cached {
        buttons.push(lifecycle_button(&snapshot.id, "download", "Download"));
    }
    if has_source_path && !cached {
        buttons.push(lifecycle_button(&snapshot.id, "cache", "Cache"));
    }
    if cached && !verified {
        buttons.push(lifecycle_button(&snapshot.id, "verify", "Verify"));
    }
    if buttons.is_empty() {
        return String::new();
    }
    format!("<div class='actions'>{}</div>", buttons.join(" "))
}

/// One stage-advancing button, styled like the apply buttons on the same row.
fn lifecycle_button(snapshot_id: &str, action: &str, label: &str) -> String {
    format!(
        "<form method='POST' action='/snapshots/{}/{action}' style='display:inline'>
            <button type='submit' class='btn btn-sm'>{label}</button>
        </form>",
        html::urlencoding_lite(snapshot_id),
    )
}

/// The browser entry point for registering a snapshot manifest. It posts to
/// `snapshot_ops::save_snapshot`, which stores the snapshot and journals it as
/// saved. A source path or an HTTPS URL is required, along with the checksum.
fn register_form() -> String {
    format!(
        r#"<h2>Register snapshot</h2>
<p class="muted">Record a fast-sync archive so it can be downloaded, cached, hash-verified and applied. Give it a source path or an HTTPS URL, and the expected SHA-256.</p>
<form class="filters" method="post" action="/snapshots/save">
{id}
{label}
{network}
{runtime}
{source_path}
{source_url}
{file_name}
{max_bytes}
{sha256}
<button type="submit">Save snapshot</button>
</form>"#,
        id = html::text_field("Snapshot id", "id", ""),
        label = html::text_field("Label", "label", ""),
        network = html::TextField {
            label: "Network",
            name: "network",
            placeholder: Some("mainnet"),
            ..html::TextField::default()
        }
        .render(),
        runtime = html::TextField {
            label: "Runtime",
            name: "node_type",
            placeholder: Some("neo-rs"),
            ..html::TextField::default()
        }
        .render(),
        source_path = html::TextField {
            label: "Source path",
            name: "source_path",
            placeholder: Some("/path/to/snapshot.acc"),
            monospace: true,
            ..html::TextField::default()
        }
        .render(),
        source_url = html::TextField {
            label: "Source URL",
            name: "source_url",
            placeholder: Some("https://…"),
            monospace: true,
            ..html::TextField::default()
        }
        .render(),
        file_name = html::text_field("Download file name", "download_file_name", ""),
        max_bytes = html::TextField {
            label: "Download size limit (bytes)",
            name: "download_max_bytes",
            placeholder: Some("optional"),
            ..html::TextField::default()
        }
        .render(),
        sha256 = html::TextField {
            label: "Expected SHA-256",
            name: "expected_sha256",
            monospace: true,
            full_width: true,
            ..html::TextField::default()
        }
        .render(),
    )
}
