use eframe::egui;

use crate::{app::NeoNexusApp, snapshots::FastSyncSnapshot, types::NodeConfig};

use super::super::status::snapshot_is_verified;

const BYTES_PER_MIB: u64 = 1024 * 1024;

pub(super) fn render_snapshot_actions(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    snapshot: &FastSyncSnapshot,
) {
    ui.horizontal(|ui| {
        if ui.button("Verify SHA-256").clicked() {
            app.verify_selected_snapshot();
        }
        if ui
            .add_enabled(
                snapshot.source_url.is_some(),
                egui::Button::new("Download HTTPS"),
            )
            .clicked()
        {
            app.download_selected_snapshot();
        }
        if ui.button("Cache Local").clicked() {
            app.cache_selected_snapshot();
        }
        if ui
            .add_enabled(
                can_apply_snapshot(app, snapshot),
                egui::Button::new("Import to Node"),
            )
            .clicked()
        {
            app.apply_selected_snapshot_to_node();
        }
        if ui.button("Load Manifest").clicked() {
            load_snapshot_into_draft(app, snapshot);
        }
    });
}

fn can_apply_snapshot(app: &NeoNexusApp, snapshot: &FastSyncSnapshot) -> bool {
    app.selected_node()
        .is_some_and(|node| target_accepts_snapshot(node, snapshot))
}

fn target_accepts_snapshot(node: &NodeConfig, snapshot: &FastSyncSnapshot) -> bool {
    node.status.is_stopped()
        && node.network == snapshot.network
        && node.node_type == snapshot.node_type
        && snapshot.cached_path.is_some()
        && snapshot_is_verified(snapshot)
}

fn load_snapshot_into_draft(app: &mut NeoNexusApp, snapshot: &FastSyncSnapshot) {
    app.snapshot_draft.id = snapshot.id.clone();
    app.snapshot_draft.label = snapshot.label.clone();
    app.snapshot_draft.network = snapshot.network;
    app.snapshot_draft.node_type = snapshot.node_type;
    app.snapshot_draft.source_path = snapshot.source_path.display().to_string();
    app.snapshot_draft.source_url = snapshot.source_url.clone().unwrap_or_default();
    app.snapshot_draft.download_file_name = snapshot.download_file_name.clone().unwrap_or_default();
    app.snapshot_draft.download_max_mib = snapshot
        .download_max_bytes
        .saturating_add(BYTES_PER_MIB - 1)
        / BYTES_PER_MIB;
    app.snapshot_draft.expected_sha256 = snapshot.expected_sha256.clone();
    app.notice = Some(format!("{} loaded into snapshot draft", snapshot.label));
}
