use super::*;

#[test]
fn page_count_never_returns_zero() {
    assert_eq!(page_count(0, 7), 1);
    assert_eq!(page_count(1, 7), 1);
    assert_eq!(page_count(8, 7), 2);
}

#[test]
fn clamp_page_keeps_page_in_range() {
    assert_eq!(clamp_page(5, 0), 0);
    assert_eq!(clamp_page(5, 8), 5);
    assert_eq!(clamp_page(0, 8), 0);
    assert_eq!(clamp_page(9, 8), 7);
}

/// The flow-level clamp must never contradict a render site that derived a
/// smaller page size than the old fixed constant. With 5 rows and the former
/// NODE_PAGE_SIZE of 7 the constant allowed only page 0, while a panel that
/// fits 3 rows genuinely has 2 pages — clamping to the constant stranded the
/// rows on page 2.
#[test]
fn clamp_page_does_not_strand_rows_a_smaller_derived_page_reaches() {
    let items = 5;
    let derived = rows_that_fit(200.0, 60.0, 20.0);
    assert_eq!(derived, 3);
    let last_rendered_page = page_count(items, derived) - 1;
    assert_eq!(last_rendered_page, 1);
    assert_eq!(clamp_page(last_rendered_page, items), last_rendered_page);
}

#[test]
fn rows_that_fit_divides_the_space_left_after_chrome() {
    // 400pt of panel, 100pt of filters and pagination, 60pt rows -> 5 rows.
    assert_eq!(rows_that_fit(400.0, 60.0, 100.0), 5);
    // A partial row does not count: it would paint outside the panel.
    assert_eq!(rows_that_fit(399.0, 60.0, 100.0), 4);
}

#[test]
fn rows_that_fit_always_offers_at_least_one_row() {
    assert_eq!(rows_that_fit(0.0, 60.0, 0.0), 1);
    assert_eq!(rows_that_fit(50.0, 60.0, 40.0), 1);
    assert_eq!(rows_that_fit(-100.0, 60.0, 0.0), 1);
}

#[test]
fn rows_that_fit_rejects_a_degenerate_row_height() {
    assert_eq!(rows_that_fit(400.0, 0.0, 0.0), 1);
    assert_eq!(rows_that_fit(400.0, -10.0, 0.0), 1);
    assert_eq!(rows_that_fit(400.0, f32::NAN, 0.0), 1);
}

#[test]
fn a_page_derived_from_the_fit_still_covers_every_item() {
    let page_size = rows_that_fit(400.0, 60.0, 100.0);
    let items = 23;
    let pages = page_count(items, page_size);
    assert_eq!(pages, 5);
    assert!(pages * page_size >= items);
    assert!(clamp_page(99, items) >= pages - 1);
}
