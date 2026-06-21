use super::View;

#[test]
fn compact_workspace_labels_fit_fixed_native_tabs() {
    assert!(View::ALL.iter().all(|view| view.short_label().len() <= 5));
}
