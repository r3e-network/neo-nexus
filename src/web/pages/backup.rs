//! Backup: workspace export archive covering all profiles, nodes, snapshots,
//! events — the complete state snapshot exported via CLI or the browser. The
//! page shows what exists and provides a button to generate a new archive.

use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse, Response},
};

use super::super::{html, WebState};

pub async fn backup_page(State(state): State<WebState>, RawQuery(flash): RawQuery) -> Response {
    let body = match render_body(&state) {
        Ok(body) => body,
        Err(error) => html::note(&format!("failed to load backup status: {error}")),
    };

    Html(html::layout(
        "Backup",
        "backup",
        &html::flash(flash.as_deref()),
        &body,
    ))
    .into_response()
}

fn render_body(state: &WebState) -> anyhow::Result<String> {
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
        r#"<h1>Workspace Backup</h1>
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
        export_status = if has_recent_export {
            html::notice(
                "",
                &format!(
                    "Export directory exists. Located at: {}",
                    export_dir.display()
                ),
            )
        } else {
            html::note("No export yet. Generate one using the button below.")
        },
        // Posts to the route that exists. This read "/backup/export", which is
        // registered nowhere, so the page's only button 404'd and a workspace
        // backup could not be taken from the console at all.
        actions = restore_and_export(),
    ))
}

/// The two directions a workspace archive travels.
///
/// Export posted to `/backup/export`, which is registered nowhere, so the
/// page's only button 404'd and no archive could be taken from the console at
/// all. Import had no route whatsoever, while the page promised the round trip
/// in prose.
fn restore_and_export() -> String {
    format!(
        r#"{export}
<div class="panel" style="margin-top: 18px; padding: 16px; border: 1px solid var(--line); border-radius: 8px;">
  <h3 style="margin-top: 0;">Restore from an archive</h3>
  <p class="muted" style="font-size: 12px;">
    Give the path to an archive <strong>on this host</strong>. An archive carries signer
    bindings, wallet profiles and the whole journal, so it is read from disk rather than
    pushed through the browser. Check it first if you are not sure what is in it — the check
    reads the archive and changes nothing.
  </p>
  <form method="post" action="/backup/import" style="display: flex; gap: 8px; flex-wrap: wrap; align-items: flex-end;">
    <label class="field" style="flex: 1 1 320px;"><span>Archive path</span><input name="archive_path" placeholder="/path/to/neonexus-backup-….json" required></label>
    <button type="submit" name="mode" value="validate" class="btn">Check it first</button>
    <button type="submit" name="mode" value="import" class="btn danger">Import and overwrite</button>
  </form>
</div>"#,
        export = html::control_form("/backup", &[], "Generate workspace backup"),
    )
}
