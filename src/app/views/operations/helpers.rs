use eframe::egui;

use crate::{diagnostics::CheckSeverity, events::EventSeverity};

use super::super::super::theme::muted_text;

pub(super) fn score_color(score: usize) -> egui::Color32 {
    if score >= 90 {
        egui::Color32::from_rgb(21, 128, 61)
    } else if score >= 70 {
        egui::Color32::from_rgb(202, 138, 4)
    } else {
        egui::Color32::from_rgb(185, 28, 28)
    }
}

pub(super) fn severity_color(severity: CheckSeverity) -> egui::Color32 {
    match severity {
        CheckSeverity::Pass => egui::Color32::from_rgb(21, 128, 61),
        CheckSeverity::Info => muted_text(),
        CheckSeverity::Warning => egui::Color32::from_rgb(202, 138, 4),
        CheckSeverity::Critical => egui::Color32::from_rgb(185, 28, 28),
    }
}

pub(super) fn event_color(severity: EventSeverity) -> egui::Color32 {
    match severity {
        EventSeverity::Info => muted_text(),
        EventSeverity::Warning => egui::Color32::from_rgb(202, 138, 4),
        EventSeverity::Critical => egui::Color32::from_rgb(185, 28, 28),
    }
}
