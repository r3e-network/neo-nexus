//! Identity matching for pid-based stops. Getting this wrong means signalling a
//! process that has nothing to do with the node, so the refusals matter as much
//! as the matches.

use std::path::{Path, PathBuf};

use crate::{
    supervisor::{recorded_process, termination::stop_by_pid, PidStop, RecordedProcess},
    types::NodeConfig,
};

/// A process that stays alive long enough to be classified.
fn spawn_witness() -> std::process::Child {
    let (binary, args): (&str, &[&str]) = if cfg!(windows) {
        (r"C:\Windows\System32\ping.exe", &["-n", "60", "127.0.0.1"])
    } else {
        ("/bin/sleep", &["60"])
    };
    std::process::Command::new(binary)
        .args(args)
        .spawn()
        .expect("a witness process is needed to classify a live pid")
}

fn node_recorded_as(binary: &str, pid: Option<u32>) -> NodeConfig {
    NodeConfig {
        id: "node-1".to_string(),
        name: "witness".to_string(),
        node_type: crate::types::NodeType::NeoGo,
        network: crate::types::Network::Testnet,
        binary_path: PathBuf::from(binary),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: crate::types::StorageEngine::LevelDb,
        rpc_port: 33_332,
        p2p_port: 33_333,
        ws_port: None,
        status: crate::types::NodeStatus::Running,
        pid,
    }
}

#[test]
fn a_node_with_no_recorded_pid_is_simply_gone() {
    assert_eq!(
        recorded_process(&node_recorded_as("/bin/sleep", None)),
        RecordedProcess::Gone
    );
}

#[test]
fn a_pid_that_nothing_answers_is_gone() {
    // A pid far outside any plausible allocation.
    assert_eq!(
        recorded_process(&node_recorded_as("/bin/sleep", Some(4_000_000))),
        RecordedProcess::Gone
    );
}

#[test]
fn a_live_process_is_ours_only_when_the_binary_matches() {
    let mut witness = spawn_witness();
    let pid = witness.id();
    let binary = if cfg!(windows) {
        r"C:\Windows\System32\ping.exe"
    } else {
        "/bin/sleep"
    };

    let ours = node_recorded_as(binary, Some(pid));
    assert_eq!(
        recorded_process(&ours),
        RecordedProcess::Alive,
        "the live witness must be recognised as our own node"
    );

    // Same pid, different recorded binary: the number was recycled, so a
    // blanket "it is running" would be wrong and a kill would hit a stranger.
    let recycled = node_recorded_as("/opt/someone/else/entirely", Some(pid));
    assert_eq!(recorded_process(&recycled), RecordedProcess::Reused);

    // Matching the basename would confuse distinct node installations. The
    // alternative exists and contains the same executable bytes, but its path
    // does not own this PID and must never be signalled.
    let dir = tempfile::tempdir().unwrap();
    let alternative = dir.path().join(Path::new(binary).file_name().unwrap());
    std::fs::copy(binary, &alternative).unwrap();
    let other_installation = node_recorded_as(alternative.to_str().unwrap(), Some(pid));
    assert_eq!(
        recorded_process(&other_installation),
        RecordedProcess::Reused
    );
    assert_eq!(
        stop_by_pid(
            &other_installation,
            dir.path().join("stop.log"),
            std::time::Duration::ZERO
        )
        .unwrap(),
        PidStop::PidReused
    );
    assert!(witness.try_wait().unwrap().is_none());

    let mut supervisor = crate::supervisor::ProcessSupervisor::default();
    assert!(crate::node_lifecycle::quiesce_before_restart(
        &mut supervisor,
        &other_installation,
        dir.path().join("stop.log")
    )
    .is_err());
    assert!(witness.try_wait().unwrap().is_none());

    let _ = witness.kill();
    let _ = witness.wait();
}
