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
fn every_view_round_trips_through_its_persist_key() {
    for view in View::ALL {
        assert!(!view.persist_key().is_empty());
        assert_eq!(View::from_persist_key(view.persist_key()), Some(view));
    }
    assert_eq!(View::from_persist_key("not-a-view"), None);
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

#[test]
fn primary_nav_maps_legacy_tools_into_six_destinations() {
    assert_eq!(View::PRIMARY.len(), 6);
    assert!(View::Summary.is_primary());
    assert!(!View::Logs.is_primary());
    assert_eq!(View::Logs.primary_nav(), View::Nodes);
    assert_eq!(View::Snapshots.primary_nav(), View::Runtimes);
    assert_eq!(View::Alerts.primary_nav(), View::Settings);
    assert_eq!(View::Roles.primary_nav(), View::Federation);
    assert_eq!(View::Wallets.primary_nav(), View::Federation);
}
