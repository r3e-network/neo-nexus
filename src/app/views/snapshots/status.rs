use eframe::egui;

use crate::{snapshots::FastSyncSnapshot, types::NodeStatus};

use super::super::super::theme::{accent, muted_text, status_color};

pub(super) fn snapshot_is_verified(snapshot: &FastSyncSnapshot) -> bool {
    snapshot
        .verified_sha256
        .as_ref()
        .is_some_and(|sha256| sha256 == &snapshot.expected_sha256)
}

pub(super) fn status_label(snapshot: &FastSyncSnapshot) -> &'static str {
    if snapshot.cached_path.is_some() && snapshot_is_verified(snapshot) {
        "cached"
    } else if snapshot_is_verified(snapshot) {
        "verified"
    } else if snapshot.verified_sha256.is_some() {
        "mismatch"
    } else {
        "pending"
    }
}

pub(super) fn snapshot_status_color(snapshot: &FastSyncSnapshot) -> egui::Color32 {
    match status_label(snapshot) {
        "cached" | "verified" => accent(),
        "mismatch" => status_color(NodeStatus::Error),
        _ => muted_text(),
    }
}
