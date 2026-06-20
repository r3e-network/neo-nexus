use eframe::egui;

use crate::app::NeoNexusApp;

pub(super) fn render_remote_profile_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    render_profile_mutations(app, ui);
    render_form_actions(app, ui);
    ui.separator();
    render_probe_actions(app, ui);
}

fn render_profile_mutations(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("Add Profile").clicked() {
            app.create_remote_server_profile();
        }
        if ui
            .add_enabled(has_selection(app), egui::Button::new("Update Selected"))
            .clicked()
        {
            app.update_selected_remote_server_profile();
        }
    });
}

fn render_form_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(has_selection(app), egui::Button::new("Load Selected"))
            .clicked()
        {
            app.load_selected_remote_server_into_form();
        }
        if ui.button("Reset Form").clicked() {
            app.reset_remote_server_form();
        }
    });
}

fn render_probe_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(has_selection(app), egui::Button::new("Probe Status"))
            .clicked()
        {
            app.probe_selected_remote_server();
        }
        if ui
            .add_enabled(has_selection(app), egui::Button::new("Toggle Enabled"))
            .clicked()
        {
            app.toggle_selected_remote_server_enabled();
        }
    });
    if ui
        .add_enabled(has_selection(app), egui::Button::new("Delete Selected"))
        .clicked()
    {
        app.delete_selected_remote_server();
    }
}

fn has_selection(app: &NeoNexusApp) -> bool {
    app.selected_remote_server.is_some()
}
