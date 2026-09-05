use super::*;

fn fixture() -> (
    tempfile::TempDir,
    crate::supervision::EngineState,
    AgentProfile,
) {
    let dir = tempfile::tempdir().unwrap();
    let state = crate::supervision::EngineState {
        repository: crate::repository::Repository::open(dir.path().join("test.db")).unwrap(),
        data_dir: dir.path().into(),
        supervisor: std::sync::Arc::new(std::sync::Mutex::new(
            crate::supervisor::ProcessSupervisor::with_stop_grace_period(
                std::time::Duration::from_millis(50),
            ),
        )),
        heartbeat: crate::supervision_heartbeat::SupervisionHeartbeat::new(),
        notifications: crate::supervision_heartbeat::SupervisionHeartbeat::new(),
    };
    let profile = AgentProfile {
        id: "worker".into(),
        name: "Node companion".into(),
        kind: AgentKind::Sidecar,
        node_id: None,
        version: "1.0.0".into(),
        binary_path: std::env::current_exe().unwrap(),
        working_dir: dir.path().into(),
        args: vec![
            "--exact".into(),
            "agents::tests::child_process".into(),
            "--ignored".into(),
        ],
        config_path: None,
        health_url: None,
        auto_restart: false,
        binary_sha256: String::new(),
        config_sha256: None,
    };
    (dir, state, profile)
}

#[test]
#[ignore = "fixture launched only as a supervised child"]
fn child_process() {
    std::thread::sleep(std::time::Duration::from_secs(60));
}

#[test]
fn starts_stops_rejects_duplicate_start_and_records_lifecycle() {
    let (_dir, state, profile) = fixture();
    save(&state, profile.clone()).unwrap();
    start(&state, "worker").unwrap();
    let running = state.repository.list_agents().unwrap().remove(0);
    assert_eq!(running.status, AgentStatus::Running);
    assert!(running.process_started_at.is_some());
    assert!(start(&state, "worker").is_err());
    assert!(save(&state, profile).is_err());
    assert!(delete(&state, "worker").is_err());
    stop(&state, "worker").unwrap();
    let stopped = state.repository.list_agents().unwrap().remove(0);
    assert!(stopped.pid.is_none());
    assert!(!stopped.desired_running);
    assert!(!crate::supervisor::process_is_live(running.pid.unwrap()));
    delete(&state, "worker").unwrap();
    let events = state.repository.list_recent_events(20).unwrap();
    assert!(events
        .iter()
        .any(|event| event.kind == crate::events::EventKind::AgentStarted));
    assert!(events
        .iter()
        .any(|event| event.kind == crate::events::EventKind::AgentStopped));
}

#[test]
fn configuration_drift_requires_review_without_overwriting_local_file() {
    let (dir, state, mut profile) = fixture();
    let config = dir.path().join("worker.toml");
    std::fs::write(&config, "rpc = 1234\n").unwrap();
    profile.config_path = Some(config.clone());
    save(&state, profile).unwrap();
    std::fs::write(&config, "rpc = 9876\n").unwrap();
    assert!(start(&state, "worker")
        .unwrap_err()
        .to_string()
        .contains("configuration changed"));
    assert_eq!(std::fs::read_to_string(config).unwrap(), "rpc = 9876\n");
    assert!(state.repository.list_agents().unwrap()[0].pid.is_none());
}

#[test]
fn refuses_secret_arguments_and_unbound_references() {
    let (_dir, state, mut profile) = fixture();
    profile.args = vec!["--password".into(), "test-secret".into()];
    assert!(save(&state, profile.clone()).is_err());
    profile.args = vec!["{rpc_url}".into()];
    assert!(save(&state, profile).is_err());
    assert!(state.repository.list_agents().unwrap().is_empty());
}

#[test]
fn hermes_pins_home_and_source_without_global_environment_changes() {
    let (dir, state, mut profile) = fixture();
    profile.kind = AgentKind::Hermes;
    profile.binary_path = dir.path().join(if cfg!(windows) {
        "python.exe"
    } else {
        "python3"
    });
    std::fs::write(&profile.binary_path, "test interpreter bytes").unwrap();
    std::fs::create_dir(dir.path().join("hermes_cli")).unwrap();
    let source = dir.path().join("hermes_cli/main.py");
    std::fs::write(&source, "print('Hermes v1')").unwrap();
    let config = dir.path().join("config.yaml");
    std::fs::write(&config, "model: local\n").unwrap();
    profile.config_path = Some(config);
    profile.args = vec![];
    profile.validate(&state.repository).unwrap();
    assert_eq!(
        profile.spec(&state.repository).unwrap().args,
        hermes::arguments()
    );
    let before = std::env::var_os("HERMES_HOME");
    let env = hermes::environment(&profile);
    assert!(env.contains(&("HERMES_GATEWAY_EXTERNAL_SUPERVISOR".into(), "1".into())));
    assert_eq!(std::env::var_os("HERMES_HOME"), before);
    let old = profile::code_digest(&profile).unwrap();
    std::fs::write(source, "print('Hermes v2')").unwrap();
    assert_ne!(old, profile::code_digest(&profile).unwrap());
    profile.args = vec!["--profile".into(), "other".into()];
    assert!(profile.validate(&state.repository).is_err());
}

