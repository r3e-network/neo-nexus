use eframe::egui::{self, Color32};

use crate::app::domain::NodeStatus;

mod density;
mod icons;
mod palette;
mod style;
mod tokens;

pub(super) use icons::{
    brand_glyph, empty_glyph, glyph as view_icon_glyph, install as install_icons,
};
use palette::{active_theme, palette, set_active_theme};
pub(super) use style::configure_style;
// PR-12 will re-export configure_style_with_density for frame.rs.
#[allow(unused_imports)]
pub(in crate::app) use style::configure_style_with_density;
#[allow(unused_imports)] // PR-02/05/12 adopt density metrics and XL spacing
pub(in crate::app) use density::{DensityMetrics, UiDensity};
pub(in crate::app) use tokens::{
    body, column_header, label_caption, metric_value, muted_body, page_title, section_title,
    CHROME_HEADER_HEIGHT, CHROME_SIDEBAR_WIDTH, CHROME_STATUS_HEIGHT, LG, MD, SM, XS,
};
#[allow(unused_imports)]
pub(in crate::app) use tokens::XL;

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
