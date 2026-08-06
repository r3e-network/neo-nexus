use eframe::egui;

use super::super::super::{theme, NeoNexusApp};

mod actions;
mod fields;
mod status;
mod summary;

pub(super) use summary::render_install_summary;

impl NeoNexusApp {
    /// The install wizard, read top to bottom: what the package **is**, whether
    /// its manifest checks out, and what to do about it.
    ///
    /// It used to be a fixed 54/46 split with the definition groups stacked
    /// down the left and the workspace paths at the bottom. In a narrow
    /// workspace that ran to eleven stacked fields, and the actions below them
    /// were laid out past the panel edge — invisible on a surface that does not
    /// scroll. All four field groups now flow across the pane together, and the
    /// workspace paths moved to Installed, which is the tab about what is
    /// already on disk.
    pub(super) fn render_runtime_install_form(&mut self, ui: &mut egui::Ui) {
        fields::render_package_fields(self, ui);
        ui.add_space(theme::MD);
        status::render_manifest_status(self, ui);
        ui.add_space(theme::MD);
        actions::render_install_actions(self, ui);
    }
}
