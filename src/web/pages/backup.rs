//! Backup: workspace export archive covering all profiles, nodes, snapshots,
//! events — the complete state snapshot exported via CLI or the browser. The
//! page shows what exists and provides a button to generate a new archive.

use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse, Response},
};

use super::super::{
    form_state::{self, FieldErrors},
    html, WebState,
};

/// The one field the restore panel carries, named here so the page that draws
/// it and the handler that blames it cannot disagree.
pub const ARCHIVE_PATH: &str = "archive_path";

pub async fn backup_page(State(state): State<WebState>, RawQuery(flash): RawQuery) -> Response {
    let body = match render_body(&state, None) {
        Ok(body) => body,
        Err(error) => html::note(&format!("failed to load backup status: {error}")),
    };

    Html(html::layout(
        "Backup",
        "backup",
        html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

/// The page again, with the archive path the operator typed and the reason the
/// restore refused it.
///
/// Restore is the one control here that can destroy what it is run against, so
/// the refusal has to be readable next to the path it is about: a path typed on
/// a server is long, and retyping it to read one sentence is how operators end
/// up pasting the wrong archive.
pub(crate) fn render_with_restore_draft(
    state: &WebState,
    path: &str,
    errors: &FieldErrors,
) -> anyhow::Result<String> {
    render_body(state, Some((path, errors)))
}

fn render_body(state: &WebState, restore: Option<(&str, &FieldErrors)>) -> anyhow::Result<String> {
    let nodes = state.workspace.list_nodes()?;
    let snapshots = state.workspace.list_fast_sync_snapshots()?;

    // Count different profile types from custody profiles iterator
    let mut signer_backends = 0;

    for profile in state.custody().profiles() {
        match profile.kind {
            crate::signing::SignerBackendKind::LocalWallet => signer_backends += 1,
            crate::signing::SignerBackendKind::LocalSigner => signer_backends += 1,
            crate::signing::SignerBackendKind::NeoOsService => signer_backends += 1,
        }
    }

    // Events count from operation log (recent 100)
    let event_count = state.workspace.list_recent_events(100)?.len();

    // Recent export attempt directory
    let export_dir = state.workspace_child_dir("export").join("backup");
    let has_recent_export = export_dir.is_dir();

    Ok(format!(
        r#"<h1>Workspace backup</h1>
{tiles}
{description}
{export_status}
{actions}"#,
        tiles = html::cards(&[
            ("Nodes", nodes.len().to_string()),
            ("Signer Profiles", signer_backends.to_string()),
            ("Snapshots", snapshots.len().to_string()),
            ("Events", event_count.to_string()),
        ]),
        description = html::notice(
            "info",
            "A workspace backup captures every artifact your NeoNexus instance manages \
             — signer profiles, wallet profiles, node configurations, fast-sync snapshots, \
             operational policies, and audit events. Export archives can be imported into \
             another workspace to clone the entire setup.",
        ),
        // The export button moves into the empty state when there is nothing
        // to show: an empty state whose only way forward lives somewhere else
        // on the page is a dead end the operator has to go looking for.
        export_status = if has_recent_export {
            html::notice(
                "",
                &format!(
                    "Export directory exists. Located at: {}",
                    export_dir.display()
                ),
            )
        } else {
            html::empty_state(
                "No export yet",
                "No workspace backup archive has been generated yet.",
                &html::pending_control_form(
                    "/backup",
                    &[],
                    "Generate workspace backup",
                    "Collecting every artifact…",
                ),
            )
        },
        // Posts to the route that exists. This read "/backup/export", which is
        // registered nowhere, so the page's only button 404'd and a workspace
        // backup could not be taken from the console at all.
        actions = if has_recent_export {
            format!(
                "{}\n{}",
                html::pending_control_form(
                    "/backup",
                    &[],
                    "Generate workspace backup",
                    "Collecting every artifact…",
                ),
                restore_panel(restore)
            )
        } else {
            restore_panel(restore)
        },
    ))
}

/// The restore half of the round trip.
///
/// Export posted to `/backup/export`, which is registered nowhere, so the
/// page's only button 404'd and no archive could be taken from the console at
/// all. Import had no route whatsoever, while the page promised the round trip
/// in prose.
fn restore_panel(restore: Option<(&str, &FieldErrors)>) -> String {
    let (path, error) = match restore {
        Some((path, errors)) => (path, form_state::error_for(errors, ARCHIVE_PATH)),
        None => ("", None),
    };
    // A refusal says so above the controls, because the sentence that matters is
    // about the archive, and the path beside it is what has to change.
    let summary = match restore {
        Some((_, errors)) if !errors.is_empty() => {
            format!(
                "{}{}",
                form_state::banner(errors),
                html::note(
                    "Nothing was imported. The workspace is exactly as it was; what is in the \
                 field is what you typed and what it would not accept."
                )
            )
        }
        _ => String::new(),
    };
    format!(
        r#"<div class="panel" style="margin-top: 18px; padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
  <h3 style="margin-top: 0;">Restore from an archive</h3>
  <p class="muted" style="font-size: 12px;">
    Give the path to an archive <strong>on this host</strong>. An archive carries signer
    bindings, wallet profiles and the whole journal, so it is read from disk rather than
    pushed through the browser. Check it first if you are not sure what is in it — the check
    reads the archive and changes nothing.
  </p>
  {summary}
  <form method="post" action="/backup/import" style="display: flex; gap: 8px; flex-wrap: wrap; align-items: flex-end;">
    {path_field}
    <button type="submit" name="mode" value="validate" class="btn">Check it first</button>
    <button type="submit" name="mode" value="import" class="btn danger">Import and overwrite</button>
    {pending}
  </form>
</div>"#,
        summary = summary,
        path_field = html::TextField {
            label: "Archive path",
            name: ARCHIVE_PATH,
            value: path,
            placeholder: Some("/path/to/neonexus-backup-….json"),
            monospace: true,
            required: true,
            full_width: true,
            error,
            ..html::TextField::default()
        }
        .render(),
        // Reading and checking an archive takes as long as the archive is
        // large, and so does overwriting the workspace with one.
        pending = html::pending_marker("Reading the archive…"),
    )
}
