//! `--config-drift` against a real workspace: a managed config a launch would
//! write needs no attention, and a hand edit or a missing file is reported.

use super::super::*;

use crate::config::{ConfigExporter, GenerationContext};

/// A workspace with one node, anchored at `home` so the action's children
/// convention (`nodes/` beside the database) lands inside the same tempdir the
/// test controls.
fn drift_workspace(home: &tempfile::TempDir) -> NodeConfig {
    let repository = Repository::open(home.path().join("neonexus.db")).unwrap();
    let node_id = repository
        .create_node(NewNode {
            name: "drift-check".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("neo-node"),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap()
        .id;
    repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap()
}

fn managed_config_path(home: &tempfile::TempDir, node: &NodeConfig) -> PathBuf {
    ConfigExporter::managed_target_path(home.path().join("nodes").join(&node.id), node)
}

fn write_managed_config(home: &tempfile::TempDir, node: &NodeConfig) -> PathBuf {
    let path = managed_config_path(home, node);
    ConfigExporter::write_node_config_to_path_with_context(
        &path,
        node,
        &[],
        None,
        &GenerationContext::default(),
    )
    .unwrap();
    path
}

fn drift_json_action(db_path: &Path) -> Result<(i32, serde_json::Value)> {
    let action = action_from_args([
        "neo-nexus",
        "--config-drift-json",
        &db_path.display().to_string(),
    ])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected config drift JSON action");
    };
    let value: serde_json::Value = serde_json::from_str(&text)?;
    Ok((exit_code, value))
}

#[test]
fn a_managed_config_a_launch_would_write_needs_no_attention() {
    let home = tempfile::tempdir().unwrap();
    let node = drift_workspace(&home);
    write_managed_config(&home, &node);

    let (exit_code, value) = drift_json_action(&home.path().join("neonexus.db")).unwrap();

    assert_eq!(exit_code, 0);
    assert_eq!(value["success"], true);
    assert_eq!(value["attention_count"], 0);
    assert_eq!(value["nodes"][0]["status"], "ok");
    assert!(value["nodes"][0]["findings"].as_array().unwrap().is_empty());
}

/// The version-upgrade case: the file on disk carries a line a fresh render
/// would not write — a hand edit, or a legacy setting a newer node version
/// dropped. That is attention, with the offending line quoted.
#[test]
fn a_disk_config_the_workspace_would_not_write_is_attention() {
    let home = tempfile::tempdir().unwrap();
    let node = drift_workspace(&home);
    let config_path = write_managed_config(&home, &node);
    let rendered = std::fs::read_to_string(&config_path).unwrap();
    std::fs::write(&config_path, format!("{rendered}legacy_setting = true\n")).unwrap();

    let (exit_code, value) = drift_json_action(&home.path().join("neonexus.db")).unwrap();

    assert_eq!(exit_code, 1);
    assert_eq!(value["attention_count"], 1);
    assert_eq!(value["nodes"][0]["status"], "attention");
    let findings = value["nodes"][0]["findings"].as_array().unwrap();
    let drift = findings
        .iter()
        .find(|finding| finding["kind"] == "drift")
        .expect("a drift finding");
    assert!(
        drift["detail"]
            .as_str()
            .unwrap()
            .contains("legacy_setting: [value redacted]"),
        "the unexpected field is identified without its value: {}",
        drift["detail"]
    );
}

#[test]
fn a_node_with_no_config_on_disk_is_reported_missing() {
    let home = tempfile::tempdir().unwrap();
    drift_workspace(&home);

    let (exit_code, value) = drift_json_action(&home.path().join("neonexus.db")).unwrap();

    assert_eq!(exit_code, 1);
    assert_eq!(value["nodes"][0]["findings"][0]["kind"], "missing");
}
