use eframe::egui;

use super::super::{
    text::truncate_middle,
    widgets::{metric_tile, panel},
    NeoNexusApp,
};

mod inspector;
mod profiles;

impl NeoNexusApp {
    pub(super) fn render_federation(&mut self, ui: &mut egui::Ui) {
        let enabled = self
            .remote_servers
            .iter()
            .filter(|profile| profile.enabled)
            .count();
        let disabled = self.remote_servers.len().saturating_sub(enabled);
        let selected = self
            .selected_remote_server_profile()
            .map_or_else(|| "none".to_string(), |profile| profile.name);
        let probe = self
            .selected_remote_server_probe()
            .map_or("not probed", |report| report.status.label());
        let auto_label = if self.remote_federation_monitor_policy.enabled {
            "enabled"
        } else {
            "disabled"
        };
        let auto_detail = format!("{} pending", self.remote_federation_pending.len());

        ui.horizontal(|ui| {
            metric_tile(
                ui,
                "Remotes",
                &self.remote_servers.len().to_string(),
                "saved profiles",
            );
            metric_tile(ui, "Enabled", &enabled.to_string(), "active probes");
            metric_tile(ui, "Disabled", &disabled.to_string(), "paused profiles");
            metric_tile(ui, "Auto", auto_label, &auto_detail);
            metric_tile(ui, "Probe", probe, &truncate_middle(&selected, 20));
        });

        ui.add_space(10.0);
        let available = ui.available_size();
        let gap = 8.0;
        let list_width = (available.x * 0.34)
            .clamp(310.0, 460.0)
            .min((available.x - gap * 2.0 - 620.0).max(310.0));
        let editor_width = (available.x * 0.34)
            .clamp(320.0, 500.0)
            .min((available.x - list_width - gap * 2.0 - 300.0).max(320.0));
        let inspector_width = (available.x - list_width - editor_width - gap * 2.0).max(300.0);

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(list_width, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Remote profiles", |ui| {
                        self.render_remote_profile_list(ui);
                    });
                },
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                egui::vec2(editor_width, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Profile editor", |ui| {
                        self.render_remote_profile_editor(ui);
                    });
                },
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                egui::vec2(inspector_width, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Endpoint inspector", |ui| {
                        self.render_remote_profile_inspector(ui);
                    });
                },
            );
        });
    }
}
