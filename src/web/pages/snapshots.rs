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
    let body = match state.repository.list_fast_sync_snapshots() {
        Ok(snapshots) => {
            let visible = filter_snapshots(&snapshots, &snapshot_filter(&params));
            render_body(&snapshots, &visible, &params)
        }
        Err(error) => html::note(&format!("failed to load snapshots: {error}")),
    };
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
    params: &SnapshotQuery,
) -> String {
    format!(
        r#"<h1>Snapshots</h1>
{tiles}
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
        filters = html::filter_form(
            "/snapshots",
            &[
                ("network", &params.network),
                ("runtime", &params.runtime),
                ("verified", &params.verified),
                ("cached", &params.cached),
                ("q", &params.q),
            ],
        ),
        table = snapshot_table(visible),
    )
}

fn count(snapshots: &[FastSyncSnapshot], wanted: impl Fn(&FastSyncSnapshot) -> bool) -> String {
    snapshots
        .iter()
        .filter(|snapshot| wanted(snapshot))
        .count()
        .to_string()
}

fn snapshot_table(snapshots: &[FastSyncSnapshot]) -> String {
    if snapshots.is_empty() {
        return html::note("No fast-sync snapshots are registered in this workspace.");
    }
    let rows = snapshots
        .iter()
        .map(|snapshot| {
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
