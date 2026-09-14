use super::*;

use std::path::PathBuf;

use crate::{
    config::{format::ConfigFormat, ConfigGenerator, ConfigValidationSeverity, ConfigValidator},
    types::{NodeStatus, NodeType, StorageEngine},
};

/// A bare report to push findings into. `ConfigValidationReport` has no
/// constructor because production only ever builds one through the validator.
fn empty_report(node_type: NodeType) -> crate::config::ConfigValidationReport {
    crate::config::ConfigValidationReport {
        node_type,
        format: ConfigFormat::Json,
        checks: Vec::new(),
    }
}

fn rank(severity: ConfigValidationSeverity) -> u8 {
    match severity {
        ConfigValidationSeverity::Pass => 0,
        ConfigValidationSeverity::Warning => 1,
        ConfigValidationSeverity::Critical => 2,
    }
}

fn node(node_type: NodeType, network: Network) -> NodeConfig {
    NodeConfig {
        id: "node-1".to_string(),
        name: "node-1".to_string(),
        node_type,
        network,
        binary_path: PathBuf::from("/opt/neo/node"),
        args: Vec::new(),
        runtime_version: "1.0".to_string(),
        storage_engine: match node_type {
            NodeType::NeoRs => StorageEngine::RocksDb,
            _ => StorageEngine::LevelDb,
        },
        rpc_port: 30332,
        p2p_port: 30333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

fn findings(
    node_type: NodeType,
    network: Network,
) -> Vec<(&'static str, ConfigValidationSeverity, String)> {
    let node = node(node_type, network);
    let rendered = ConfigGenerator::render_for_node(&node, &[]).expect("the config renders");
    ConfigValidator::validate_rendered(&node, &rendered)
        .checks
        .into_iter()
        .map(|check| (check.title, check.severity, check.detail))
        .collect()
}

/// One check by its exact title. `starts_with` is not enough: the drift check
/// beside this one is titled "Seed list matches", and matching the prefix would
/// silently assert against the wrong finding.
fn finding(
    node_type: NodeType,
    network: Network,
    title: &str,
) -> Option<(ConfigValidationSeverity, String)> {
    findings(node_type, network)
        .into_iter()
        .find(|(found, _, _)| *found == title)
        .map(|(_, severity, detail)| (severity, detail))
}

fn worst(node_type: NodeType, network: Network) -> ConfigValidationSeverity {
    findings(node_type, network)
        .into_iter()
        .map(|(_, severity, _)| severity)
        .max_by_key(|severity| rank(*severity))
        .unwrap_or(ConfigValidationSeverity::Pass)
}

/// **The reproduction the register recorded.**
///
/// Every one of these reported "ready" with zero findings, over a config that
/// described a node which could not join anything: neo-go private rendered
/// `SeedList: []` and `StandbyCommittee: []` and was called ready with 10
/// passes. The existing checks compare the file against what this workspace
/// would generate, which cannot catch a workspace generating an unusable file —
/// `effective_seed_nodes(Private, None)` is empty, and `len >= 0` passes.
#[test]
fn a_private_node_with_no_seeds_is_no_longer_called_ready() {
    for node_type in [NodeType::NeoGo, NodeType::NeoRs, NodeType::NeoCli] {
        let Some(seed_finding) = finding(node_type, Network::Private, "Seed list") else {
            unreachable!("{node_type} reported nothing at all about its seed list")
        };
        assert_eq!(
            seed_finding.0,
            ConfigValidationSeverity::Warning,
            "{node_type}: {}",
            seed_finding.1
        );
        assert_ne!(
            worst(node_type, Network::Private),
            ConfigValidationSeverity::Pass,
            "{node_type} private still reports clean"
        );
    }
}

/// A public network is genuinely ready, and must stay silent.
///
/// A check that fires on every correctly-configured node teaches operators to
/// ignore it, which costs more than the problem it reports.
#[test]
fn a_public_node_reports_nothing_because_nothing_is_wrong_with_it() {
    for node_type in [NodeType::NeoGo, NodeType::NeoRs, NodeType::NeoCli] {
        for network in [Network::Mainnet, Network::Testnet] {
            assert_eq!(
                worst(node_type, network),
                ConfigValidationSeverity::Pass,
                "{node_type} on {network} reported: {:?}",
                findings(node_type, network)
            );
        }
    }
}

/// **The worst case, fixed at the generator.**
///
/// neo-cli's `ProtocolConfiguration` was `{"Network": 1230000}` and nothing
/// else, because the generator wrote the identity keys only under a profile. A
/// client that finds no seed list and no committee uses the ones it was
/// compiled with — the **public** ones — so the node dialled MainNet seeds and
/// trusted the MainNet committee while carrying a private magic. It joined the
/// wrong chain rather than no chain, silently.
#[test]
fn neo_cli_always_writes_the_identity_keys_rather_than_inheriting_public_ones() {
    for network in [Network::Mainnet, Network::Testnet, Network::Private] {
        let node = node(NodeType::NeoCli, network);
        let rendered = ConfigGenerator::render_for_node(&node, &[]).expect("the config renders");
        let value: serde_json::Value =
            serde_json::from_str(&rendered.text).expect("neo-cli config is JSON");
        let protocol = &value["ProtocolConfiguration"];
        for key in ["SeedList", "ValidatorsCount", "StandbyCommittee"] {
            assert!(
                !protocol[key].is_null(),
                "{network}: {key} is absent, so neo-cli would use its compiled-in value"
            );
        }
    }
}

/// neo-rs takes no committee in its config — the client holds its own — so an
/// absent one there is the schema, not an omission. Reporting it would be a
/// finding an operator could never clear.
#[test]
fn a_client_that_cannot_express_a_committee_is_not_faulted_for_lacking_one() {
    let committee = finding(NodeType::NeoRs, Network::Private, "Standby committee");
    assert!(
        committee
            .as_ref()
            .is_none_or(|(severity, _)| *severity == ConfigValidationSeverity::Pass),
        "neo-rs was faulted for a key its config format does not have: {committee:?}"
    );
}

/// A committee smaller than the validators it must elect is a node that refuses
/// to start. The check reads both numbers out of the same file rather than
/// against an expectation, so it catches a hand-edited config too.
#[test]
fn a_committee_too_small_for_its_validator_count_is_critical() {
    let mut report = empty_report(NodeType::NeoCli);
    check_chain_identity(
        &mut report,
        &node(NodeType::NeoCli, Network::Private),
        &ChainIdentity {
            seed_count: Some(3),
            committee_count: Some(1),
            validators_count: Some(4),
            committee_is_expressible: true,
        },
    );
    let committee = report
        .checks
        .iter()
        .find(|check| check.title == "Standby committee")
        .expect("the committee is checked");
    assert_eq!(committee.severity, ConfigValidationSeverity::Critical);
    assert!(committee.detail.contains("cannot elect more validators"));
}

/// An **absent** key and an **empty** key are different failures and get
/// different severities, because a critical finding stops the config being
/// written at all.
///
/// Absent means the client falls back to its public defaults and joins the
/// wrong network — wrong, and worth refusing. Empty means the node will not
/// sync, which is serious but is also exactly right for a single-node private
/// chain that produces its own blocks; refusing that would strand a legitimate
/// setup.
#[test]
fn an_absent_key_is_critical_where_an_empty_one_is_only_a_warning() {
    let private = node(NodeType::NeoCli, Network::Private);

    let mut absent = empty_report(NodeType::NeoCli);
    check_chain_identity(
        &mut absent,
        &private,
        &ChainIdentity {
            seed_count: None,
            committee_count: None,
            validators_count: None,
            committee_is_expressible: true,
        },
    );
    assert!(absent
        .checks
        .iter()
        .all(|check| check.severity == ConfigValidationSeverity::Critical));

    let mut empty = empty_report(NodeType::NeoCli);
    check_chain_identity(
        &mut empty,
        &private,
        &ChainIdentity {
            seed_count: Some(0),
            committee_count: Some(0),
            validators_count: Some(0),
            committee_is_expressible: true,
        },
    );
    assert!(empty
        .checks
        .iter()
        .all(|check| check.severity == ConfigValidationSeverity::Warning));
}
