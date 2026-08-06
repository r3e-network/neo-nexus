use super::NetworkHubSection;
use crate::app::view::View;

/// Network is about topology and reach: the remote endpoints this workbench
/// talks to, the private network it can stand up, and the wallets that sign for
/// both. Role presets are not here — a role is applied to one node, so it lives
/// in that node's workspace.
#[test]
fn the_hub_covers_topology_and_reach_only() {
    let labels: Vec<&str> = NetworkHubSection::ALL
        .iter()
        .map(|section| section.label())
        .collect();
    assert_eq!(labels, ["Remote", "Private Net", "Wallets"]);
}

#[test]
fn sections_round_trip_through_their_index() {
    for (index, section) in NetworkHubSection::ALL.iter().enumerate() {
        assert_eq!(*section as usize, index);
    }
}

/// The legacy `Roles` deep link now lands on private-network planning, which is
/// the part of the old Roles page that belonged on Network.
#[test]
fn deep_links_resolve_to_their_hub_section() {
    assert_eq!(
        NetworkHubSection::from_view(View::Federation),
        Some(NetworkHubSection::Federation),
    );
    assert_eq!(
        NetworkHubSection::from_view(View::Roles),
        Some(NetworkHubSection::PrivateNetwork),
    );
    assert_eq!(
        NetworkHubSection::from_view(View::Wallets),
        Some(NetworkHubSection::Wallets),
    );
    assert_eq!(NetworkHubSection::from_view(View::Nodes), None);
}

#[test]
fn the_default_section_is_the_remote_endpoint_list() {
    assert_eq!(NetworkHubSection::default(), NetworkHubSection::Federation);
}
