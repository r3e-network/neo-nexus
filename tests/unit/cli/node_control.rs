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
            if text.contains("not started") && text.contains("readiness blocked"))
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

/// `--node-restart` on a node that is not running refuses to restart (mirrors
/// the GUI's guard), proving the CLI restart path is reached.
#[test]
fn node_restart_cli_refuses_when_not_running() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    repository.create_node(NewNode {
        name: "stopped node".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/opt/neo-rs/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 30332,
        p2p_port: 30333,
        ws_port: None,
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--node-restart", &db_arg, "stopped node"])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 1 }
            if text.contains("must be running before restart"))
    );
    Ok(())
}

/// `--node-list` prints a compact table of all nodes, with a header row, so a
/// script can parse fleet status headlessly. An empty workspace prints a clear
/// "no nodes" message.
#[test]
fn node_list_cli_prints_fleet_table() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    repository.create_node(NewNode {
        name: "alpha".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Mainnet,
        binary_path: "/opt/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 40332,
        p2p_port: 40333,
        ws_port: None,
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--node-list", &db_arg])?;

    match action {
        CliAction::PrintWithExitCode { text, exit_code } => {
            assert_eq!(exit_code, 0, "node-list should succeed");
            assert!(text.contains("NAME"), "table should have a header");
            assert!(text.contains("alpha"), "table should list the node");
            assert!(
                text.contains("mainnet"),
                "table should show the node's network"
            );
        }
        other => panic!("expected PrintWithExitCode, got {other:?}"),
    }
    Ok(())
}

/// `--node-list` on an empty workspace reports no nodes rather than printing an
/// empty table.
#[test]
fn node_list_cli_reports_empty_workspace() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    Repository::open(&db_path)?;
    drop(Repository::open(&db_path)?);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--node-list", &db_arg])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 0 }
            if text.contains("No nodes"))
    );
    Ok(())
}

/// `--node-status` prints a detailed single-node report: identity, status,
/// ports, version, and the RPC-health section (which reads through the core
/// operation, not the repository). An unknown node errors cleanly.
#[test]
fn node_status_cli_prints_detailed_report() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    repository.create_node(NewNode {
        name: "alpha".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Mainnet,
        binary_path: "/opt/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 50332,
        p2p_port: 50333,
        ws_port: Some(50334),
    })?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--node-status", &db_arg, "alpha"])?;

    match action {
        CliAction::PrintWithExitCode { text, exit_code } => {
            assert_eq!(exit_code, 0, "node-status should succeed");
            assert!(
                text.contains("Name:    alpha"),
                "report should name the node"
            );
            assert!(
                text.contains("RPC:     50332"),
                "report should show the RPC port"
            );
            assert!(
                text.contains("WS:      50334"),
                "report should show the WS port"
            );
            assert!(
                text.contains("RPC health:"),
                "report should include the RPC health section"
            );
        }
        other => panic!("expected PrintWithExitCode, got {other:?}"),
    }
    Ok(())
}
