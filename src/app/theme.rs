use eframe::egui::{self, Color32};

use crate::app::domain::NodeStatus;

mod density;
mod icons;
mod palette;
mod style;
mod tokens;

pub(in crate::app) use density::{DensityMetrics, UiDensity};
pub(super) use icons::{
    brand_glyph, empty_glyph, glyph as view_icon_glyph, install as install_icons,
};
use palette::{active_theme, palette, set_active_theme};
pub(in crate::app) use style::configure_style_with_density;
#[allow(unused_imports)]
pub(in crate::app) use tokens::XL;
pub(in crate::app) use tokens::{
    body, body_font, caption, column_header, label_caption, metric_value, muted_body, page_title,
    section_title, CHROME_HEADER_HEIGHT, CHROME_SIDEBAR_WIDTH, CHROME_STATUS_HEIGHT, LG, MD, SM,
    XS,
};

/// Visual theme for the native workbench. The palettes follow a calm,
/// warm-neutral design language: paper-toned surfaces, hairline separators,
/// generous spacing, soft corners, flat (shadowless) cards, and a single
/// restrained coral accent that appears as a tint far more often than as a
/// solid fill.
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

pub(super) fn accent() -> Color32 {
    palette(active_theme()).accent
}

/// The deeper accent used for hover and pressed states on accent-filled
/// controls, so the app's primary action visibly responds to the pointer.
pub(super) fn accent_hover() -> Color32 {
    palette(active_theme()).accent_hover
}

/// The accent as *text or icon* on a neutral surface — a selected nav label, a
/// link, an active segment. Use this instead of [`accent`] whenever the accent
/// is drawn as glyphs rather than as a filled shape: the fill accent is tuned
/// to carry white labels, this one is tuned to be read against paper.
pub(super) fn accent_text() -> Color32 {
    palette(active_theme()).accent_text
}

/// The tinted selection wash: what a selected navigation row, list row, or
/// active toggle is filled with. The workbench never paints a solid accent
/// block behind body copy — selection reads as a quiet coral tint.
pub(super) fn accent_wash() -> Color32 {
    palette(active_theme()).accent_wash
}

pub(super) fn on_accent() -> Color32 {
    palette(active_theme()).on_accent
}

/// The primary foreground colour. Exposed so typography tokens can state their
/// colour explicitly rather than depending on egui's strong/weak resolution.
pub(super) fn text() -> Color32 {
    palette(active_theme()).text
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

/// The recessed track a segmented control or chip group sits in. Slightly
/// darker than the surrounding surface so the selected pill reads as lifted out
/// of it rather than painted onto it.
pub(super) fn track_surface() -> Color32 {
    palette(active_theme()).faint_fill
}

/// Hairline separator stroke shared by panel boundaries, card edges, and
/// dividers. Cards are flat: their edge — not a drop shadow — is what separates
/// them from the surface they sit on, so this stroke carries all of the
/// workbench's elevation. Depth comes from the three fill tiers instead.
pub(super) fn hairline() -> egui::Stroke {
    egui::Stroke::new(1.0, palette(active_theme()).border)
}

/// The stronger hairline used for hover and focus edges, so interaction
/// feedback does not have to reach for the accent on every control.
pub(super) fn hairline_strong() -> egui::Stroke {
    egui::Stroke::new(1.0, palette(active_theme()).border_strong)
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
