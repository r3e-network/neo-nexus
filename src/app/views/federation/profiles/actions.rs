use eframe::egui;

use crate::app::{theme, widgets, NeoNexusApp};

pub(super) fn render_remote_profile_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.add_space(theme::SM);
    render_profile_mutations(app, ui);
    render_form_actions(app, ui);
    ui.separator();
    render_probe_actions(app, ui);
}

fn render_profile_mutations(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if widgets::secondary_button(ui, "Add Profile").clicked() {
            app.create_remote_server_profile();
        }
        if widgets::secondary_button_enabled(ui, "Update Selected", has_selection(app)).clicked() {
            app.update_selected_remote_server_profile();
        }
    });
}

fn render_form_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if widgets::secondary_button_enabled(ui, "Load Selected", has_selection(app)).clicked() {
            app.load_selected_remote_server_into_form();
        }
        if widgets::secondary_button(ui, "Reset Form").clicked() {
            app.reset_remote_server_form();
        }
    });
}

fn render_probe_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if widgets::secondary_button_enabled(ui, "Probe Status", has_selection(app)).clicked() {
            app.probe_selected_remote_server();
        }
        if widgets::secondary_button_enabled(ui, "Toggle Enabled", has_selection(app)).clicked() {
            app.toggle_selected_remote_server_enabled();
        }
    });
    if widgets::secondary_button_enabled(ui, "Delete Selected", has_selection(app)).clicked() {
        app.delete_selected_remote_server();
    }
}

fn has_selection(app: &NeoNexusApp) -> bool {
    app.selected_remote_server.is_some()
}
