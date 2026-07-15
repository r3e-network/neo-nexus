//! In-workspace page header used by multi-section pages (Phase 2+). Kept in the
//! shared kit so sectioned views share one chrome path.

#![allow(dead_code)]

use eframe::egui;

use crate::app::theme;

use super::segmented::segmented_control;

/// In-workspace page header: title, optional subtitle, optional segmented tabs.
/// Returns `true` when the segment selection changed this frame.
pub(in crate::app) fn page_header(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: Option<&str>,
    segments: Option<(&[&str], &mut usize)>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(theme::page_title(title));
            if let Some(subtitle) = subtitle {
                ui.add_space(2.0);
                ui.label(theme::muted_body(subtitle));
            }
        });
        if let Some((labels, selected)) = segments {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Cap width so the segmented control stays compact on wide panes.
                let width = (ui.available_width() * 0.55).clamp(220.0, 420.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 32.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        changed = segmented_control(ui, labels, selected);
                    },
                );
            });
        }
    });
    ui.add_space(theme::SM);
    ui.separator();
    ui.add_space(theme::SM);
    changed
}
