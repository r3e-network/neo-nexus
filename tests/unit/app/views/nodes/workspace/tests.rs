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

/// A role is applied to one node, so it belongs in that node's workspace —
/// not on the Network page, where it used to sit beside remote endpoints and
/// private-network topology.
#[test]
fn the_node_workspace_owns_the_roles_tab() {
    assert!(NodeWorkspaceTab::ALL.contains(&NodeWorkspaceTab::Roles));
    assert_eq!(NodeWorkspaceTab::Roles.persist_key(), "roles");
    assert_eq!(
        NodeWorkspaceTab::from_persist_key("roles"),
        Some(NodeWorkspaceTab::Roles),
    );
}

/// Tabs read in the order an operator works through a node: define it, see the
/// config that produces, give it a role, enable the plugins the role needs,
/// then watch it run.
#[test]
fn tabs_follow_the_order_of_work() {
    let labels: Vec<&str> = NodeWorkspaceTab::ALL
        .iter()
        .map(|tab| tab.label())
        .collect();
    assert_eq!(
        labels,
        ["Studio", "Config", "Roles", "Plugins", "Logs", "Health"],
    );
}
