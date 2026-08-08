//! Neo X end to end: what the generator writes, what the validator accepts,
//! and the wall between the two chain families.
//!
//! Neo N3 and Neo X share this workspace, this database and most of this
//! codebase, and their settings rhyme — both have a "network", both have
//! bootstrap peers, both have validators. They are not interchangeable: an N3
//! network magic taken as an EVM chain id produces signatures replayable
//! nowhere, and an `enode://` URL in an N3 seed list is a host that cannot
//! answer. Neither runtime would reject the mix-up loudly, so these tests are
//! where it gets caught.

use crate::*;

fn neox_node(repo: &Repository, node_type: NodeType) -> NodeConfig {
    let node_id = create_node(repo, &node_type.to_string(), node_type);
    repo.list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap()
}

/// Everything the generator writes must survive its own validator. A schema
/// that only round-trips for Neo N3 would let a Neo X export ship broken.
#[test]
fn a_generated_neox_config_passes_its_own_validation() {
    let repo = create_repo();
    for node_type in [NodeType::NeoXGeth, NodeType::NeoXReth] {
        let node = neox_node(&repo, node_type);
        let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
        assert_eq!(rendered.format, ConfigFormat::Toml, "{node_type}");

        let report = ConfigValidator::validate_rendered(&node, &rendered);
        assert!(
            report.is_success(),
            "{node_type}: {}",
            report.operator_summary()
        );
    }
}

/// The contamination guard, from the direction that matters: an N3 setting
/// smuggled into a Neo X file must be rejected, not merely ignored the way
/// both EVM clients would ignore it.
#[test]
fn a_neo_n3_setting_in_a_neox_config_is_rejected() {
    let repo = create_repo();
    let node = neox_node(&repo, NodeType::NeoXReth);
    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();

    for injected in [
        "\n[ProtocolConfiguration]\nNetwork = 894710606\n",
        "\nseed_nodes = [\"seed1t5.neo.org:20333\"]\n",
        "\nnetwork_magic = 860833102\n",
    ] {
        let tampered = format!("{}{injected}", rendered.text);
        let report = ConfigValidator::validate_text(&node, ConfigFormat::Toml, &tampered);
        assert!(
            !report.is_success(),
            "a Neo N3 setting survived a Neo X config: {injected}"
        );
        assert!(report.checks.iter().any(|check| {
            check.severity == ConfigValidationSeverity::Critical
                && (check.title == "Chain family" || check.title == "Chain identity")
        }));
    }
}

/// The reverse direction. A Neo X chain id or bootnode reaching an N3 config
/// would put a node on a chain its client cannot speak to at all.
#[test]
fn no_neox_chain_identity_reaches_a_neo_n3_config() {
    let repo = create_repo();
    for node_type in [NodeType::NeoCli, NodeType::NeoGo, NodeType::NeoRs] {
        let node = neox_node(&repo, node_type);
        let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
        for neox_only in ["enode://", "47763", "12227332", "neox-mainnet", "chainId"] {
            assert!(
                !rendered.text.contains(neox_only),
                "{node_type} config carries the Neo X value `{neox_only}`",
            );
        }
    }
}

/// Neo X Geth reads go-ethereum's `gethConfig`, whose `MissingField` hook makes
/// an unrecognised key a fatal startup error — so the generated key names have
/// to be the Go field names exactly.
#[test]
fn the_geth_config_uses_go_field_names() {
    let repo = create_repo();
    let node = neox_node(&repo, NodeType::NeoXGeth);
    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    let value: toml::Value = toml::from_str(&rendered.text).unwrap();

    assert!(value.get("Eth").is_some(), "[Eth] section");
    assert!(value["Eth"].get("NetworkId").is_some());
    assert!(value["Node"].get("HTTPPort").is_some());
    assert!(value["Node"]["P2P"].get("BootstrapNodes").is_some());
    // The lower-case spellings are Reth's, and geth would abort on them.
    assert!(value.get("eth").is_none() && value.get("node").is_none());
}

/// A tampered chain id is the whole ballgame: a Neo X node on the wrong id
/// signs transactions that replay on no chain and reach no peers.
#[test]
fn a_tampered_neox_chain_id_is_rejected() {
    let repo = create_repo();
    let node = neox_node(&repo, NodeType::NeoXGeth);
    let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
    let tampered = rendered.text.replace("12227332", "1");

    let report = ConfigValidator::validate_text(&node, ConfigFormat::Toml, &tampered);
    assert!(!report.is_success());
    assert!(report
        .checks
        .iter()
        .any(|check| check.title == "Chain id"
            && check.severity == ConfigValidationSeverity::Critical));
}

/// Neither Neo X client can be launched from its config file alone, and the
/// gap is silent: geth would write its chain to `~/.ethereum`, and neox-rs
/// would come up on Ethereum mainnet with default ports.
#[test]
fn the_launch_plan_carries_what_the_neox_config_cannot() {
    let repo = create_repo();
    let work_dir = tempfile::tempdir().unwrap();

    let geth = neox_node(&repo, NodeType::NeoXGeth);
    let plan = LaunchPlanner::plan(&geth, work_dir.path().join("neox.toml"), work_dir.path());
    assert!(plan.args.iter().any(|arg| arg == "--datadir"));
    assert!(plan.args.iter().any(|arg| arg == "--config"));

    let reth = neox_node(&repo, NodeType::NeoXReth);
    let plan = LaunchPlanner::plan(&reth, work_dir.path().join("neox.toml"), work_dir.path());
    assert_eq!(plan.args.first().map(String::as_str), Some("node"));
    for flag in ["--chain", "--datadir", "--http.port", "--port", "--config"] {
        assert!(plan.args.iter().any(|arg| arg == flag), "missing {flag}");
    }
    let chain = plan
        .args
        .iter()
        .position(|arg| arg == "--chain")
        .and_then(|index| plan.args.get(index + 1));
    assert_eq!(chain.map(String::as_str), Some("neox-testnet"));
}

/// Plugins are a Neo N3 concept. A Neo X node that offered a plugin surface
/// would be offering assemblies its client never loads.
#[test]
fn neox_nodes_carry_no_plugin_sidecars() {
    let repo = create_repo();
    for node_type in [NodeType::NeoXGeth, NodeType::NeoXReth] {
        let node = neox_node(&repo, node_type);
        assert!(ConfigGenerator::sidecars_for_node(&node, &[]).is_empty());
        assert!(!node_type.family().has_plugins());
    }
}
