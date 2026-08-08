//! Titled card — the workbench's primary content container.
//!
//! A card hugs its content height so cards **stack** down a panel, which is
//! what the inspector column and every card-based page need.
//! [`super::layout::panel`] is the stretch variant: same chrome, but it claims
//! the full remaining height for surfaces that own their region.

use eframe::egui;

use crate::app::theme;

use super::{divider::hr, layout::card_frame};

/// A flat, hairline-bordered card with a section title, a rule, and a body.
/// Optionally shows a muted caption on the title row's trailing edge — a count,
/// a freshness stamp, or a status word.
pub(in crate::app) fn card(
    ui: &mut egui::Ui,
    title: &str,
    trailing: Option<&str>,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    card_frame(ui.style()).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        card_title(ui, title, trailing);
        hr(ui);
        add_contents(ui);
    });
}

/// The stretch variant: identical chrome, but it claims the full remaining
/// height for a surface that owns its region rather than stacking with peers.
/// Backs [`super::layout::panel`] so both share one code path.
pub(in crate::app) fn stretch_card(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let frame = card_frame(ui.style());
    // `available_size()` inside the closure is the room left for *content*, and
    // the frame adds its own margins around whatever the content asks for. So
    // stretching to the full available size makes the card its margins taller
    // and wider than the space it was given — which is exactly the 10pt every
    // filled page was painting below the panel that clips it.
    let margin = frame.total_margin().sum();
    frame.show(ui, |ui| {
        ui.set_min_size((ui.available_size() - margin).max(egui::Vec2::ZERO));
        card_title(ui, title, None);
        hr(ui);
        add_contents(ui);
    });
}

pub(in crate::app) fn card_title(ui: &mut egui::Ui, title: &str, trailing: Option<&str>) {
    ui.horizontal(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        ui.label(theme::section_title(title));
        if let Some(trailing) = trailing {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(theme::muted_body(trailing));
            });
        }
    });
}