#[test]
fn stale_pid_with_different_start_time_cannot_be_stopped() {
    let (_dir, state, profile) = fixture();
    save(&state, profile).unwrap();
    let mut record = state.repository.list_agents().unwrap().remove(0);
    record.pid = Some(std::process::id());
    record.process_started_at = Some(1);
    record.status = AgentStatus::Running;
    state.repository.put_agent(&record).unwrap();
    assert!(stop(&state, "worker").is_err());
    assert_eq!(
        state.repository.list_agents().unwrap()[0].pid,
        Some(std::process::id())
    );
}

#[test]
fn restart_budget_is_bounded_and_persistent() {
    let dir = tempfile::tempdir().unwrap();
    let repository = crate::repository::Repository::open(dir.path().join("test.db")).unwrap();
    let mut record = AgentRecord {
        profile: AgentProfile {
            id: "test".into(),
            name: "Hermes".into(),
            kind: AgentKind::Hermes,
            node_id: None,
            version: "1".into(),
            binary_path: std::env::current_exe().unwrap(),
            working_dir: dir.path().into(),
            args: vec![],
            config_path: None,
            health_url: None,
            auto_restart: true,
            binary_sha256: String::new(),
            config_sha256: None,
        },
        status: AgentStatus::Crashed,
        pid: None,
        process_started_at: None,
        desired_running: true,
        restart_attempts: 0,
        restart_after: None,
        healthy: None,
        last_health_at: 0,
    };
    for attempt in 1..=3 {
        lifecycle::schedule(&mut record);
        assert_eq!(record.restart_attempts, attempt);
        assert!(record.restart_after.is_some());
        repository.put_agent(&record).unwrap();
        record = repository.list_agents().unwrap().remove(0);
    }
    lifecycle::schedule(&mut record);
    assert!(!record.desired_running);
    assert!(record.restart_after.is_none());
}

#[test]
fn launch_intent_is_durable_and_rejects_stale_or_duplicate_controllers() {
    let (_dir, state, profile) = fixture();
    save(&state, profile).unwrap();
    let original = state.repository.list_agents().unwrap().remove(0);
    let mut edited = original.clone();
    edited.profile.name = "Reviewed companion".into();
    state.repository.put_agent(&edited).unwrap();
    assert!(state.repository.claim_agent_start(&original, true).is_err());
    let claimed = state.repository.claim_agent_start(&edited, true).unwrap();
    assert_eq!(claimed.status, AgentStatus::Starting);
    assert!(claimed.desired_running);
    let reopened = crate::repository::Repository::open(state.repository.db_path()).unwrap();
    assert_eq!(reopened.list_agents().unwrap()[0], claimed);
    assert!(reopened.claim_agent_start(&edited, true).is_err());
    assert!(reopened.claim_agent_start(&claimed, true).is_err());
    assert!(start(&state, "worker").is_err());
    stop(&state, "worker").unwrap();
    assert!(!reopened.list_agents().unwrap()[0].desired_running);
}

#[test]
fn failed_launch_intent_write_cannot_spawn_or_overwrite_the_profile() {
    let (_dir, state, profile) = fixture();
    save(&state, profile).unwrap();
    let original = state.repository.list_agents().unwrap().remove(0);
    let database = rusqlite::Connection::open(state.repository.db_path()).unwrap();
    database
        .execute_batch(
            "CREATE TRIGGER reject_start_intent BEFORE UPDATE ON managed_agents
        WHEN json_extract(NEW.record, '$.status') = 'Starting'
        BEGIN SELECT RAISE(ABORT, 'test intent write failure'); END;",
        )
        .unwrap();
    assert!(start(&state, "worker")
        .unwrap_err()
        .to_string()
        .contains("test intent write failure"));
    assert_eq!(state.repository.list_agents().unwrap()[0], original);
    assert!(!state.supervisor.lock().unwrap().is_managing("agent:worker"));
}
