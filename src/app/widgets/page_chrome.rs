//! In-workspace chrome: segmented sections and optional trailing filters.
//! Shell header owns title/subtitle — pass `title: None` on all primaries.

use eframe::egui;

use crate::app::theme;

use super::segmented::segmented_control;

/// Optional title path is reserved for non-shell contexts (none in v3.1).
/// Primaries must pass `title: None` so the shell header is not duplicated.
pub(in crate::app) fn page_chrome(
    ui: &mut egui::Ui,
    title: Option<&str>,
    segments: Option<(&[&str], &mut usize)>,
) -> bool {
    let mut changed = false;

    if let Some(title) = title {
        // Non-primary / exceptional surfaces only.
        ui.label(theme::page_title(title));
        ui.add_space(theme::SM);
    }

    if let Some((labels, selected)) = segments {
        let mut index = *selected;
        if segmented_control(ui, labels, &mut index) {
            *selected = index;
            changed = true;
        }
        ui.add_space(theme::MD);
    }

    changed
}
