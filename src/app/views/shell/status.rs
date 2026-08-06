use eframe::egui;

use super::super::super::{render_toast_strip, text::short_path, theme, widgets::vr, NeoNexusApp};

/// Height of the divider rules between status pairs. Deliberately shorter than
/// the 28pt bar so the rules read as light separators rather than as a grid.
const DIVIDER_HEIGHT: f32 = 14.0;

/// Vertical size each item in the bar is allowed to claim. The bar is a fixed
/// 28pt panel with 4pt margins, leaving 20pt of content: at the default 28pt
/// `interact_size` every chip and separator claimed more room than the panel
/// has and the bar painted ~10pt outside its own bounds.
const ITEM_HEIGHT: f32 = 18.0;

impl NeoNexusApp {
    pub(in crate::app) fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.spacing_mut().interact_size.y = ITEM_HEIGHT;
        ui.spacing_mut().item_spacing.x = theme::SM;
        ui.horizontal(|ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            let running = self.running_node_count();
            let system = &self.metrics_snapshot.system;
            status_chip(ui, "Nodes", &self.fleet.nodes.len().to_string(), None);
            vr(ui, DIVIDER_HEIGHT);
            status_chip(
                ui,
                "Running",
                &running.to_string(),
                (running > 0).then(theme::success),
            );
            vr(ui, DIVIDER_HEIGHT);
            status_chip(
                ui,
                "Wallets",
                &self.neo_wallet_profiles.len().to_string(),
                None,
            );
            vr(ui, DIVIDER_HEIGHT);
            status_chip(
                ui,
                "CPU",
                &format!("{:.0}%", system.cpu_usage_percent),
                pressure_tint(system.cpu_usage_percent),
            );
            vr(ui, DIVIDER_HEIGHT);
            status_chip(
                ui,
                "Mem",
                &format!("{:.0}%", system.memory_usage_percent),
                pressure_tint(system.memory_usage_percent),
            );
            vr(ui, DIVIDER_HEIGHT);
            status_chip(ui, "DB", &short_path(self.repository.db_path(), 34), None);
            self.render_pending_work_chips(ui);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_toast_strip(ui, &self.session.toasts);
            });
        });
    }

    /// Async work counters, shown only while something is in flight so the
    /// resting bar stays quiet.
    fn render_pending_work_chips(&self, ui: &mut egui::Ui) {
        for (label, count, tint) in [
            (
                "RPC",
                self.async_bus.rpc_health_pending.len(),
                theme::info(),
            ),
            (
                "Fed",
                self.async_bus.remote_federation_pending.len(),
                theme::info(),
            ),
            (
                "Alerts",
                self.async_bus.alert_delivery_pending,
                theme::warning(),
            ),
        ] {
            if count == 0 {
                continue;
            }
            vr(ui, DIVIDER_HEIGHT);
            status_chip(ui, label, &count.to_string(), Some(tint));
        }
    }
}

/// One `label value` pair. Drawn as two labels in the parent horizontal rather
/// than in a nested `ui.horizontal`, which would allocate a second full row
/// height inside a bar that has none to spare.
fn status_chip(ui: &mut egui::Ui, label: &str, value: &str, color: Option<egui::Color32>) {
    ui.spacing_mut().item_spacing.x = theme::XS;
    ui.label(theme::caption(label));
    let text = theme::body(value).strong();
    ui.label(match color {
        Some(color) => text.color(color),
        None => text,
    });
    ui.spacing_mut().item_spacing.x = theme::SM;
}

fn pressure_tint(percent: f32) -> Option<egui::Color32> {
    if percent >= 90.0 {
        Some(theme::danger())
    } else if percent >= 75.0 {
        Some(theme::warning())
    } else {
        None
    }
}
