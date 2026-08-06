use eframe::egui;

use super::super::super::{theme, widgets::columns_that_fit, NeoNexusApp};

mod actions;
mod fields;
mod status;
mod summary;

/// Narrowest the download pane may become before its labelled fields clip.
const SIDE_PANE_MIN_WIDTH: f32 = 220.0;

impl NeoNexusApp {
    /// The install wizard, read top to bottom: what the package **is**, where
    /// it comes **from**, then what the app will **do** with it.
    ///
    /// It used to be a fixed 54/46 split with the three definition groups
    /// stacked down the left. In a narrow workspace that column ran to eleven
    /// stacked fields and the actions and summary at the bottom were laid out
    /// past the panel edge — invisible on a surface that does not scroll. Both
    /// bands now flow across as many columns as the pane can actually hold.
    pub(super) fn render_runtime_install_form(&mut self, ui: &mut egui::Ui) {
        fields::render_package_fields(self, ui);
        ui.add_space(theme::MD);
        self.render_source_and_status(ui);
        ui.add_space(theme::MD);
        actions::render_install_actions(self, ui);
        ui.add_space(theme::SM);
        summary::render_install_summary(self, ui);
    }

    /// The download source beside the manifest validation it feeds, so an
    /// operator sees the URL and the verdict on it without moving their eyes
    /// down the page.
    fn render_source_and_status(&mut self, ui: &mut egui::Ui) {
        if columns_that_fit(ui.available_width(), SIDE_PANE_MIN_WIDTH, 2) < 2 {
            fields::render_download_fields(self, ui);
            ui.add_space(theme::MD);
            status::render_manifest_status(self, ui);
            return;
        }
        ui.columns(2, |columns| {
            fields::render_download_fields(self, &mut columns[0]);
            status::render_manifest_status(self, &mut columns[1]);
        });
    }
}
