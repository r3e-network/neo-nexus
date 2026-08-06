//! A mirror of neo-go's `KnownFields(true)`.
//!
//! neo-go decodes its config with unknown-field rejection on: a key that is not
//! a field of the target struct is a **fatal startup error**, not a warning. So
//! a generator that invents a key does not produce a slightly-off node, it
//! produces a node that will not boot — and nothing in NeoNexus notices,
//! because our own validator reads our own output.
//!
//! Every name below was read from `pkg/config/*.go` on nspcc-dev/neo-go and
//! cross-checked against the shipped `config/protocol.mainnet.yml`. Adding a
//! key here without that check defeats the point of the test.

use crate::*;

#[path = "neo_go_schema/fields.rs"]
mod fields;

use fields::{APPLICATION_FIELDS, NESTED_FIELDS, PROTOCOL_FIELDS};

fn keys_of(value: &serde_yaml::Value) -> Vec<String> {
    value
        .as_mapping()
        .map(|mapping| {
            mapping
                .keys()
                .filter_map(serde_yaml::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn assert_known(section: &str, value: &serde_yaml::Value, allowed: &[&str]) {
    for key in keys_of(value) {
        assert!(
            allowed.contains(&key.as_str()),
            "neo-go has no `{section}.{key}` field; `KnownFields(true)` makes this a fatal \
             startup error, so the node would refuse to boot on our generated config",
        );
    }
}

#[test]
fn every_generated_neo_go_key_exists_in_neo_go() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-go", NodeType::NeoGo);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    let config: serde_yaml::Value = serde_yaml::from_str(&rendered.text).unwrap();

    assert_known(
        "ProtocolConfiguration",
        &config["ProtocolConfiguration"],
        PROTOCOL_FIELDS,
    );
    let application = &config["ApplicationConfiguration"];
    assert_known("ApplicationConfiguration", application, APPLICATION_FIELDS);
    for (section, allowed) in NESTED_FIELDS {
        let value = &application[*section];
        if !value.is_null() {
            assert_known(section, value, allowed);
        }
    }
    assert_known(
        "LevelDBOptions",
        &application["DBConfiguration"]["LevelDBOptions"],
        &["DataDirectoryPath", "ReadOnly"],
    );
}

/// neo-go parses these as Go durations. A bare integer is a parse error, and
/// the old generator emitted one for the session lifetime.
#[test]
fn duration_fields_are_go_durations_not_bare_numbers() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-go", NodeType::NeoGo);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    let config: serde_yaml::Value = serde_yaml::from_str(&rendered.text).unwrap();

    let application = &config["ApplicationConfiguration"];
    for path in [
        vec!["RPC", "SessionLifetime"],
        vec!["P2P", "DialTimeout"],
        vec!["P2P", "PingInterval"],
        vec!["P2P", "PingTimeout"],
        vec!["P2P", "ProtoTickInterval"],
    ] {
        let value = &application[path[0]][path[1]];
        let text = value.as_str();
        assert!(
            text.is_some(),
            "{path:?} must be a Go duration string, got {value:?}",
        );
        let text = text.unwrap_or_default();
        assert!(
            text.ends_with('s') || text.ends_with('m') || text.ends_with('h'),
            "{path:?} is {text}, which is not a Go duration",
        );
    }
    assert_eq!(
        config["ProtocolConfiguration"]["TimePerBlock"]
            .as_str()
            .unwrap(),
        "15s",
    );
}

/// The three keys that made every exported neo-go config unbootable: a `Node:`
/// section neo-go has no field for, and `Address`/`Port` pairs where it only
/// accepts an `Addresses` list.
#[test]
fn services_bind_through_address_lists_and_relay_is_flat() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-go", NodeType::NeoGo);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    let config: serde_yaml::Value = serde_yaml::from_str(&rendered.text).unwrap();
    let application = &config["ApplicationConfiguration"];

    assert_eq!(application["RPC"]["Addresses"][0], "127.0.0.1:10332");
    assert_eq!(application["P2P"]["Addresses"][0], "0.0.0.0:10333");
    assert_eq!(application["Relay"], true);
    assert!(application["Node"].is_null());
    for service in ["RPC", "P2P", "Prometheus", "Pprof"] {
        assert!(
            application[service]["Port"].is_null(),
            "{service}.Port is not a neo-go field",
        );
    }
}

/// neo-go refuses to start when `StandbyCommittee` is empty or holds fewer keys
/// than `ValidatorsCount`, so a public-network config must carry the real one.
#[test]
fn a_public_network_config_carries_its_standby_committee() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-go", NodeType::NeoGo);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    let config: serde_yaml::Value = serde_yaml::from_str(&rendered.text).unwrap();

    let committee = config["ProtocolConfiguration"]["StandbyCommittee"]
        .as_sequence()
        .expect("StandbyCommittee must be a list");
    let validators = config["ProtocolConfiguration"]["ValidatorsCount"]
        .as_u64()
        .expect("ValidatorsCount must be a number");
    assert_eq!(committee.len(), 21);
    assert!(committee.len() as u64 >= validators);
}
