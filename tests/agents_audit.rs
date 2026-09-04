use neo_nexus::{
    agents::{self, AgentKind, AgentProfile, AgentStatus},
    backup::{WorkspaceBackupExporter, WorkspaceBackupImporter},
    repository::Repository,
    supervision::EngineState,
    supervisor::ProcessSupervisor,
    types::{Network, NewNode, NodeType, StorageEngine},
};
use std::sync::{Arc, Mutex};

fn state(path: &std::path::Path) -> EngineState {
    EngineState {
        repository: Repository::open(path.join("test.db")).unwrap(),
        data_dir: path.into(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
    }
}

fn node(state: &EngineState) -> String {
    state
        .repository
        .create_node(NewNode {
            name: "associated node".into(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: std::env::current_exe().unwrap(),
            args: vec![],
            runtime_version: "1".into(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 12332,
            p2p_port: 12333,
            ws_port: None,
        })
        .unwrap()
        .id
}

fn profile(path: &std::path::Path, node_id: String) -> AgentProfile {
    AgentProfile {
        id: "companion".into(),
        name: "Sidecar".into(),
        kind: AgentKind::Sidecar,
        node_id: Some(node_id),
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

#[test]
fn associated_node_cannot_disappear_until_agent_is_detached() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let node_id = node(&state);
    let mut profile = profile(dir.path(), node_id.clone());
    agents::save(&state, profile.clone()).unwrap();
    assert!(state
        .repository
        .delete_node(&node_id)
        .unwrap_err()
        .to_string()
        .contains("associated agent"));
    assert_eq!(state.repository.list_nodes().unwrap().len(), 1);
    profile.node_id = None;
    agents::save(&state, profile).unwrap();
    state.repository.delete_node(&node_id).unwrap();
    assert!(state.repository.list_nodes().unwrap().is_empty());
    assert!(state.repository.list_agents().unwrap()[0]
        .profile
        .node_id
        .is_none());
}

#[test]
fn corrupt_agent_record_is_not_silently_replaced_by_profile_save() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let profile = profile(dir.path(), node(&state));
    agents::save(&state, profile.clone()).unwrap();
    let connection = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    connection.execute("UPDATE managed_agents SET record = 'broken' WHERE id = ?1", [&profile.id]).unwrap();
    assert!(agents::save(&state, profile).is_err());
    let record: String = connection.query_row("SELECT record FROM managed_agents", [], |row| row.get(0)).unwrap();
    assert_eq!(record, "broken");
}

#[test]
fn managed_handle_prevents_edit_and_delete_even_without_a_recorded_pid() {
    use neo_nexus::supervisor::{ManagedProcessKind, ManagedProcessSpec};
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let profile = profile(dir.path(), node(&state));
    agents::save(&state, profile.clone()).unwrap();
    // Exercise the handle/database intermediate state using a harmless child
    // that lists this test binary's tests; its handle remains until reaped.
    let spec = ManagedProcessSpec {
        id: profile.process_id(),
        kind: ManagedProcessKind::Sidecar,
        label: "guard regression fixture".into(),
        binary_path: std::env::current_exe().unwrap(),
        args: vec!["--list".into()],
        working_dir: dir.path().into(),
        display_command: "test fixture --list".into(),
    };
    state.supervisor.lock().unwrap().start_process(&spec, dir.path().join("fixture.log")).unwrap();
    assert!(agents::save(&state, profile).unwrap_err().to_string().contains("stop the agent"));
    assert!(agents::delete(&state, "companion").unwrap_err().to_string().contains("stop the agent"));
    assert!(agents::forget_stale(&state, "companion").unwrap_err().to_string().contains("managed agent"));
    state.supervisor.lock().unwrap().stop_process(&spec.id).unwrap();
    agents::delete(&state, "companion").unwrap();
    assert!(state.repository.list_agents().unwrap().is_empty());
}

#[test]
fn stale_pid_recovery_never_releases_a_matching_live_process() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    agents::save(&state, profile(dir.path(), node(&state))).unwrap();
    let mut record = state.repository.list_agents().unwrap().remove(0);
    let pid = std::process::id();
    let system = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing().with_processes(sysinfo::ProcessRefreshKind::nothing()),
    );
    let started = system.process(sysinfo::Pid::from_u32(pid)).unwrap().start_time();
    record.pid = Some(pid);
    record.process_started_at = Some(started);
    record.status = AgentStatus::Running;
    record.desired_running = true;
    let connection = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    let write_record = |record: &neo_nexus::agents::AgentRecord| {
        connection.execute("UPDATE managed_agents SET record = ?1 WHERE id = ?2",
            rusqlite::params![serde_json::to_string(record).unwrap(), "companion"]).unwrap();
    };
    write_record(&record);
    assert!(agents::forget_stale(&state, "companion").unwrap_err().to_string().contains("still alive"));
    assert_eq!(state.repository.list_agents().unwrap()[0].pid, Some(pid));
    record.process_started_at = Some(started.saturating_sub(1));
    write_record(&record);
    agents::forget_stale(&state, "companion").unwrap();
    let recovered = state.repository.list_agents().unwrap().remove(0);
    assert_eq!(recovered.status, AgentStatus::Stopped);
    assert!(recovered.pid.is_none() && !recovered.desired_running);
    agents::delete(&state, "companion").unwrap();
}

#[test]
fn agent_backup_restores_references_stopped_and_preserves_existing_profiles() {
    let source_dir = tempfile::tempdir().unwrap();
    let source = state(source_dir.path());
    let node_id = node(&source);
    let mut profile = profile(source_dir.path(), node_id);
    let config = source_dir.path().join("agent.toml");
    std::fs::write(
        &config,
        "password = 'configuration-secret-must-not-be-exported'\n",
    )
    .unwrap();
    profile.config_path = Some(config);
    agents::save(&source, profile).unwrap();
    let mut record = source.repository.list_agents().unwrap().remove(0);
    record.pid = Some(1234567);
    record.process_started_at = Some(42);
    record.status = AgentStatus::Running;
    record.desired_running = true;
    record.restart_after = Some(123);
    record.restart_attempts = 2;
    // Simulate an active persisted record without starting an unrelated process.
    rusqlite::Connection::open(source_dir.path().join("test.db"))
        .unwrap()
        .execute(
            "UPDATE managed_agents SET record = ?1 WHERE id = ?2",
            rusqlite::params![serde_json::to_string(&record).unwrap(), "companion"],
        )
        .unwrap();
    let backup = WorkspaceBackupExporter::snapshot(&source.repository, "test", 123).unwrap();
    assert_eq!(backup.schema_version, 8);
    assert_eq!(backup.agents.len(), 1);
    let value = serde_json::to_value(&backup).unwrap();
    for field in [
        "pid",
        "process_started_at",
        "desired_running",
        "restart_after",
        "status",
    ] {
        assert!(value["agents"][0].get(field).is_none());
    }
    assert!(!value
        .to_string()
        .contains("configuration-secret-must-not-be-exported"));
    let target_dir = tempfile::tempdir().unwrap();
    let target = state(target_dir.path());
    let imported = WorkspaceBackupImporter::import(&target.repository, &backup).unwrap();
    assert_eq!(imported.agent_count, 1);
    let restored = target.repository.list_agents().unwrap().remove(0);
    assert_eq!(restored.status, AgentStatus::Stopped);
    assert!(restored.pid.is_none() && restored.process_started_at.is_none());
    assert!(!restored.desired_running);
    assert!(restored.restart_after.is_none());
    assert_eq!(restored.restart_attempts, 0);
    assert_eq!(restored.profile, record.profile);
    let mut newer = restored.profile;
    newer.version = "2".into();
    agents::save(&target, newer).unwrap();
    assert_eq!(
        WorkspaceBackupImporter::import(&target.repository, &backup)
            .unwrap()
            .agent_count,
        0
    );
    assert_eq!(
        target.repository.list_agents().unwrap()[0].profile.version,
        "2"
    );
}

#[test]
fn agent_backup_rejects_secret_arguments_and_orphaned_node_references() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    agents::save(&state, profile(dir.path(), node(&state))).unwrap();
    let mut backup = WorkspaceBackupExporter::snapshot(&state.repository, "test", 123).unwrap();
    backup.agents[0].args = vec!["--password".into(), "embedded-secret".into()];
    assert!(WorkspaceBackupImporter::validate(&backup).is_err());
    backup.agents[0].args.clear();
    backup.agents[0].node_id = Some("missing-node".into());
    assert!(WorkspaceBackupImporter::validate(&backup).is_err());
    backup.agents[0].node_id = None;
    let mut old = serde_json::to_value(&backup).unwrap();
    old["schema_version"] = serde_json::json!(7);
    old.as_object_mut().unwrap().remove("agents");
    let old = serde_json::from_value(old).unwrap();
    assert!(WorkspaceBackupImporter::validate(&old).is_ok());
}
