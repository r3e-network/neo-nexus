use std::sync::atomic::{AtomicU8, Ordering};

use eframe::egui::{self, Color32};

use crate::app::domain::NodeStatus;

mod icons;
mod style;
mod tokens;

pub(super) use icons::{glyph as view_icon_glyph, install as install_icons};
pub(super) use style::configure_style;
pub(in crate::app) use tokens::{
    body, column_header, label_caption, metric_value, muted_body, page_title, section_title, MD,
    SM, XS,
};

/// Visual theme for the native workbench. The palettes follow a calm,
/// macOS-style design language: near-neutral surfaces, hairline separators,
/// generous spacing, soft corners, and a single restrained indigo accent.
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
    accent_hover: Color32,
    on_accent: Color32,
    text: Color32,
    muted_text: Color32,
    card_fill: Color32,
    panel_fill: Color32,
    window_fill: Color32,
    field_fill: Color32,
    faint_fill: Color32,
    border: Color32,
    status_running: Color32,
    status_starting: Color32,
    status_stopped: Color32,
    status_error: Color32,
    info: Color32,
}

// Light: airy near-white surfaces with a soft grey workspace, hairline borders,
// and macOS system status colours.
const LIGHT_PALETTE: Palette = Palette {
    accent: Color32::from_rgb(88, 86, 214),
    accent_hover: Color32::from_rgb(73, 71, 196),
    on_accent: Color32::from_rgb(255, 255, 255),
    text: Color32::from_rgb(29, 29, 31),
    muted_text: Color32::from_rgb(117, 117, 123),
    card_fill: Color32::from_rgb(255, 255, 255),
    panel_fill: Color32::from_rgb(237, 237, 241),
    window_fill: Color32::from_rgb(242, 242, 246),
    field_fill: Color32::from_rgb(255, 255, 255),
    faint_fill: Color32::from_rgb(232, 232, 237),
    border: Color32::from_rgb(220, 220, 226),
    status_running: Color32::from_rgb(40, 167, 90),
    status_starting: Color32::from_rgb(200, 110, 0),
    status_stopped: Color32::from_rgb(142, 142, 147),
    status_error: Color32::from_rgb(213, 60, 55),
    info: Color32::from_rgb(10, 122, 158),
};

// Dark: layered greys (window < sidebar/card) with a brighter indigo accent and
// lifted status colours for contrast on dark surfaces.
const DARK_PALETTE: Palette = Palette {
    accent: Color32::from_rgb(100, 97, 235),
    accent_hover: Color32::from_rgb(85, 82, 220),
    on_accent: Color32::from_rgb(255, 255, 255),
    text: Color32::from_rgb(243, 243, 245),
    muted_text: Color32::from_rgb(152, 152, 157),
    card_fill: Color32::from_rgb(38, 38, 42),
    panel_fill: Color32::from_rgb(26, 26, 29),
    window_fill: Color32::from_rgb(22, 22, 25),
    field_fill: Color32::from_rgb(32, 32, 36),
    faint_fill: Color32::from_rgb(34, 34, 38),
    border: Color32::from_rgb(54, 54, 60),
    status_running: Color32::from_rgb(48, 209, 88),
    status_starting: Color32::from_rgb(255, 214, 70),
    status_stopped: Color32::from_rgb(152, 152, 157),
    status_error: Color32::from_rgb(255, 105, 97),
    info: Color32::from_rgb(90, 200, 250),
};

fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Light => LIGHT_PALETTE,
        Theme::Dark => DARK_PALETTE,
    }
}

pub(super) fn accent() -> Color32 {
    palette(active_theme()).accent
}

pub(super) fn on_accent() -> Color32 {
    palette(active_theme()).on_accent
}

pub(super) fn muted_text() -> Color32 {
    palette(active_theme()).muted_text
}

pub(super) fn panel_fill() -> Color32 {
    palette(active_theme()).card_fill
}

/// Semantic colours for inline status text (validation, severity, pressure,
/// diagnosis). They reuse the palette's status hues so the whole app shares one
/// set of success/warning/danger/info colours that adapt to light and dark.
pub(super) fn success() -> Color32 {
    palette(active_theme()).status_running
}

pub(super) fn warning() -> Color32 {
    palette(active_theme()).status_starting
}

pub(super) fn danger() -> Color32 {
    palette(active_theme()).status_error
}

pub(super) fn info() -> Color32 {
    palette(active_theme()).info
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
