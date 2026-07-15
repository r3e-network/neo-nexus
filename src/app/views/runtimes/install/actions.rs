use eframe::egui;

use crate::app::{
    theme,
    widgets::{loading_callout, primary_button, secondary_button, secondary_button_enabled},
    NeoNexusApp,
};

pub(super) fn render_install_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.label(theme::label_caption("Actions"));
    ui.add_space(theme::XS);

    let busy = install_work_in_progress(app.session.notice.as_deref());
    if busy {
        loading_callout(
            ui,
            "Package work in progress",
            "Wait for the current install or download notice to clear.",
        );
        ui.add_space(theme::SM);
    }

    ui.horizontal_wrapped(|ui| {
        let install = if busy {
            secondary_button_enabled(ui, "Install", false)
        } else {
            primary_button(ui, "Install")
        };
        if install
            .on_hover_text("Install the package after local verification")
            .clicked()
        {
            app.install_runtime_package();
        }
        if secondary_button_enabled(ui, "Download HTTPS", !busy)
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

fn install_work_in_progress(notice: Option<&str>) -> bool {
    let Some(notice) = notice else {
        return false;
    };
    let lower = notice.to_ascii_lowercase();
    // Heuristic: domain flows surface install/download progress via notice.
    lower.contains("installing")
        || lower.contains("downloading")
        || lower.contains("extracting")
        || lower.contains("verifying package")
}
