use super::*;
use crate::app::view::View;

#[test]
fn every_tab_round_trips_through_its_persist_key() {
    for tab in NodeWorkspaceTab::ALL {
        assert_eq!(
            NodeWorkspaceTab::from_persist_key(tab.persist_key()),
            Some(tab)
        );
    }
}

#[test]
fn legacy_views_map_into_workspace_tabs() {
    assert_eq!(
        NodeWorkspaceTab::from_legacy_view(View::Logs),
        Some(NodeWorkspaceTab::Logs)
    );
    assert_eq!(
        NodeWorkspaceTab::from_legacy_view(View::Config),
        Some(NodeWorkspaceTab::Config)
    );
    assert_eq!(
        NodeWorkspaceTab::from_legacy_view(View::Plugins),
        Some(NodeWorkspaceTab::Plugins)
    );
    assert_eq!(
        NodeWorkspaceTab::from_legacy_view(View::Monitor),
        Some(NodeWorkspaceTab::Health)
    );
    assert_eq!(NodeWorkspaceTab::from_legacy_view(View::Summary), None);
}
