use eframe::egui::RichText;

use super::muted_text;

// Spacing scale (logical points). One rhythm shared by every view so panels and
// sections breathe consistently instead of each file inventing its own gaps.
// The scale is a 4pt grid (XS 4 → SM 8 → MD 12 → LG 16 → XL 20) so spacing
// stays harmonious across the whole workbench.
pub(in crate::app) const XS: f32 = 4.0;
pub(in crate::app) const SM: f32 = 8.0;
pub(in crate::app) const MD: f32 = 12.0;
pub(in crate::app) const LG: f32 = 16.0;
pub(in crate::app) const XL: f32 = 20.0;

// Type scale. Sizes live here as the single tuning knob; colour is pulled from
// the live theme inside each constructor so typography stays theme-correct.
const SIZE_PAGE_TITLE: f32 = 17.0;
const SIZE_SECTION_TITLE: f32 = 14.0;
const SIZE_COLUMN_HEADER: f32 = 12.0;
const SIZE_CAPTION: f32 = 11.0;
const SIZE_METRIC_VALUE: f32 = 24.0;
const SIZE_BODY: f32 = 13.0;

/// Top-of-page heading (shell header bar, empty-state titles).
pub(in crate::app) fn page_title(text: impl Into<String>) -> RichText {
    RichText::new(text).size(SIZE_PAGE_TITLE).strong()
}

/// Heading for a card/panel or an in-panel section.
pub(in crate::app) fn section_title(text: impl Into<String>) -> RichText {
    RichText::new(text).size(SIZE_SECTION_TITLE).strong()
}

/// Muted, upper-cased eyebrow label (metric titles, group headers).
pub(in crate::app) fn label_caption(text: impl Into<String>) -> RichText {
    RichText::new(text.into().to_uppercase())
        .size(SIZE_CAPTION)
        .color(muted_text())
}

/// Large, strong figure for metric tiles.
pub(in crate::app) fn metric_value(text: impl Into<String>) -> RichText {
    RichText::new(text).size(SIZE_METRIC_VALUE).strong()
}

/// Column header for data tables: small, muted, semibold (macOS list style).
pub(in crate::app) fn column_header(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .size(SIZE_COLUMN_HEADER)
        .color(muted_text())
        .strong()
}

/// Secondary/explanatory body text (subtitles, captions, empty-state messages).
pub(in crate::app) fn muted_body(text: impl Into<String>) -> RichText {
    RichText::new(text).size(SIZE_BODY).color(muted_text())
}

/// Primary body text at the default foreground colour (navigation labels, list
/// rows, default-strength body copy).
pub(in crate::app) fn body(text: impl Into<String>) -> RichText {
    RichText::new(text).size(SIZE_BODY)
}
