use eframe::egui;

use crate::app::{
    theme::muted_text, NeoNexusApp, REMOTE_PROBE_RETAIN_PER_PROFILE, RPC_HEALTH_RETAIN_PER_NODE,
};

pub(super) fn render_monitor_retention(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("Prune RPC").clicked() {
            app.prune_rpc_health_history();
        }
        if ui.button("Prune Federation").clicked() {
            app.prune_remote_federation_history();
        }
    });
    ui.label(
        egui::RichText::new(format!(
            "Retain {RPC_HEALTH_RETAIN_PER_NODE} RPC checks per node and {REMOTE_PROBE_RETAIN_PER_PROFILE} Federation probes per profile."
        ))
        .color(muted_text()),
    );
}
