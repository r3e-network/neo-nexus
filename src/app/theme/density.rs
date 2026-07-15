//! UI density metrics for Comfortable (default) and Compact.
//!
//! Compact **list/inventory row heights stay equal to Comfortable** until a
//! geometry proof PR. Compact only defines denser control metrics (buttons,
//! padding, spacing, nav row) applied at runtime in PR-12 via Settings.

/// Operator-facing density preference. Persisted as `appearance.ui_density`
/// (PR-05); applied to egui style in PR-12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::app) enum UiDensity {
    #[default]
    Comfortable,
    Compact,
}

impl UiDensity {
    /// Persist key for `appearance.ui_density`.
    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Comfortable => "comfortable",
            Self::Compact => "compact",
        }
    }

    /// Load from `appearance.ui_density`.
    pub(in crate::app) fn from_persist_key(key: &str) -> Option<Self> {
        match key {
            "comfortable" => Some(Self::Comfortable),
            "compact" => Some(Self::Compact),
            _ => None,
        }
    }

    /// Settings control label (UI in PR-12).
    #[allow(dead_code)]
    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Comfortable => "Comfortable",
            Self::Compact => "Compact",
        }
    }
}

/// Concrete spacing/sizing table for a density mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::app) struct DensityMetrics {
    /// `style.spacing.interact_size.y`
    pub(in crate::app) interact_y: f32,
    /// `style.spacing.button_padding` (x, y)
    pub(in crate::app) button_pad_x: f32,
    pub(in crate::app) button_pad_y: f32,
    /// `style.spacing.item_spacing` (x, y)
    pub(in crate::app) item_spacing_x: f32,
    pub(in crate::app) item_spacing_y: f32,
    /// Sidebar nav row height (not applied until PR-12 for Compact).
    pub(in crate::app) nav_row_height: f32,
    /// Inventory/fleet compact node row min height.
    pub(in crate::app) list_row_compact: f32,
    /// Inventory/fleet expanded node row min height.
    pub(in crate::app) list_row_expanded: f32,
    /// Event journal fixed empty-slot height.
    pub(in crate::app) journal_slot: f32,
}

impl DensityMetrics {
    pub(in crate::app) const COMFORTABLE: Self = Self {
        interact_y: 28.0,
        button_pad_x: 14.0,
        button_pad_y: 8.0,
        item_spacing_x: 10.0,
        item_spacing_y: 8.0,
        nav_row_height: 34.0,
        list_row_compact: 44.0,
        list_row_expanded: 56.0,
        journal_slot: 52.0,
    };

    /// Compact densifies **controls only**. List row heights match Comfortable
    /// so inventory geometry stays proven until PR-14-full.
    pub(in crate::app) const COMPACT: Self = Self {
        interact_y: 24.0,
        button_pad_x: 10.0,
        button_pad_y: 6.0,
        item_spacing_x: 8.0,
        item_spacing_y: 6.0,
        nav_row_height: 28.0,
        list_row_compact: 44.0,
        list_row_expanded: 56.0,
        journal_slot: 52.0,
    };

    pub(in crate::app) fn for_density(density: UiDensity) -> Self {
        match density {
            UiDensity::Comfortable => Self::COMFORTABLE,
            UiDensity::Compact => Self::COMPACT,
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app/theme/density/tests.rs"]
mod tests;
