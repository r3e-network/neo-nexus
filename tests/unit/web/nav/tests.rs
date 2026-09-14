//! The sidebar is the workbench's only map, so its own table is checked for
//! drift: duplicate keys would highlight two entries, and an href that nothing
//! routes to would be a dead link an operator finds by clicking.

use super::*;

#[test]
fn every_destination_key_is_unique() {
    let keys = keys();
    let unique = keys
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(keys.len(), unique.len(), "sidebar keys repeat: {keys:?}");
    assert!(!keys.is_empty(), "the workbench would render no navigation");
}

#[test]
fn every_destination_resolves_to_its_own_href() {
    for key in keys() {
        let href = href_for(key).expect("listed key must resolve");
        assert!(href.starts_with('/'), "{key} href {href} is not absolute");
    }
    assert_eq!(href_for("not-a-page"), None);
}

#[test]
fn no_two_destinations_share_a_path() {
    let hrefs = keys()
        .iter()
        .filter_map(|key| href_for(key))
        .collect::<Vec<_>>();
    let unique = hrefs
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(hrefs.len(), unique.len(), "sidebar paths repeat: {hrefs:?}");
}

#[test]
fn render_highlights_exactly_the_active_destination() {
    let markup = render("nodes");
    let highlighted = markup.matches("nav-item current").count();
    assert_eq!(highlighted, 1, "expected one current entry in {markup}");
    assert!(markup.contains(r#"href="/nodes""#));
}

#[test]
fn render_highlights_nothing_for_an_unknown_key() {
    let markup = render("nowhere");
    assert!(!markup.contains("nav-item current"), "{markup}");
}

#[test]
fn render_lists_every_label_and_groups_them() {
    let markup = render("home");
    for key in keys() {
        let label = SECTIONS
            .iter()
            .flat_map(|section| section.destinations.iter())
            .find(|destination| destination.key == key)
            .map(|destination| destination.label)
            .expect("every key belongs to a destination");
        assert!(markup.contains(label), "{label} missing from the sidebar");
    }
    let groups = SECTIONS
        .iter()
        .filter(|section| !section.destinations.is_empty())
        .count();
    assert_eq!(
        markup.matches("nav-title").count(),
        groups,
        "every section with destinations needs a heading"
    );
}

#[test]
fn information_architecture_matches_the_operations_console() {
    let expected = [
        ("home", "/", "Fleet overview"),
        ("nodes", "/nodes", "Nodes"),
        // "Health" named the host page while chain health lives on each node —
        // one word over two different questions, and the more important one was
        // not the one it linked to.
        ("monitor", "/monitor", "Host health"),
        ("logs", "/logs", "Logs"),
        ("operations", "/operations", "Operations"),
        ("events", "/events", "Journal"),
        // "Alerts" promised alarms; this page holds the routing policy and the
        // delivery record.
        ("alerts", "/alerts", "Alert routing"),
        ("federation", "/federation", "Federation"),
        // Said "Private network", opened the per-node duty matrix — naming a
        // feature this build does not have while hiding the one it does.
        ("roles", "/roles", "Duties"),
        ("runtimes", "/runtimes", "Runtimes"),
        // Inbound archives a node fast-syncs *from*, not a capture of this
        // workspace going out — which is what "Backup" below is.
        ("snapshots", "/snapshots", "Fast-sync archives"),
        ("plugins", "/plugins", "Plugins"),
        ("config", "/config", "Configuration"),
        ("wallets", "/wallets", "Wallets"),
        // One signer binding wore five names across the console. Two words
        // survive: "key" on the custody side and "signer binding" on the node
        // side.
        ("signer", "/signer", "Signing keys"),
        // Registered, marked Settings active, and absent from the nav entirely
        // — reachable only from the hand-written services menu that is now
        // generated from these same destinations.
        ("api-tokens", "/settings/api-tokens", "API tokens"),
        ("metrics", "/metrics", "Metrics"),
        ("backup", "/backup", "Workspace backup"),
        ("settings", "/settings", "Settings"),
    ];
    assert_eq!(
        keys(),
        expected.iter().map(|entry| entry.0).collect::<Vec<_>>()
    );
    let markup = render("home");
    for (key, href, label) in expected {
        assert_eq!(href_for(key), Some(href));
        assert!(markup.contains(label), "{label} missing from {markup}");
    }
    assert!(markup.contains(r#"class="nav-group utility""#));
    assert!(markup.contains(r#"class="nav-icon""#));
}
