use std::path::PathBuf;

use super::*;
use crate::{
    launch::LaunchPlan,
    types::{Network, NodeStatus, NodeType, StorageEngine},
};

#[test]
fn start_and_restart_refuse_an_unbound_imported_runtime_before_spawn() {
    let node = NodeConfig {
        id: "node-imported".to_string(),
        name: "imported".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::new(),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    };
    let plan = LaunchPlan {
        // If the guard used only the plan, this executable could be launched.
        binary_path: std::env::current_exe().expect("test executable path"),
        args: Vec::new(),
        working_dir: std::env::temp_dir(),
        managed_config_path: None,
        display_command: "test executable".to_string(),
    };
    let temp_dir = tempfile::tempdir().expect("temporary log directory");
    let log_path = temp_dir.path().join("unbound-node-must-not-launch.log");
    let mut supervisor = ProcessSupervisor::default();

    let start = supervisor
        .start(&node, &plan, &log_path)
        .expect_err("unbound node start must fail");
    assert!(start.to_string().contains("no trusted local runtime"));
    let restart = supervisor
        .restart(&node, &plan, &log_path)
        .expect_err("unbound node restart must fail");
    assert!(restart.to_string().contains("no trusted local runtime"));
    assert!(!log_path.exists());
}

#[test]
fn a_pid_line_the_child_beat_onto_the_log_is_not_quoted_as_the_childs_words() {
    // The losing order: the child printed its reason before the supervisor
    // could append the pid, so the pid line trails the child's own output.
    let temp_dir = tempfile::tempdir().expect("temporary log directory");
    let log_path = temp_dir.path().join("child-first.log");
    std::fs::write(
        &log_path,
        "== NeoNexus launch 0 ==\npid: 7\nstub runtime rejected its arguments\npid: 42\n",
    )
    .expect("write the log");

    let quoted = read_startup_output(
        &log_path,
        "== NeoNexus launch 0 ==\npid: 7\n".len() as u64,
        42,
    );
    assert_eq!(quoted, "stub runtime rejected its arguments");
}

#[test]
fn a_pid_line_written_before_the_child_is_not_quoted_as_the_childs_output() {
    // The common order: the pid line lands first, the child's reason after.
    let temp_dir = tempfile::tempdir().expect("temporary log directory");
    let log_path = temp_dir.path().join("pid-first.log");
    std::fs::write(
        &log_path,
        "== NeoNexus launch 0 ==\npid: 7\npid: 42\nstub runtime rejected its arguments\n",
    )
    .expect("write the log");

    let quoted = read_startup_output(
        &log_path,
        "== NeoNexus launch 0 ==\npid: 7\n".len() as u64,
        42,
    );
    assert_eq!(quoted, "stub runtime rejected its arguments");
}

#[test]
fn a_pid_prefixed_child_diagnostic_is_preserved() {
    let temp_dir = tempfile::tempdir().expect("temporary log directory");
    let log_path = temp_dir.path().join("child-pid-diagnostic.log");
    std::fs::write(&log_path, "pid: 42\npid: 9001 failed to bind port\n").expect("write log");
    assert_eq!(
        read_startup_output(&log_path, 0, 42),
        "pid: 9001 failed to bind port"
    );
}

#[test]
fn an_unrelated_numeric_pid_line_is_child_output() {
    let temp_dir = tempfile::tempdir().expect("temporary log directory");
    let log_path = temp_dir.path().join("child-numeric-pid.log");
    std::fs::write(&log_path, "pid: 42\npid: 9001\n").expect("write log");
    assert_eq!(read_startup_output(&log_path, 0, 42), "pid: 9001");
}
