use eframe::egui;

use crate::app::{
    domain::validate_runtime_manifest,
    theme::{accent, danger},
    NeoNexusApp,
};

pub(super) fn render_manifest_status(app: &NeoNexusApp, ui: &mut egui::Ui) {
    let manifest_status = app
        .runtime_package_draft
        .to_manifest()
        .and_then(|manifest| validate_runtime_manifest(&manifest));
    match manifest_status {
        Ok(()) => ui.label(egui::RichText::new("Manifest is valid.").color(accent())),
        Err(error) => ui.label(egui::RichText::new(error.to_string()).color(danger())),
    };
}
