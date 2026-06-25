//! The two colour palettes (light/dark) and the active-theme plumbing that the
//! zero-argument accessor helpers in the parent module read. Extracted from
//! theme.rs so that file stays under the 200-line source-quality budget; the
//! palette data lives here, the public colour accessors live in theme.rs.

use std::sync::atomic::{AtomicU8, Ordering};

use eframe::egui::Color32;

use super::Theme;

// Active theme shared with the zero-argument colour helpers. The egui
// application runs on a single thread, so a relaxed atomic is sufficient and
// avoids threading the palette through every one of the ~190 call sites.
static ACTIVE_THEME: AtomicU8 = AtomicU8::new(0);

pub(super) fn set_active_theme(theme: Theme) {
    ACTIVE_THEME.store(u8::from(theme.is_dark()), Ordering::Relaxed);
}

pub(super) fn active_theme() -> Theme {
    Theme::from_dark_mode(ACTIVE_THEME.load(Ordering::Relaxed) == 1)
}

#[derive(Clone, Copy)]
pub(super) struct Palette {
    pub(super) accent: Color32,
    pub(super) accent_hover: Color32,
    pub(super) on_accent: Color32,
    pub(super) text: Color32,
    pub(super) muted_text: Color32,
    pub(super) card_fill: Color32,
    pub(super) panel_fill: Color32,
    pub(super) window_fill: Color32,
    pub(super) field_fill: Color32,
    pub(super) faint_fill: Color32,
    pub(super) border: Color32,
    pub(super) status_running: Color32,
    pub(super) status_starting: Color32,
    pub(super) status_stopped: Color32,
    pub(super) status_error: Color32,
    pub(super) info: Color32,
}

// Light: airy near-white surfaces with a soft grey workspace, hairline borders,
// and macOS system status colours. The three background tiers run darkest to
// lightest: window (canvas) < panel (sidebar chrome) < card (raised content),
// so cards and panels each read as a distinct elevated layer.
pub(super) const LIGHT_PALETTE: Palette = Palette {
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
pub(super) const DARK_PALETTE: Palette = Palette {
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

pub(super) fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Light => LIGHT_PALETTE,
        Theme::Dark => DARK_PALETTE,
    }
}
