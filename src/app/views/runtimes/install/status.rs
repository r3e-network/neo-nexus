use eframe::egui;

use crate::app::{
    domain::validate_runtime_manifest,
    theme,
    widgets::{callout, CalloutKind},
    NeoNexusApp,
};

pub(super) fn render_manifest_status(app: &NeoNexusApp, ui: &mut egui::Ui) {
    let manifest_status = app
        .runtime_package_draft
        .to_manifest()
        .and_then(|manifest| validate_runtime_manifest(&manifest));
    match manifest_status {
        Ok(()) => callout(
            ui,
            CalloutKind::Success,
            "Manifest is valid",
            "Package identity, platform, and integrity fields can be installed.",
        ),
        Err(error) => {
            callout(
                ui,
                CalloutKind::Danger,
                "Manifest is incomplete",
                "Fix the highlighted draft fields, then try again.",
            );
            ui.add_space(theme::SM);
            ui.label(egui::RichText::new(error.to_string()).color(theme::danger()));
        }
    }
}
