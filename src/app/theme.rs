use std::sync::atomic::{AtomicU8, Ordering};

use eframe::egui::{self, Color32};

use crate::app::domain::NodeStatus;

/// Visual theme for the native workbench. `Light` preserves the original
/// slate/cyan workbench palette; `Dark` is a slate-900 counterpart with the
/// same accent family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::app) enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    pub(in crate::app) fn from_dark_mode(dark: bool) -> Self {
        if dark {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    pub(in crate::app) fn is_dark(self) -> bool {
        matches!(self, Theme::Dark)
    }

    pub(in crate::app) fn toggled(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }

    /// Label for the control that switches to the other theme.
    pub(in crate::app) fn toggle_label(self) -> &'static str {
        match self {
            Theme::Light => "Dark theme",
            Theme::Dark => "Light theme",
        }
    }
}

// Active theme shared with the zero-argument colour helpers below. The egui
// application runs on a single thread, so a relaxed atomic is sufficient and
// avoids threading the palette through every one of the ~190 call sites.
static ACTIVE_THEME: AtomicU8 = AtomicU8::new(0);

fn set_active_theme(theme: Theme) {
    ACTIVE_THEME.store(u8::from(theme.is_dark()), Ordering::Relaxed);
}

fn active_theme() -> Theme {
    Theme::from_dark_mode(ACTIVE_THEME.load(Ordering::Relaxed) == 1)
}

#[derive(Clone, Copy)]
struct Palette {
    accent: Color32,
    muted_text: Color32,
    card_fill: Color32,
    panel_fill: Color32,
    window_fill: Color32,
    extreme_bg: Color32,
    selection_text: Color32,
    status_running: Color32,
    status_starting: Color32,
    status_stopped: Color32,
    status_error: Color32,
}

const LIGHT_PALETTE: Palette = Palette {
    accent: Color32::from_rgb(14, 116, 144),
    muted_text: Color32::from_rgb(71, 85, 105),
    card_fill: Color32::from_rgb(255, 255, 255),
    panel_fill: Color32::from_rgb(238, 242, 247),
    window_fill: Color32::from_rgb(248, 250, 252),
    extreme_bg: Color32::from_rgb(226, 232, 240),
    selection_text: Color32::WHITE,
    status_running: Color32::from_rgb(21, 128, 61),
    status_starting: Color32::from_rgb(202, 138, 4),
    status_stopped: Color32::from_rgb(75, 85, 99),
    status_error: Color32::from_rgb(185, 28, 28),
};

const DARK_PALETTE: Palette = Palette {
    accent: Color32::from_rgb(34, 211, 238),
    muted_text: Color32::from_rgb(148, 163, 184),
    card_fill: Color32::from_rgb(30, 41, 59),
    panel_fill: Color32::from_rgb(15, 23, 42),
    window_fill: Color32::from_rgb(2, 6, 23),
    extreme_bg: Color32::from_rgb(2, 6, 23),
    selection_text: Color32::from_rgb(15, 23, 42),
    status_running: Color32::from_rgb(34, 197, 94),
    status_starting: Color32::from_rgb(250, 204, 21),
    status_stopped: Color32::from_rgb(148, 163, 184),
    status_error: Color32::from_rgb(248, 113, 113),
};

fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Light => LIGHT_PALETTE,
        Theme::Dark => DARK_PALETTE,
    }
}

pub(super) fn configure_style(context: &egui::Context, theme: Theme) {
    set_active_theme(theme);
    let palette = palette(theme);
    let mut style = (*context.style()).clone();
    style.visuals = if theme.is_dark() {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    style.visuals.panel_fill = palette.panel_fill;
    style.visuals.window_fill = palette.window_fill;
    style.visuals.extreme_bg_color = palette.extreme_bg;
    style.visuals.selection.bg_fill = palette.accent;
    style.visuals.selection.stroke.color = palette.selection_text;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    context.set_style(style);
}

pub(super) fn accent() -> Color32 {
    palette(active_theme()).accent
}

pub(super) fn muted_text() -> Color32 {
    palette(active_theme()).muted_text
}

pub(super) fn panel_fill() -> Color32 {
    palette(active_theme()).card_fill
}

pub(super) fn status_color(status: NodeStatus) -> Color32 {
    let palette = palette(active_theme());
    match status {
        NodeStatus::Running => palette.status_running,
        NodeStatus::Starting => palette.status_starting,
        NodeStatus::Stopped => palette.status_stopped,
        NodeStatus::Error => palette.status_error,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/theme/tests.rs"]
mod tests;
