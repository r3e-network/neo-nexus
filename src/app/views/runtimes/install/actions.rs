use eframe::egui;

use crate::app::{
    theme,
    widgets::{primary_button, secondary_button},
    NeoNexusApp,
};

pub(super) fn render_install_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.label(theme::label_caption("Actions"));
    ui.add_space(theme::XS);
    ui.horizontal_wrapped(|ui| {
        if primary_button(ui, "Install")
            .on_hover_text("Install the package after local verification")
            .clicked()
        {
            app.install_runtime_package();
        }
        if secondary_button(ui, "Download HTTPS")
            .on_hover_text("Download the package from the HTTPS URL into the cache")
            .clicked()
        {
            app.download_runtime_package();
        }
        if secondary_button(ui, "Current Platform")
            .on_hover_text("Fill OS/arch from this host")
            .clicked()
        {
            app.runtime_package_draft.use_current_platform();
        }
        if secondary_button(ui, "Reset")
            .on_hover_text("Clear the install draft")
            .clicked()
        {
            app.runtime_package_draft = Default::default();
        }
    });
}
