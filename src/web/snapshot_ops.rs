//! Snapshot lifecycle controls from the browser.
//!
//! Each handler drives one stage of a fast-sync snapshot — register, download,
//! cache, verify — through the same `FastSyncSnapshotManager` the CLI uses, then
//! journals the transition so the Events page can tell the story afterwards. The
//! apply stage itself lives in [`super::control`] beside the other node
//! controls; everything else that changes a snapshot's stage lives here so the
//! new producers stay isolated from the rest of the control surface.
//!
//! Downloads can take minutes, so that one handler hands its blocking work to
//! [`tokio::task::spawn_blocking`] rather than holding the request thread. The
//! others are quick local filesystem work and stay inline.

use axum::{
    extract::{Form, Path, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    core::operations::format_bytes,
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    snapshots::{FastSyncSnapshot, FastSyncSnapshotManager},
};

use super::{
    form_state::{FieldErrors, GENERAL},
    html, pages, snapshot_form,
    snapshot_form::SnapshotDraft,
    WebState,
};

/// The workspace child the manager caches downloaded and copied archives into,
/// mirroring the `runtime-downloads` convention the runtime installer uses.
const CACHE_DIR: &str = "snapshot-cache";

/// Hash-verify a snapshot's source archive and record the recomputed digest.
///
/// A mismatch is refused rather than persisted, so a corrupted archive can never
/// masquerade as verified on the inventory page.
pub async fn verify_snapshot(
    State(state): State<WebState>,
    Path(snapshot_id): Path<String>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let snapshot = load_snapshot(&state, &snapshot_id)?;
        let verification = FastSyncSnapshotManager::verify(&snapshot)
            .map_err(|error| anyhow::anyhow!("verification failed: {error}"))?;
        if !verification.matches {
            anyhow::bail!(
                "snapshot '{}' failed verification: expected {}, got {}",
                snapshot.label,
                short_hash(&verification.expected_sha256),
                short_hash(&verification.sha256),
            );
        }
        state
            .commands
            .mark_fast_sync_snapshot_verified(&snapshot.id, &verification)?;
        let message = format!(
            "snapshot '{}' verified — sha256 {} ({})",
            snapshot.label,
            short_hash(&verification.sha256),
            format_bytes(verification.bytes),
        );
        record(&state, EventKind::SnapshotVerified, message.clone());
        Ok(message)
    })();
    respond(outcome)
}

/// Download a snapshot over HTTPS into the workspace cache.
///
/// The blocking fetch runs on a dedicated thread so a multi-minute download does
/// not tie up the async request. A finished download is both a download and a
/// cache write, so it journals both events.
pub async fn download_snapshot(
    State(state): State<WebState>,
    Path(snapshot_id): Path<String>,
) -> Response {
    let outcome =
        match tokio::task::spawn_blocking(move || download_blocking(&state, &snapshot_id)).await {
            Ok(inner) => inner,
            Err(joined) => Err(anyhow::anyhow!("download task did not finish: {joined}")),
        };
    respond(outcome)
}

/// The blocking body of a download: fetch, verify against the expected checksum
/// inside the manager, persist the cache state, then journal it.
fn download_blocking(state: &WebState, snapshot_id: &str) -> anyhow::Result<String> {
    let snapshot = load_snapshot(state, snapshot_id)?;
    let request = snapshot.download_request()?;
    let cache =
        FastSyncSnapshotManager::download_https(&request, state.workspace_child_dir(CACHE_DIR))
            .map_err(|error| anyhow::anyhow!("download failed: {error}"))?;
    state
        .commands
        .mark_fast_sync_snapshot_cached(&snapshot.id, &cache)?;
    let message = format!(
        "snapshot '{}' downloaded and cached — {} ({})",
        snapshot.label,
        cache.path.display(),
        format_bytes(cache.bytes),
    );
    record(state, EventKind::SnapshotDownloaded, message.clone());
    record(state, EventKind::SnapshotCached, message.clone());
    Ok(message)
}

