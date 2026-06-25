use super::super::*;

use crate::{
    repository::Repository,
    types::{Network, NewNode, NodeType, StorageEngine},
};

/// `--node-start` runs the same core launch pipeline as the GUI. A node whose
/// binary cannot be resolved is a readiness block, so the CLI must report it as
/// blocked (exit 1) without spawning anything. This proves the extracted core is
/// actually reached from the CLI path.
#[test]
fn node_start_cli_reports_readiness_block_for_missing_binary() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    repository.create_node(NewNode {
        name: "neo-rs blocked".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/definitely/missing/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--node-start", &db_arg, "neo-rs blocked"])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 1 }
            if text.contains("not started") && text.contains("launch readiness blocked"))
    );
    Ok(())
}

/// `--node-start` with an unknown node name surfaces a clear error rather than
/// panicking, so a script gets actionable feedback.
#[test]
fn node_start_cli_rejects_unknown_node_name() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let db_arg = db_path.display().to_string();
    Repository::open(&db_path)?;

    let result = action_from_args(["neo-nexus", "--node-start", &db_arg, "no-such-node"]);
    assert!(result.is_err(), "unknown node should error");
    assert!(
        result.unwrap_err().to_string().contains("no node named"),
        "error should name the missing node"
    );
    Ok(())
}

/// `--node-stop` on a workspace whose node is not running reports it was not
/// running rather than failing, so the command is idempotent for scripts.
#[test]
fn node_stop_cli_reports_not_running() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    repository.create_node(NewNode {
        name: "idle node".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/opt/neo-rs/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 20332,
        p2p_port: 20333,
        ws_port: None,
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--node-stop", &db_arg, "idle node"])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 0 }
            if text.contains("was not running"))
    );
    Ok(())
}
