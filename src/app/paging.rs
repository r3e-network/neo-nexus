/// The furthest page index that can still be valid for `item_count` rows.
///
/// Flow-level reconcilers run before the panel is laid out, so they cannot know
/// the page size the render site will derive from the panel height. Bounding
/// them by a fixed page-size constant would fight that derivation: whenever the
/// derived size is smaller than the constant the list grows extra pages, and a
/// clamp against the constant — re-run every frame out of `update` — snaps the
/// operator back to the constant's last page before they can reach them, so the
/// rows past it become unreachable. One row per page is the smallest size
/// `rows_that_fit` can return, which makes `item_count - 1` the loosest bound
/// that is always correct here. The render site, which alone knows the page
/// size it laid out, then tightens the index against its own page count.
pub(super) fn clamp_page(current: usize, item_count: usize) -> usize {
    current.min(item_count.saturating_sub(1))
}

pub(super) fn page_count(item_count: usize, page_size: usize) -> usize {
    item_count.div_ceil(page_size).max(1)
}

/// How many fixed-height rows fit in `available` points once `reserved` points
/// are taken by the surrounding chrome (filters, search, pagination bar).
///
/// The workbench has no scrolling: a list that renders more rows than fit does
/// not become scrollable, it paints the surplus outside the panel where nobody
/// can see or click it. Page sizes are therefore derived from the space a
/// surface actually has rather than from a fixed constant chosen by eye. Always
/// returns at least one row, so a cramped panel still shows something and its
/// pagination bar still offers a way forward.
pub(super) fn rows_that_fit(available: f32, row_height: f32, reserved: f32) -> usize {
    if !row_height.is_finite() || row_height <= 0.0 {
        return 1;
    }
    let usable = available - reserved;
    if !usable.is_finite() || usable <= 0.0 {
        return 1;
    }
    ((usable / row_height).floor() as usize).max(1)
}

#[cfg(test)]
#[path = "../../tests/unit/app/paging/tests.rs"]
mod tests;
