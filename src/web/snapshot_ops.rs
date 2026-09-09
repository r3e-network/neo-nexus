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

use std::path::PathBuf;

use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    core::operations::format_bytes,
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    snapshots::{FastSyncSnapshot, FastSyncSnapshotManager, NewFastSyncSnapshot},
    types::{Network, NodeType},
};

use super::{html, WebState};

/// The workspace child the manager caches downloaded and copied archives into,
/// mirroring the `runtime-downloads` convention the runtime installer uses.
const CACHE_DIR: &str = "snapshot-cache";

/// A blank size limit on the register form means "use a generous default"
/// rather than refusing the save: the field guards runaway downloads, and 8 GiB
/// comfortably covers a mainnet fast-sync archive.
const DEFAULT_DOWNLOAD_MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;

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
            .repository
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
        .repository
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
            .repository
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

/// The register form: every field as the text the browser posted. The domain's
/// own `validate_snapshot_input` (inside `upsert_fast_sync_snapshot`) is the sole
/// authority on what is acceptable, so this struct stays a plain carrier.
#[derive(Deserialize)]
pub struct SaveSnapshotForm {
    #[serde(default)]
    id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    network: String,
    #[serde(default)]
    node_type: String,
    #[serde(default)]
    source_path: String,
    #[serde(default)]
    source_url: String,
    #[serde(default)]
    download_file_name: String,
    #[serde(default)]
    download_max_bytes: String,
    #[serde(default)]
    expected_sha256: String,
}

/// Register or update a snapshot manifest, then journal it as saved.
///
/// This is the one genuine producer of [`EventKind::SnapshotSaved`]: the page's
/// register form posts here, so the event is only ever recorded for a snapshot a
/// real submission created.
pub async fn save_snapshot(
    State(state): State<WebState>,
    Form(input): Form<SaveSnapshotForm>,
) -> Response {
    let outcome = (|| -> anyhow::Result<String> {
        let network: Network = input
            .network
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("{} is not a network", input.network.trim()))?;
        let node_type: NodeType =
            input.node_type.trim().parse().map_err(|_| {
                anyhow::anyhow!("{} is not a supported client", input.node_type.trim())
            })?;
        let snapshot = state
            .repository
            .upsert_fast_sync_snapshot(NewFastSyncSnapshot {
                id: input.id.trim().to_string(),
                label: input.label.trim().to_string(),
                network,
                node_type,
                source_path: PathBuf::from(input.source_path.trim()),
                source_url: optional(&input.source_url),
                download_file_name: optional(&input.download_file_name),
                download_max_bytes: parse_max_bytes(&input.download_max_bytes)?,
                expected_sha256: input.expected_sha256.trim().to_string(),
            })?;
        let message = format!("snapshot '{}' saved", snapshot.label);
        record(&state, EventKind::SnapshotSaved, message.clone());
        Ok(message)
    })();
    respond(outcome)
}

/// Resolve a snapshot by id, or say plainly that it is not registered.
fn load_snapshot(state: &WebState, snapshot_id: &str) -> anyhow::Result<FastSyncSnapshot> {
    state
        .repository
        .list_fast_sync_snapshots()?
        .into_iter()
        .find(|snapshot| snapshot.id == snapshot_id)
        .ok_or_else(|| anyhow::anyhow!("snapshot {snapshot_id} was not found"))
}

/// Trim a form field to `None` when it is blank, `Some` otherwise.
fn optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// A blank size limit falls back to the default; anything else must be a whole
/// number so a hand-edited post cannot silently save nonsense.
fn parse_max_bytes(raw: &str) -> anyhow::Result<u64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(DEFAULT_DOWNLOAD_MAX_BYTES);
    }
    trimmed
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("download size limit must be a whole number of bytes"))
}

/// The first twelve characters of a digest, matching how the page abbreviates
/// hashes elsewhere.
fn short_hash(hash: &str) -> String {
    hash.chars().take(12).collect()
}

/// Journal a snapshot stage transition. The change already happened, so a failed
/// write must not be reported as if the operation itself had failed.
fn record(state: &WebState, kind: EventKind, message: String) {
    let _ = state.repository.record_event(NewRuntimeEvent {
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
    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("failed: {error}"),
    };
    Redirect::to(&format!(
        "/snapshots?flash={}",
        html::urlencoding_lite(&message)
    ))
    .into_response()
}
