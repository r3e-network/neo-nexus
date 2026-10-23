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
use crate::types::{Network, NodeConfig, NodeType};

use super::super::{
    form_state::{self, FieldErrors, FormDraft},
    html,
    snapshot_form::{self, SnapshotDraft},
    WebState,
};

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
        state.workspace.list_fast_sync_snapshots(),
        state.workspace.list_nodes(),
    ) {
        (Ok(snapshots), Ok(nodes)) => (snapshots, nodes),
        (Err(error), _) => {
            return Html(html::layout(
                "Snapshots",
                "snapshots",
                html::flash(flash.as_deref()),
                &html::note(&format!("failed to load snapshots: {error}")),
            ))
            .into_response()
        }
        (_, Err(error)) => {
            return Html(html::layout(
                "Snapshots",
                "snapshots",
                html::flash(flash.as_deref()),
                &html::note(&format!("failed to load nodes: {error}")),
            ))
            .into_response()
        }
    };

    let visible = filter_snapshots(&snapshots, &snapshot_filter(&params));
    let body = render_body(&snapshots, &visible, &nodes, &params, None);

    Html(html::layout(
        "Snapshots",
        "snapshots",
        html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

/// The same page, drawn from what the operator posted instead of from blank
/// fields.
///
/// A refused register form comes back here: the inventory above is still what
/// the workspace holds, and the form below is what was typed, with each
/// complaint beside the control it is about. That is the whole point of a
/// redraw — the two are allowed to disagree, and the page says which is which.
pub(crate) fn render_with_draft(
    workspace: &crate::core::workspace_queries::WorkspaceQueries,
    draft: &SnapshotDraft,
    errors: &FieldErrors,
) -> anyhow::Result<String> {
    let snapshots = workspace.list_fast_sync_snapshots()?;
    let nodes = workspace.list_nodes()?;
    // A redraw has no query string, so the inventory is unfiltered. Narrowing is
    // a view concern; losing it costs a click, while losing the form costs nine
    // fields.
    let params = SnapshotQuery::default();
    let visible = filter_snapshots(&snapshots, &snapshot_filter(&params));
    Ok(render_body(
        &snapshots,
        &visible,
        &nodes,
        &params,
        Some((draft, errors)),
    ))
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
    refused: Option<(&SnapshotDraft, &FieldErrors)>,
) -> String {
    // "EBS Snapshots" named the wrong direction entirely: these are *inbound*
    // archives a node fast-syncs from, not a capture of this workspace's state.
    let breadcrumb = html::breadcrumb(&[("NeoNexus", "/"), ("Fast-sync archives", "")]);
    let head = html::page_head(
        "EBS Snapshots & Fast-Sync",
        "Point-in-time cryptographic block volume checkpoints, SHA-256 integrity verifications, and one-click instance attachments.",
        r#"<a class="btn" href="/config">⚙️ Volumes Config</a> <a class="btn primary" href="/nodes">⬡ EC2 Instances</a>"#,
    );
    format!(
        r#"{breadcrumb}
{head}
{tiles}
{register}
{filters}
<h2>EBS Snapshot Inventory</h2>
{table}"#,
        breadcrumb = breadcrumb,
        head = head,
        tiles = html::cards(&[
            ("Known Snapshots", all.len().to_string()),
            (
                "Cached on Disk",
                count(all, |snapshot| snapshot.cached_path.is_some())
            ),
            (
                "Verified (SHA-256)",
                count(all, |snapshot| snapshot.verified_sha256.is_some()),
            ),
            ("Matching", visible.len().to_string()),
        ]),
        register = register_form(refused),
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
        return html::empty_state(
            "No snapshots registered",
            "No fast-sync snapshot is registered in this workspace. Register one to attach it to \
             a compatible instance.",
            "",
        );
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
                            // Visible text, not a URL: percent-encoding it
                            // rendered a node named "node 01" as "node%2001".
                            html::escape(&node.name)
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
///
/// Every control asks [`SnapshotDraft`] for its own value by the same field
/// constant it hands the browser as `name`, so a value cannot be drawn under
/// another field's label.
fn register_form(refused: Option<(&SnapshotDraft, &FieldErrors)>) -> String {
    let (values, errors) = match refused {
        Some((values, errors)) => (Some(values), Some(errors)),
        None => (None, None),
    };
    let value = |field: &str| values.map_or("", |draft| draft.value(field));
    let error = |field: &str| errors.and_then(|errors| form_state::error_for(errors, field));
    // A first visit shows the common answer rather than an empty select; a
    // redraw shows what was posted, because that is what has to be corrected.
    let network_default = Network::Mainnet.to_string();
    let runtime_default = NodeType::ALL[0].to_string();
    let network = match values {
        Some(draft) => draft.value(snapshot_form::NETWORK),
        None => network_default.as_str(),
    };
    let runtime = match values {
        Some(draft) => draft.value(snapshot_form::NODE_TYPE),
        None => runtime_default.as_str(),
    };
    let summary = match errors {
        Some(errors) if !errors.is_empty() => format!(
            "{}{}",
            form_state::banner(errors),
            html::note(
                "Nothing was saved. The inventory above is still what this workspace holds; \
                 what is in the form is what you typed and what it would not accept."
            ),
        ),
        _ => String::new(),
    };
    format!(
        r#"<h2>Register snapshot</h2>
<p class="muted">Record a fast-sync archive so it can be downloaded, cached, hash-verified and applied. Give it a source path or an HTTPS URL, and the expected SHA-256.</p>
{summary}
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
        summary = summary,
        id = html::TextField {
            label: "Snapshot id",
            name: snapshot_form::ID,
            value: value(snapshot_form::ID),
            required: true,
            error: error(snapshot_form::ID),
            ..html::TextField::default()
        }
        .render(),
        label = html::TextField {
            label: "Label",
            name: snapshot_form::LABEL,
            value: value(snapshot_form::LABEL),
            required: true,
            error: error(snapshot_form::LABEL),
            ..html::TextField::default()
        }
        .render(),
        // Both are closed sets the save handler parses and rejects on a typo,
        // so they are chosen rather than typed. The node editor has always
        // offered them this way; this form asked the operator to remember the
        // exact spelling and told them afterwards when it was wrong.
        network = html::ChoiceField {
            label: "Network",
            name: snapshot_form::NETWORK,
            options: &Network::ALL
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            selected: network,
            error: error(snapshot_form::NETWORK),
            ..html::ChoiceField::default()
        }
        .render(),
        runtime = html::ChoiceField {
            label: "Runtime",
            name: snapshot_form::NODE_TYPE,
            options: &NodeType::ALL
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            selected: runtime,
            error: error(snapshot_form::NODE_TYPE),
            ..html::ChoiceField::default()
        }
        .render(),
        source_path = html::TextField {
            label: "Source path",
            name: snapshot_form::SOURCE_PATH,
            value: value(snapshot_form::SOURCE_PATH),
            placeholder: Some("/path/to/snapshot.acc"),
            monospace: true,
            error: error(snapshot_form::SOURCE_PATH),
            ..html::TextField::default()
        }
        .render(),
        source_url = html::TextField {
            label: "Source URL",
            name: snapshot_form::SOURCE_URL,
            value: value(snapshot_form::SOURCE_URL),
            placeholder: Some("https://…"),
            monospace: true,
            error: error(snapshot_form::SOURCE_URL),
            ..html::TextField::default()
        }
        .render(),
        file_name = html::TextField {
            label: "Download file name",
            name: snapshot_form::DOWNLOAD_FILE_NAME,
            value: value(snapshot_form::DOWNLOAD_FILE_NAME),
            error: error(snapshot_form::DOWNLOAD_FILE_NAME),
            ..html::TextField::default()
        }
        .render(),
        max_bytes = html::TextField {
            label: "Download size limit (bytes)",
            name: snapshot_form::DOWNLOAD_MAX_BYTES,
            value: value(snapshot_form::DOWNLOAD_MAX_BYTES),
            placeholder: Some("optional"),
            // A limit of zero would refuse every download, so the domain
            // rejects it outright. The field stays optional: blank means the
            // default limit.
            input_type: html::TextInputType::Number,
            min: Some("1"),
            step: Some("1"),
            error: error(snapshot_form::DOWNLOAD_MAX_BYTES),
            ..html::TextField::default()
        }
        .render(),
        sha256 = html::TextField {
            label: "Expected SHA-256",
            name: snapshot_form::EXPECTED_SHA256,
            value: value(snapshot_form::EXPECTED_SHA256),
            monospace: true,
            full_width: true,
            // Exactly 64 hexadecimal characters, lower or upper case. The
            // domain lowercases before checking, so both are accepted.
            pattern: Some("[0-9a-fA-F]{64}"),
            maxlength: Some("64"),
            error: error(snapshot_form::EXPECTED_SHA256),
            ..html::TextField::default()
        }
        .render(),
    )
}
