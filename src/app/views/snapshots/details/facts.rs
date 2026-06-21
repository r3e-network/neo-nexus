use eframe::egui;

use crate::app::{
    domain::{format_bytes, FastSyncSnapshot},
    text::{short_path, truncate_middle},
    widgets::fact,
};

pub(super) fn render_snapshot_facts(ui: &mut egui::Ui, snapshot: &FastSyncSnapshot) {
    ui.columns(2, |columns| {
        render_source_facts(&mut columns[0], snapshot);
        render_integrity_facts(&mut columns[1], snapshot);
    });
}

fn render_source_facts(ui: &mut egui::Ui, snapshot: &FastSyncSnapshot) {
    fact(ui, "ID", &snapshot.id);
    fact(ui, "Runtime", &snapshot.node_type.to_string());
    fact(ui, "Network", &snapshot.network.to_string());
    fact(ui, "Source", &short_path(&snapshot.source_path, 58));
    fact(
        ui,
        "HTTPS",
        &snapshot
            .source_url
            .as_ref()
            .map_or_else(|| "-".to_string(), |url| truncate_middle(url, 58)),
    );
}

fn render_integrity_facts(ui: &mut egui::Ui, snapshot: &FastSyncSnapshot) {
    fact(
        ui,
        "Expected",
        &truncate_middle(&snapshot.expected_sha256, 40),
    );
    fact(
        ui,
        "Verified",
        &snapshot
            .verified_sha256
            .as_ref()
            .map_or_else(|| "-".to_string(), |value| truncate_middle(value, 40)),
    );
    fact(
        ui,
        "Bytes",
        &snapshot.bytes.map_or_else(|| "-".to_string(), format_bytes),
    );
    fact(
        ui,
        "Cached",
        &snapshot
            .cached_path
            .as_ref()
            .map_or_else(|| "-".to_string(), |path| short_path(path, 58)),
    );
}
