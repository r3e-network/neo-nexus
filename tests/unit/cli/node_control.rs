use super::super::*;

use std::path::PathBuf;

use crate::{
    repository::Repository,
    types::{Network, NewNode, NodeStatus, NodeType, StorageEngine},
};

#[path = "../../support/stub_runtime.rs"]
mod stub_runtime;

fn controllable_long_running_command() -> (PathBuf, Vec<String>) {
    (stub_runtime::stub_runtime_binary(), Vec::new())
}

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

#[test]
fn node_stop_cli_does_not_signal_or_mark_stopped_on_pid_identity_mismatch() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    let node = repository.create_node(NewNode {
        name: "reused pid".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/definitely/not/the-test-process/neo-node".into(),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 25332,
        p2p_port: 25333,
        ws_port: None,
    })?;
    let unrelated_pid = std::process::id();
    repository.update_node_status(&node.id, NodeStatus::Running, Some(unrelated_pid))?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--node-stop", &db_arg, "reused pid"])?;
    assert!(
        matches!(action, CliAction::PrintWithExitCode { ref text, exit_code: 1 }
            if text.contains("different process") && text.contains("status unchanged")),
        "identity mismatch must fail closed: {action:?}"
    );
    let stored = Repository::open(&db_path)?
        .list_nodes()?
        .into_iter()
        .find(|stored| stored.id == node.id)
        .expect("node remains registered");
    assert_eq!(stored.status, NodeStatus::Running);
    assert_eq!(stored.pid, Some(unrelated_pid));
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

#[test]
fn node_restart_cli_quiesces_the_recorded_process_before_spawning() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    let (binary, args) = controllable_long_running_command();
    let mut original = std::process::Command::new(&binary).args(&args).spawn()?;
    let original_pid = original.id();
    let node = repository.create_node(NewNode {
        name: "restart witness".to_string(),
        node_type: NodeType::NeoCli,
        network: Network::Testnet,
        binary_path: binary,
        args,
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port: 34332,
        p2p_port: 34333,
        ws_port: None,
    })?;
    repository.update_node_status(&node.id, NodeStatus::Running, Some(original_pid))?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let restart = action_from_args(["neo-nexus", "--node-restart", &db_arg, "restart witness"]);
    let stored = Repository::open(&db_path)?
        .list_nodes()?
        .into_iter()
        .find(|stored| stored.id == node.id)
        .expect("restart witness remains registered");
    let original_is_alive = crate::supervisor::process_is_live(original_pid);

    // Always clean up both possible generations before asserting, so a failed
    // regression cannot leave a two-minute witness behind on the test host.
    if stored.pid.is_some() {
        let _ = action_from_args(["neo-nexus", "--node-stop", &db_arg, "restart witness"]);
    }
    let _ = original.kill();
    let _ = original.wait();

    let restart = restart?;
    assert!(
        matches!(restart, CliAction::PrintWithExitCode { exit_code: 0, .. }),
        "restart should launch one replacement: {restart:?}"
    );
    assert!(
        !original_is_alive,
        "the original pid {original_pid} survived CLI restart"
    );
    assert_eq!(stored.status, NodeStatus::Running);
    assert_ne!(stored.pid, Some(original_pid));
    Ok(())
}

#[path = "node_control/report.rs"]
mod report;
