mod alerts;
mod config;
mod federation;
mod logs;
mod monitor;
mod nodes;
mod operations;
mod overview;
mod plugins;
mod roles;
mod runtimes;
mod settings;
mod shell;
mod snapshots;
mod wallets;

use eframe::egui;

use super::{theme::muted_text, view::View, widgets::workspace_header, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_workspace(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        workspace_header(
            ui,
            self.selected_view.title(),
            self.selected_view.subtitle(),
        );
        ui.add_space(6.0);
        self.render_workspace_tabs(ui);
        ui.add_space(8.0);

        match self.selected_view {
            View::Summary => self.render_overview(ui),
            View::Operations => self.render_operations(ui),
            View::Monitor => self.render_monitor(ui),
            View::Alerts => self.render_alerts(ui),
            View::Federation => self.render_federation(ui),
            View::Settings => self.render_settings(ui),
            View::Runtimes => self.render_runtimes(ui),
            View::Wallets => self.render_wallets(ui),
            View::Nodes => self.render_nodes(ui),
            View::Roles => self.render_roles(ui),
            View::Snapshots => self.render_snapshots(ui),
            View::Plugins => self.render_plugins(ui),
            View::Config => self.render_config(ui),
            View::Logs => self.render_logs(ui),
        }
    }

    fn render_workspace_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("View").color(muted_text()));
            ui.separator();
            let tab_gap = 4.0;
            let reserved = 56.0;
            let tab_width =
                ((ui.available_width() - reserved) / View::ALL.len() as f32).clamp(42.0, 74.0);
            for view in View::ALL {
                if ui
                    .add_sized(
                        [tab_width, 24.0],
                        egui::Button::new(view.short_label()).selected(self.selected_view == view),
                    )
                    .on_hover_text(format!("{} - {}", view.title(), view.subtitle()))
                    .clicked()
                {
                    self.selected_view = view;
                }
                ui.add_space(tab_gap);
            }
        });
    }
}