/// Copy a local snapshot archive into the workspace cache, verifying its
/// checksum as it goes. This is the offline counterpart to a download: it is how
/// a snapshot with only a `source_path` reaches the cached stage.
pub async fn cache_snapshot(
    State(state): State<WebState>,
    Path(snapshot_id): Path<String>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let snapshot = load_snapshot(&state, &snapshot_id)?;
        let cache = FastSyncSnapshotManager::cache(&snapshot, state.workspace_child_dir(CACHE_DIR))
            .map_err(|error| anyhow::anyhow!("cache failed: {error}"))?;
        state
            .commands
            .mark_fast_sync_snapshot_cached(&snapshot.id, &cache)?;
        let message = format!(
            "snapshot '{}' cached — {} ({})",
            snapshot.label,
            cache.path.display(),
            format_bytes(cache.bytes),
        );
        record(&state, EventKind::SnapshotCached, message.clone());
        Ok(message)
    })();
    respond(outcome)
}

/// Register or update a snapshot manifest, then journal it as saved.
///
/// This is the one genuine producer of [`EventKind::SnapshotSaved`]: the page's
/// register form posts here, so the event is only ever recorded for a snapshot a
/// real submission created.
///
/// A manifest this handler cannot turn into a snapshot — a digest of the wrong
/// length, a size limit that is not a number — comes back as the form itself,
/// with the posted text still in it and the sentence beside the control that
/// caused it. The alternative was a redirect: nine fields, one typo, retype all
/// nine.
pub async fn save_snapshot(
    State(state): State<WebState>,
    Form(draft): Form<SnapshotDraft>,
) -> Response {
    let snapshot = match snapshot_form::build(&draft) {
        Ok(snapshot) => snapshot,
        Err(refusal) => return redraw(&state, &draft, refusal.field, &refusal.message),
    };
    let outcome = state
        .commands
        .upsert_fast_sync_snapshot(snapshot)
        .map(|snapshot| {
            let message = format!("snapshot '{}' saved", snapshot.label);
            record(&state, EventKind::SnapshotSaved, message.clone());
            message
        });
    match outcome {
        Ok(message) => respond(Ok(message)),
        // The workspace is the authority on what a manifest may be. Its own
        // sentence goes above the form: the checks the form can pin to a control
        // already ran, so whatever is left belongs to no single field.
        Err(error) => redraw(&state, &draft, GENERAL, &format!("{error:#}")),
    }
}

/// The form again, with what was posted and the reason it was refused.
fn redraw(state: &WebState, draft: &SnapshotDraft, field: &'static str, message: &str) -> Response {
    let mut errors = FieldErrors::new();
    errors.insert(field, message.to_string());
    match pages::snapshots::render_with_draft(&state.workspace, draft, &errors) {
        Ok(body) => Html(html::layout(
            "Snapshots",
            "snapshots",
            html::Flash::default(),
            &body,
        ))
        .into_response(),
        // Redrawing needs the inventory this page was built from, which is a
        // second read of the workspace. If that read fails too, the input cannot
        // be kept — say so, rather than landing on a blank form with no reason.
        Err(error) => {
            let flash = html::Flash::danger(format!(
                "not saved: {message} — and the page could not be redrawn ({error}), so the form \
                 was lost"
            ));
            Redirect::to(&flash.redirect("/snapshots")).into_response()
        }
    }
}

/// Resolve a snapshot by id, or say plainly that it is not registered.
fn load_snapshot(state: &WebState, snapshot_id: &str) -> anyhow::Result<FastSyncSnapshot> {
    state
        .workspace
        .list_fast_sync_snapshots()?
        .into_iter()
        .find(|snapshot| snapshot.id == snapshot_id)
        .ok_or_else(|| anyhow::anyhow!("snapshot {snapshot_id} was not found"))
}

/// The first twelve characters of a digest, matching how the page abbreviates
/// hashes elsewhere.
fn short_hash(hash: &str) -> String {
    hash.chars().take(12).collect()
}

/// Journal a snapshot stage transition. The change already happened, so a failed
/// write must not be reported as if the operation itself had failed.
fn record(state: &WebState, kind: EventKind, message: String) {
    let _ = state.commands.record_event(NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind,
        severity: EventSeverity::Info,
        message,
    });
}

/// The shared tail of every snapshot control: describe the outcome and send the
/// browser back to the inventory page.
fn respond(outcome: anyhow::Result<String>) -> Response {
    let flash = match outcome {
        Ok(message) => html::Flash::ok(message),
        Err(error) => html::Flash::danger(format!("failed: {error}")),
    };
    Redirect::to(&flash.redirect("/snapshots")).into_response()
}
