//! Managed-agent lifecycle shares the durable operation ledger with nodes, and
//! restarts accrue attempts across the same window as Hermes.

use neo_nexus::{
    agents::{self, AgentProfile, AgentStatus},
    repository::Repository,
    supervision::EngineState,
    supervisor::ProcessSupervisor,
};
use std::sync::{Arc, Mutex};

fn state(path: &std::path::Path) -> EngineState {
    EngineState {
        repository: Repository::open(path.join("test.db")).unwrap(),
        data_dir: path.into(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        heartbeat: neo_nexus::supervision_heartbeat::SupervisionHeartbeat::new(),
        notifications: neo_nexus::supervision_heartbeat::SupervisionHeartbeat::new(),
    }
}

fn sidecar(path: &std::path::Path) -> AgentProfile {
    AgentProfile {
        id: "ledger-agent".into(),
        name: "Ledger companion".into(),
        kind: neo_nexus::agents::AgentKind::Sidecar,
        node_id: None,
        version: "1".into(),
        binary_path: std::env::current_exe().unwrap(),
        working_dir: path.into(),
        args: vec![],
        config_path: None,
        health_url: None,
        auto_restart: true,
        binary_sha256: String::new(),
        config_sha256: None,
    }
}

/// A successfully started agent commits a controller operation in phase
/// 'committed' rather than leaving a dangling reserved row.
#[test]
fn agent_start_records_a_committed_controller_operation() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let profile = sidecar(dir.path());
    agents::save(&state, profile).unwrap();
    let id = "ledger-agent".to_string();
    agents::start(&state, &id).unwrap();
    let operations = state
        .repository
        .pending_controller_operations()
        .expect("active operations");
    // The active-operation view excludes committed rows; any leftover would
    // mean the agent start left a reserved phase that will never commit.
    assert!(operations.is_empty(), "no dangling controller operation");
    let record = state
        .repository
        .list_agents()
        .expect("agents")
        .into_iter()
        .find(|agent| agent.profile.id == id)
        .expect("agent");
    assert_eq!(record.status, AgentStatus::Running);
    assert!(record.pid.is_some());
}

/// Restart is stop followed by start and must not leave a stale active lease
/// from the stopped half blocking the started half.
#[test]
fn agent_restart_does_not_leave_a_dangling_lease() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    agents::save(&state, sidecar(dir.path())).unwrap();
    let id = "ledger-agent".to_string();
    agents::start(&state, &id).unwrap();
    agents::stop(&state, &id).unwrap();
    agents::start(&state, &id).unwrap();
    let operations = state
        .repository
        .pending_controller_operations()
        .expect("active operations");
    assert!(operations.is_empty(), "restart must not leave stale leases");
}
