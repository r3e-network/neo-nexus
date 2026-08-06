//! Responsive two-column layout.
//!
//! The workbench has up to four fixed columns (sidebar, inventory, workspace,
//! inspector) inside a 1280pt design window, so the central workspace is
//! routinely only ~450pt wide. Pages that split that space again with a
//! hard-coded fractional two-up ended up with panes too narrow to hold a form
//! field or a fact row, and their content painted straight over the neighbouring
//! pane. These helpers make "two-up when there is room, stacked when there is
//! not" a single decision the whole app shares.

use crate::app::theme;

/// The narrowest a content column can be and still hold a labelled field, a
/// fact row, or a node row without truncating. Below this, two-up is worse than
/// stacked in every case.
pub(in crate::app) const MIN_COLUMN_WIDTH: f32 = 306.0;

/// Whether `available` width can hold two [`MIN_COLUMN_WIDTH`] columns and the
/// gap between them.
pub(in crate::app) fn fits_side_by_side(available: f32) -> bool {
    fits_side_by_side_at(available, MIN_COLUMN_WIDTH)
}

/// [`fits_side_by_side`] with a caller-chosen minimum, for denser content such
/// as form fields that read fine in a narrower column than a full pane needs.
pub(in crate::app) fn fits_side_by_side_at(available: f32, min_column: f32) -> bool {
    available >= min_column * 2.0 + theme::SM
}

/// How many `min_column`-wide columns of `count` items fit in `available`.
/// Never fewer than one, never more than there are items.
///
/// This is the flow rule for *grouped* content — form field groups, tiles —
/// where wrapping onto a second row is always better than shrinking a labelled
/// field below the width of its own label. The workspace does not scroll, so a
/// tall single column paints its last rows outside the panel.
pub(in crate::app) fn columns_that_fit(available: f32, min_column: f32, count: usize) -> usize {
    if count == 0 || !min_column.is_finite() || min_column <= 0.0 {
        return 1;
    }
    let fits = (available / min_column).floor();
    if !fits.is_finite() || fits < 1.0 {
        return 1;
    }
    (fits as usize).min(count).max(1)
}

#[cfg(test)]
#[path = "../../../tests/unit/app/widgets/columns/tests.rs"]
mod tests;
