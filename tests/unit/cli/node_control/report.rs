//! The read-only node actions: the fleet table and the single-node report.
//!
//! Split from the lifecycle actions for the same reason the source is —
//! these only read.

use super::super::*;

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
        other => anyhow::bail!("expected PrintWithExitCode, got {other:?}"),
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
        other => anyhow::bail!("expected PrintWithExitCode, got {other:?}"),
    }
    Ok(())
}
