use super::*;

#[test]
fn native_header_tabs_fit_minimum_window_budget() {
    let width = view_button_width(1280.0);
    let total = width * View::ALL.len() as f32 + RESERVED_ACTION_WIDTH;

    assert!(width >= MIN_VIEW_BUTTON_WIDTH);
    assert!(total <= 1280.0);
}
