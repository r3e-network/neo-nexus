use eframe::egui;

use crate::metrics::format_bytes;

use super::super::super::super::{widgets::fact, NeoNexusApp};

struct RuntimeCatalogSummary {
    entries: usize,
    compatible: usize,
    generated: String,
    trust: &'static str,
    bytes: u64,
    fleet_ready: usize,
    fleet_blocked: usize,
    fleet_current: usize,
}

impl RuntimeCatalogSummary {
    fn from_app(app: &NeoNexusApp, entries: usize, compatible: usize, generated: String) -> Self {
        let fleet_plan = app.catalog_fleet_upgrade_plan();
        Self {
            entries,
            compatible,
            generated,
            trust: if app.runtime_catalog_signature_verified == Some(true) {
                "signed"
            } else {
                "local"
            },
            bytes: app.runtime_catalog_bytes,
            fleet_ready: fleet_plan.as_ref().map_or(0, |plan| plan.candidates.len()),
            fleet_blocked: fleet_plan.as_ref().map_or(0, |plan| plan.blocked_running),
            fleet_current: fleet_plan
                .as_ref()
                .map_or(0, |plan| plan.current_or_unavailable),
        }
    }
}

impl NeoNexusApp {
    pub(super) fn render_runtime_catalog_summary(
        &mut self,
        ui: &mut egui::Ui,
        entries: usize,
        compatible: usize,
        generated: String,
    ) {
        let summary = RuntimeCatalogSummary::from_app(self, entries, compatible, generated);

        ui.columns(2, |columns| {
            fact(&mut columns[0], "Entries", &summary.entries.to_string());
            fact(&mut columns[0], "Host", &summary.compatible.to_string());
            fact(&mut columns[1], "Trust", summary.trust);
            fact(&mut columns[1], "Size", &format_bytes(summary.bytes));
        });
        fact(ui, "Generated", &summary.generated);
        ui.columns(3, |columns| {
            fact(
                &mut columns[0],
                "Fleet ready",
                &summary.fleet_ready.to_string(),
            );
            fact(
                &mut columns[1],
                "Blocked",
                &summary.fleet_blocked.to_string(),
            );
            fact(
                &mut columns[2],
                "Current",
                &summary.fleet_current.to_string(),
            );
        });
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    summary.fleet_ready > 0,
                    egui::Button::new("Upgrade Stopped Fleet"),
                )
                .clicked()
            {
                self.upgrade_stopped_nodes_from_catalog();
            }
        });
    }
}
