//! The validation matrix for the node editor. These are the rules that decide
//! whether an operator's form comes back with a reason or silently saves, so
//! each case is stated once here rather than inferred from an HTTP round trip.

use crate::{
    core::node::{Network, NodeConfig, NodeStatus, NodeType, StorageEngine},
    web::node_form::{DraftOutcome, NodeDraft},
};

use std::path::PathBuf;

fn node(
    id: &str,
    name: &str,
    node_type: NodeType,
    rpc: u16,
    p2p: u16,
    ws: Option<u16>,
) -> NodeConfig {
    NodeConfig {
        id: id.to_string(),
        name: name.to_string(),
        node_type,
        network: Network::Mainnet,
        binary_path: PathBuf::from("/opt/neo/node"),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: node_type.default_storage_engine(),
        rpc_port: rpc,
        p2p_port: p2p,
        ws_port: ws,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

fn draft(name: &str, client: &str) -> NodeDraft {
    NodeDraft {
        name: name.to_string(),
        node_type: client.to_string(),
        network: Network::Mainnet.to_string(),
        binary_path: "/opt/neo/node".to_string(),
        storage_engine: NodeType::ALL
            .iter()
            .find(|node_type| node_type.to_string() == client)
            .map_or_else(
                || StorageEngine::RocksDb.to_string(),
                |node_type| node_type.default_storage_engine().to_string(),
            ),
        rpc_port: "30332".to_string(),
        p2p_port: "30333".to_string(),
        ..NodeDraft::default()
    }
}

fn accepted(outcome: DraftOutcome) -> NewNodeCase {
    match outcome {
        DraftOutcome::Valid(input) => NewNodeCase::ok(input),
        DraftOutcome::Invalid(errors) => NewNodeCase::rejected(errors),
    }
}

/// A result that can be asserted either way without panicking, so one test can
/// check several fields at once.
struct NewNodeCase {
    accepted: Option<crate::core::node::NewNode>,
    fields: Vec<&'static str>,
}

impl NewNodeCase {
    fn ok(input: crate::core::node::NewNode) -> Self {
        Self {
            accepted: Some(input),
            fields: Vec::new(),
        }
    }

    fn rejected(errors: crate::web::node_form::FieldErrors) -> Self {
        Self {
            accepted: None,
            fields: errors.keys().copied().collect(),
        }
    }

    fn was_accepted(&self) -> bool {
        self.accepted.is_some()
    }

    fn node(&self) -> &crate::core::node::NewNode {
        self.accepted
            .as_ref()
            .expect("a validated draft should be accepted")
    }

    fn flagged(&self, field: &str) -> bool {
        self.fields.contains(&field)
    }
}

#[test]
fn a_complete_draft_is_accepted() {
    let case = accepted(draft("seed-1", "neo-go").validate(&[], None));
    assert!(case.was_accepted(), "fields: {:?}", case.fields);
    assert_eq!(case.node().name, "seed-1");
    assert_eq!(case.node().rpc_port, 30332);
    assert_eq!(
        case.node().ws_port,
        None,
        "blank WebSocket means none, not zero"
    );
}

#[test]
fn surrounding_whitespace_is_trimmed_but_inner_spaces_survive() {
    let case = accepted(draft("  relay two  ", "neo-go").validate(&[], None));
    assert!(case.was_accepted());
    assert_eq!(case.node().name, "relay two");
}

#[test]
fn a_blank_name_is_reported_against_the_name_field() {
    let case = accepted(draft("   ", "neo-go").validate(&[], None));
    assert!(!case.was_accepted());
    assert!(case.flagged("name"), "fields: {:?}", case.fields);
}

#[test]
fn a_duplicate_name_is_refused_case_insensitively() {
    let fleet = vec![node(
        "node-1",
        "Seed-One",
        NodeType::NeoGo,
        30332,
        30333,
        None,
    )];
    let case = accepted(draft("seed-one", "neo-rs").validate(&fleet, None));
    assert!(!case.was_accepted());
    assert!(case.flagged("name"), "fields: {:?}", case.fields);
}

#[test]
fn editing_a_node_does_not_collide_with_its_own_name() {
    let id = "node-1";
    let fleet = vec![node(id, "seed-1", NodeType::NeoGo, 30332, 30333, None)];
    let mut form = draft("seed-1", "neo-go");
    form.rpc_port = "30340".to_string();
    form.p2p_port = "30341".to_string();
    let case = accepted(form.validate(&fleet, Some(id)));
    assert!(case.was_accepted(), "fields: {:?}", case.fields);
}

#[test]
fn each_client_only_offers_the_storage_it_can_run() {
    // neo-rs is RocksDB only, neo-go is LevelDB only, neo-cli takes either.
    let cases = [
        ("neo-rs", StorageEngine::LevelDb, false),
        ("neo-rs", StorageEngine::RocksDb, true),
        ("neo-go", StorageEngine::RocksDb, false),
        ("neo-go", StorageEngine::LevelDb, true),
        ("neo-cli", StorageEngine::LevelDb, true),
        ("neox-geth", StorageEngine::LevelDb, false),
    ];
    for (client, storage, should_pass) in cases {
        let mut form = draft("probe", client);
        form.storage_engine = storage.to_string();
        let case = accepted(form.validate(&[], None));
        assert_eq!(
            case.was_accepted(),
            should_pass,
            "{client} with {storage}: fields {:?}",
            case.fields
        );
        if !should_pass {
            assert!(case.flagged("storage_engine"));
        }
    }
}

#[test]
fn storage_is_only_a_choice_where_the_client_offers_one() {
    let neo_rs = draft("probe", "neo-rs");
    assert!(!neo_rs.storage_is_selectable(), "neo-rs has one engine");
    assert!(neo_rs.storage_note().is_some());

    let neo_cli = draft("probe", "neo-cli");
    assert!(neo_cli.storage_is_selectable(), "neo-cli has two engines");
    assert!(neo_cli.storage_note().is_none());

    let mut neox = draft("probe", "neox-geth");
    neox.storage_engine = StorageEngine::RocksDb.to_string();
    assert!(!neox.storage_is_selectable());
    let note = neox
        .storage_note()
        .expect("a built-in store should be explained");
    assert!(note.contains("Pebble"), "unexpected note: {note}");
}

#[test]
fn an_unknown_client_is_refused_rather_than_defaulted() {
    let case = accepted(draft("probe", "neo-somewhat").validate(&[], None));
    assert!(!case.was_accepted());
    assert!(case.flagged("node_type"), "fields: {:?}", case.fields);
}

#[test]
fn ports_must_be_numbers_above_zero() {
    let mut form = draft("probe", "neo-go");
    form.rpc_port = "http".to_string();
    let case = accepted(form.validate(&[], None));
    assert!(case.flagged("rpc_port"), "fields: {:?}", case.fields);

    let mut form = draft("probe", "neo-go");
    form.p2p_port = "0".to_string();
    let case = accepted(form.validate(&[], None));
    assert!(case.flagged("p2p_port"), "fields: {:?}", case.fields);

    let mut form = draft("probe", "neo-go");
    form.ws_port = "99999999".to_string();
    let case = accepted(form.validate(&[], None));
    assert!(case.flagged("ws_port"), "fields: {:?}", case.fields);
}

#[test]
fn one_node_cannot_bind_the_same_port_twice() {
    // A message naming two ports is filed under the second one: that is the port
    // duplicating the field above it.
    for (field, rpc, p2p, ws) in [
        ("p2p_port", "30332", "30332", ""),
        ("ws_port", "30332", "30333", "30332"),
        ("ws_port", "30332", "30333", "30333"),
    ] {
        let mut form = draft("probe", "neo-go");
        form.rpc_port = rpc.to_string();
        form.p2p_port = p2p.to_string();
        form.ws_port = ws.to_string();
        let case = accepted(form.validate(&[], None));
        assert!(!case.was_accepted(), "{rpc}/{p2p}/{ws} should be refused");
        assert!(
            case.flagged(field),
            "expected {field}, got {:?}",
            case.fields
        );
    }
}

#[test]
fn a_port_already_held_by_another_node_is_refused_and_named() {
    let fleet = vec![node(
        "node-9",
        "rpc-front",
        NodeType::NeoGo,
        30332,
        40333,
        None,
    )];
    let case = accepted(draft("probe", "neo-rs").validate(&fleet, None));
    assert!(!case.was_accepted());
    assert!(case.flagged("rpc_port"), "fields: {:?}", case.fields);
}

#[test]
fn a_port_colliding_only_across_nodes_is_still_caught() {
    // New node's P2P reusing another node's RPC: legal within one node, fatal
    // between two.
    let fleet = vec![node(
        "node-9",
        "rpc-front",
        NodeType::NeoGo,
        30333,
        41333,
        None,
    )];
    let mut form = draft("probe", "neo-rs");
    form.rpc_port = "30332".to_string();
    form.p2p_port = "30333".to_string();
    let case = accepted(form.validate(&fleet, None));
    assert!(!case.was_accepted(), "cross collision should be refused");
}

#[test]
fn an_unterminated_quote_in_arguments_is_reported_not_stored() {
    let mut form = draft("probe", "neo-go");
    form.args = "--data-dir \"/var/lib/neo".to_string();
    let case = accepted(form.validate(&[], None));
    assert!(!case.was_accepted());
    assert!(case.flagged("args"), "fields: {:?}", case.fields);
}

#[test]
fn quoted_arguments_keep_their_spaces() {
    let mut form = draft("probe", "neo-go");
    form.args = r#"--data-dir "/var/lib/neo node""#.to_string();
    let case = accepted(form.validate(&[], None));
    assert!(case.was_accepted(), "fields: {:?}", case.fields);
    assert_eq!(
        case.node().args,
        vec!["--data-dir".to_string(), "/var/lib/neo node".to_string()]
    );
}

#[test]
fn a_blank_runtime_version_means_latest() {
    let mut form = draft("probe", "neo-go");
    form.runtime_version = "   ".to_string();
    let case = accepted(form.validate(&[], None));
    assert_eq!(case.node().runtime_version, "latest");
}

#[test]
fn suggested_ports_avoid_every_port_the_fleet_holds() {
    let fleet = vec![
        node("node-1", "a", NodeType::NeoGo, 30332, 30333, Some(30334)),
        node("node-2", "b", NodeType::NeoGo, 30335, 30336, Some(30337)),
    ];
    let form = draft("newcomer", "neo-rs");
    let suggested = form
        .suggest_ports(&fleet, None)
        .expect("the planner should find a free block");

    let held = fleet
        .iter()
        .flat_map(|node| [Some(node.rpc_port), Some(node.p2p_port), node.ws_port])
        .flatten()
        .collect::<Vec<u16>>();
    // The suggestion comes back as a draft, so its ports are still text.
    let suggested_ports = [suggested.rpc_port.as_str(), suggested.p2p_port.as_str()]
        .iter()
        .filter_map(|raw| raw.trim().parse::<u16>().ok())
        .chain(suggested.ws_port.trim().parse::<u16>().ok())
        .collect::<Vec<u16>>();
    assert!(
        suggested_ports.len() >= 2,
        "the planner should produce numeric RPC and P2P ports, got {:?}",
        suggested_ports
    );
    for port in suggested_ports {
        assert!(!held.contains(&port), "suggested {port} is already held");
    }
    assert_ne!(suggested.rpc_port, suggested.p2p_port);
}

#[test]
fn suggesting_ports_leaves_the_other_fields_alone() {
    let form = draft("keep-me", "neo-go");
    let suggested = form
        .clone()
        .suggest_ports(&[], None)
        .expect("an empty fleet always has room");
    assert_eq!(suggested.name, form.name);
    assert_eq!(suggested.node_type, form.node_type);
    assert_eq!(suggested.binary_path, form.binary_path);
}

#[test]
fn a_client_switch_carries_the_storage_it_can_actually_use() {
    let mut form = draft("probe", "neo-go");
    form.storage_engine = StorageEngine::LevelDb.to_string();
    // Moving to neo-rs, which has no LevelDB, must not leave an impossible value
    // sitting in the form.
    form.node_type = "neo-rs".to_string();
    let moved = form.with_client_defaults();
    assert_eq!(moved.storage_engine, StorageEngine::RocksDb.to_string());
    assert!(accepted(moved.validate(&[], None)).was_accepted());
}

#[test]
fn a_fresh_form_arrives_filled_enough_to_save() {
    // Everything inferable is prefilled, so the only blanks an operator meets are
    // the two things only they can know: what to call it, and where the binary is.
    let blank = NodeDraft::blank();
    assert!(
        blank.name.trim().is_empty(),
        "only the operator can name a node"
    );
    assert!(
        !blank.node_type.trim().is_empty(),
        "a client is preselected"
    );
    assert!(!blank.network.trim().is_empty(), "a network is preselected");
    assert!(
        !blank.storage_engine.trim().is_empty(),
        "storage follows the client"
    );
    assert!(!blank.rpc_port.trim().is_empty(), "RPC is prefilled");
    assert!(
        !blank.p2p_port.trim().is_empty(),
        "P2P is prefilled alongside RPC"
    );
    assert_ne!(
        blank.rpc_port, blank.p2p_port,
        "the defaults must not collide with each other"
    );
    assert!(
        blank.binary_path.trim().is_empty(),
        "only the path is unknowable"
    );
}
