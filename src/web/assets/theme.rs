//! UI density mode for compact vs comfortable display modes.

use std::fmt;

/// UI density mode for compact vs comfortable display modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityMode {
    Comfortable,
    Compact,
}

impl DensityMode {
    pub const DEFAULT: Self = DensityMode::Comfortable;

    /// Returns row height in pixels for this density mode.
    pub const fn row_height(&self) -> u32 {
        match self {
            DensityMode::Comfortable => 56,
            DensityMode::Compact => 40,
        }
    }

    /// Returns XS margin spacing value in pixels.
    pub const fn margin_xs(&self) -> u32 {
        match self {
            DensityMode::Comfortable => 8, // xs = 8px in comfortable
            DensityMode::Compact => 4,     // xs = 4px in compact
        }
    }

    /// Returns SM margin spacing value in pixels.
    pub const fn margin_sm(&self) -> u32 {
        match self {
            DensityMode::Comfortable => 12, // sm = 12px in comfortable
            DensityMode::Compact => 8,      // sm = 8px in compact
        }
    }

    /// Parse from string (for loading from settings). Infallible: an
    /// unrecognised value falls back to the comfortable default rather than
    /// erroring, so this is an inherent method and not [`std::str::FromStr`].
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "compact" => DensityMode::Compact,
            _ => DensityMode::Comfortable, // default
        }
    }

    /// Convert to string (for saving to settings).
    pub fn as_str(&self) -> &'static str {
        match self {
            DensityMode::Comfortable => "comfortable",
            DensityMode::Compact => "compact",
        }
    }

    /// The bare CSS class name the shell carries so the stylesheet can scope
    /// its density modifiers. This is the class attribute value — the leading
    /// `.` selectors live in [`DENSITY_COMPACT_CLASS`] / [`DENSITY_COMFORTABLE_CLASS`].
    pub const fn body_class(&self) -> &'static str {
        match self {
            DensityMode::Comfortable => "density-comfortable",
            DensityMode::Compact => "density-compact",
        }
    }
}

impl Default for DensityMode {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for DensityMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Density-aware spacing tokens. Each accessor takes the active [`DensityMode`]
/// so callers resolve XS/SM margins from a single source of truth rather than
/// hard-coding the pixel step at each call site.
pub mod spacing {
    use super::DensityMode;

    /// XS spacing based on current density mode.
    pub const fn xs(mode: DensityMode) -> u32 {
        mode.margin_xs()
    }

    /// SM spacing based on current density mode.
    pub const fn sm(mode: DensityMode) -> u32 {
        mode.margin_sm()
    }
}

/// CSS selector applied to the shell when the compact density is active.
pub const DENSITY_COMPACT_CLASS: &str = ".density-compact";
/// CSS selector applied to the shell when the comfortable density is active.
pub const DENSITY_COMFORTABLE_CLASS: &str = ".density-comfortable";
