use std::sync::atomic::{AtomicU8, Ordering};

use eframe::egui::{self, Color32};

use crate::app::domain::NodeStatus;

mod icons;
mod style;
mod tokens;

pub(super) use icons::{
    brand_glyph, empty_glyph, glyph as view_icon_glyph, install as install_icons,
};
pub(super) use style::configure_style;
pub(in crate::app) use tokens::{
    body, column_header, label_caption, metric_value, muted_body, page_title, section_title, LG,
    MD, SM, XS,
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
// and macOS system status colours. The three background tiers run darkest to
// lightest: window (canvas) < panel (sidebar chrome) < card (raised content),
// so cards and panels each read as a distinct elevated layer.
const LIGHT_PALETTE: Palette = Palette {
    accent: Color32::from_rgb(88, 86, 214),
    accent_hover: Color32::from_rgb(73, 71, 196),
    on_accent: Color32::from_rgb(255, 255, 255),
    text: Color32::from_rgb(29, 29, 31),
    muted_text: Color32::from_rgb(117, 117, 123),
    card_fill: Color32::from_rgb(255, 255, 255),
    panel_fill: Color32::from_rgb(244, 244, 248),
    window_fill: Color32::from_rgb(236, 236, 240),
    field_fill: Color32::from_rgb(255, 255, 255),
    faint_fill: Color32::from_rgb(232, 232, 237),
    border: Color32::from_rgb(220, 220, 226),
    status_running: Color32::from_rgb(40, 167, 90),
    status_starting: Color32::from_rgb(200, 110, 0),
    status_stopped: Color32::from_rgb(142, 142, 147),
    status_error: Color32::from_rgb(213, 60, 55),
    info: Color32::from_rgb(10, 122, 158),
};

// Dark: three clearly separated background tiers so the workbench is never a
// wall of near-black. The window (canvas) is the darkest, chrome panels lift
// visibly above it, and cards lift again — mirroring the light theme's readable
// hierarchy. Tiers are spaced ~16 brightness steps apart so each layer is
// perceptible, with a brighter border so panel/card edges read clearly on dark.
const DARK_PALETTE: Palette = Palette {
    accent: Color32::from_rgb(100, 97, 235),
    accent_hover: Color32::from_rgb(86, 83, 220),
    on_accent: Color32::from_rgb(255, 255, 255),
    text: Color32::from_rgb(243, 243, 245),
    muted_text: Color32::from_rgb(162, 162, 168),
    card_fill: Color32::from_rgb(56, 56, 63),
    panel_fill: Color32::from_rgb(38, 38, 44),
    window_fill: Color32::from_rgb(20, 20, 23),
    field_fill: Color32::from_rgb(50, 50, 57),
    faint_fill: Color32::from_rgb(52, 52, 59),
    border: Color32::from_rgb(78, 78, 88),
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

/// The workbench surface that cards float on top of (panels, sidebar,
/// inventory, inspector). This is the mid-tone of the three-tier background
/// hierarchy: `window_fill` < `panel_fill` < `card_fill`, so cards read as a
/// distinct elevated layer rather than dissolving into the workspace.
pub(super) fn panel_fill() -> Color32 {
    palette(active_theme()).panel_fill
}

/// The raised card surface. Cards and pill containers use this lighter fill so
/// they lift clearly off the surrounding `panel_fill` workspace.
pub(super) fn card_surface() -> Color32 {
    palette(active_theme()).card_fill
}

/// Soft drop shadow for raised cards. A wider, softer blur with a touch of
/// vertical offset so cards read as floating surfaces with real depth (the
/// macOS "elevated card" look) rather than a faint edge tint. Dark surfaces
/// get a stronger, more opaque shadow since dark backgrounds swallow faint
/// ones; light surfaces stay airy but visibly lifted.
pub(super) fn card_shadow() -> egui::Shadow {
    let theme = active_theme();
    let alpha = if theme.is_dark() { 150 } else { 42 };
    egui::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 1,
        color: Color32::from_black_alpha(alpha),
    }
}

/// Hairline separator stroke shared by panel boundaries and dividers.
pub(super) fn hairline() -> egui::Stroke {
    egui::Stroke::new(1.0, palette(active_theme()).border)
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
