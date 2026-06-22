use super::View;

#[test]
fn every_page_carries_a_label_title_and_subtitle() {
    for view in View::ALL {
        assert!(!view.label().is_empty());
        assert!(!view.title().is_empty());
        assert!(!view.subtitle().is_empty());
    }
}

#[test]
fn only_node_centric_pages_show_the_inventory() {
    assert!(View::Summary.shows_inventory());
    assert!(View::Nodes.shows_inventory());
    assert!(View::Monitor.shows_inventory());

    assert!(!View::Settings.shows_inventory());
    assert!(!View::Wallets.shows_inventory());
    assert!(!View::Federation.shows_inventory());
}
