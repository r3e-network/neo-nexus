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
    /// The accent used as *text or icon* on a neutral surface. Darker than the
    /// fill accent in light mode and brighter in dark mode, so a coral label on
    /// a card still clears AA where the button fill colour would not.
    pub(super) accent_text: Color32,
    /// Tinted selection wash: the accent at low strength over the surface. This
    /// is what a selected nav row or list row is filled with — the workbench
    /// never paints a solid accent block behind body text.
    pub(super) accent_wash: Color32,
    pub(super) on_accent: Color32,
    pub(super) text: Color32,
    pub(super) muted_text: Color32,
    pub(super) card_fill: Color32,
    pub(super) panel_fill: Color32,
    pub(super) window_fill: Color32,
    pub(super) field_fill: Color32,
    pub(super) faint_fill: Color32,
    pub(super) border: Color32,
    /// A stronger hairline for hover and focus edges, so interaction feedback
    /// does not have to reach for the accent on every control.
    pub(super) border_strong: Color32,
    pub(super) status_running: Color32,
    pub(super) status_starting: Color32,
    pub(super) status_stopped: Color32,
    pub(super) status_error: Color32,
    pub(super) info: Color32,
}

// Light: warm neutral paper rather than cool grey. The three background tiers
// still run darkest to lightest — window (canvas) < panel (chrome) < card — so
// cards read as the raised layer, but depth now comes from hairlines and fill
// tiers instead of drop shadows. The accent is a terracotta coral dark enough
// that white button labels clear WCAG AA (4.6:1).
pub(super) const LIGHT_PALETTE: Palette = Palette {
    accent: Color32::from_rgb(195, 74, 44),
    accent_hover: Color32::from_rgb(168, 62, 35),
    accent_text: Color32::from_rgb(176, 66, 31),
    accent_wash: Color32::from_rgb(251, 237, 231),
    on_accent: Color32::from_rgb(255, 255, 255),
    text: Color32::from_rgb(28, 25, 23),
    muted_text: Color32::from_rgb(111, 104, 98),
    card_fill: Color32::from_rgb(255, 255, 255),
    panel_fill: Color32::from_rgb(250, 248, 245),
    window_fill: Color32::from_rgb(244, 241, 236),
    field_fill: Color32::from_rgb(255, 255, 255),
    faint_fill: Color32::from_rgb(241, 237, 231),
    border: Color32::from_rgb(230, 225, 217),
    border_strong: Color32::from_rgb(210, 203, 192),
    status_running: Color32::from_rgb(22, 124, 69),
    status_starting: Color32::from_rgb(180, 83, 9),
    status_stopped: Color32::from_rgb(122, 114, 106),
    status_error: Color32::from_rgb(179, 38, 30),
    info: Color32::from_rgb(16, 101, 127),
};

// Dark: warm charcoal, matching the light theme's paper warmth rather than a
// blue-black. The same three tiers are spaced ~12 brightness steps apart so
// each layer is perceptible (see tests/ui_visual_contract.rs), with a brighter
// border so panel and card edges read clearly. `accent_text` brightens for text
// use while `accent` stays dark enough to carry white button labels.
pub(super) const DARK_PALETTE: Palette = Palette {
    accent: Color32::from_rgb(203, 76, 40),
    accent_hover: Color32::from_rgb(224, 92, 54),
    accent_text: Color32::from_rgb(240, 138, 98),
    accent_wash: Color32::from_rgb(58, 37, 28),
    on_accent: Color32::from_rgb(255, 255, 255),
    text: Color32::from_rgb(245, 242, 238),
    muted_text: Color32::from_rgb(168, 160, 153),
    card_fill: Color32::from_rgb(46, 42, 38),
    panel_fill: Color32::from_rgb(28, 26, 24),
    window_fill: Color32::from_rgb(18, 17, 16),
    field_fill: Color32::from_rgb(38, 35, 32),
    faint_fill: Color32::from_rgb(55, 50, 45),
    border: Color32::from_rgb(69, 63, 57),
    border_strong: Color32::from_rgb(96, 88, 79),
    status_running: Color32::from_rgb(74, 222, 128),
    status_starting: Color32::from_rgb(251, 191, 36),
    status_stopped: Color32::from_rgb(154, 146, 138),
    status_error: Color32::from_rgb(255, 122, 107),
    info: Color32::from_rgb(86, 199, 232),
};

pub(super) fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Light => LIGHT_PALETTE,
        Theme::Dark => DARK_PALETTE,
    }
}
