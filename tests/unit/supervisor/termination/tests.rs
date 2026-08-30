//! Identity matching for pid-based stops. Getting this wrong means signalling a
//! process that has nothing to do with the node, so the refusals matter as much
//! as the matches.

use std::path::{Path, PathBuf};

use crate::{
    supervisor::{recorded_process, termination::name_matches_binary, RecordedProcess},
    types::NodeConfig,
};

fn matches(reported: &str, binary: &str) -> bool {
    name_matches_binary(reported, Path::new(binary))
}

#[test]
fn an_exact_name_matches() {
    assert!(matches("neo-go", "/usr/local/bin/neo-go"));
    assert!(matches("neo-go", "/opt/neo/neo-go"));
}

#[test]
fn the_windows_extension_is_tolerated_on_both_sides() {
    // Forward slashes are separators on every platform this suite runs on, so
    // the `.exe` tolerance is genuinely exercised on Linux and Windows alike.
    assert!(matches("neo-go.exe", "/opt/neo/neo-go.exe"));
    assert!(matches("neo-go", "/opt/neo/neo-go.exe"));
    assert!(matches("neo-go.exe", "/opt/neo/neo-go"));
}

#[test]
#[cfg(windows)]
fn a_backslash_recorded_path_resolves_on_windows() {
    // Windows accepts both separators; POSIX platforms do not treat a backslash
    // as one, so this case only means something there.
    assert!(matches("neo-go", r"C:\neo\neo-go.exe"));
    assert!(matches("neo-go.exe", r"C:\neo\neo-go"));
}

#[test]
fn comparison_ignores_case() {
    assert!(matches("Neo-Go", "/opt/neo/neo-go"));
    assert!(matches("neo-go", "/opt/neo/NEO-GO"));
}

#[test]
fn a_different_program_never_matches() {
    // The recycled-pid case this check exists for.
    assert!(!matches("postgres", "/opt/neo/neo-go"));
    assert!(!matches("sleep", "ping.exe"));
    // A prefix is not an identity.
    assert!(!matches("neo-gosh", "/opt/neo/neo-go"));
}

#[test]
fn an_unusable_recorded_path_is_refused() {
    assert!(!matches("anything", ""));
    assert!(!matches("anything", "/"));
}

#[test]
fn surrounding_whitespace_in_the_reported_name_is_ignored() {
    assert!(matches("  neo-go  ", "/opt/neo/neo-go"));
}

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

    let _ = witness.kill();
    let _ = witness.wait();
}
