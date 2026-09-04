use super::*;
use crate::watchdog::RestartPolicy;

mod recovery;
mod reliability;
mod startup;

fn fixture() -> (tempfile::TempDir, EngineState) {
    let dir = tempfile::tempdir().unwrap();
    let state = EngineState {
        repository: Repository::open(dir.path().join("test.db")).unwrap(),
        data_dir: dir.path().to_path_buf(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
    };
    (dir, state)
}

fn event(state: &EngineState, timestamp: u64) {
    state
        .repository
        .record_event_at(
            NewRuntimeEvent {
                node_id: None,
                node_name: None,
                kind: EventKind::NodeExited,
                severity: EventSeverity::Critical,
                message: "test event".into(),
            },
            timestamp,
        )
        .unwrap();
}

#[test]
fn drains_bursts_in_insertion_order_and_resumes_after_restart() {
    let (_dir, state) = fixture();
    let mut engine = LoopState::bootstrap(&state);
    for n in 0..60 {
        event(&state, 100 - n);
    }
    engine.route_alerts(&state);
    assert_eq!(
        engine.last_routed_event, 8,
        "must not skip beyond the oldest batch"
    );
    let mut resumed = LoopState::bootstrap(&state);
    assert_eq!(resumed.last_routed_event, 8);
    for _ in 0..7 {
        resumed.route_alerts(&state);
    }
    assert_eq!(resumed.last_routed_event, 60);
    assert!(state
        .repository
        .list_workspace_settings_for_backup()
        .unwrap()
        .iter()
        .all(|setting| setting.key != "alert_routing.progress"));
}

#[test]
fn retry_limit_survives_restart_and_does_not_lose_the_failed_event_in_a_burst() {
    let (_dir, state) = fixture();
    // Keep a port bound but unserved to force a bounded local timeout.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    state
        .repository
        .save_alert_routing_policy(crate::alerts::AlertRoutingPolicy {
            enabled: true,
            provider: crate::alerts::AlertProvider::Generic,
            min_severity: EventSeverity::Critical,
            webhook_url: Some(format!("http://{}", listener.local_addr().unwrap())),
            timeout_seconds: 1,
        })
        .unwrap();
    let mut engine = LoopState::bootstrap(&state);
    event(&state, 100);
    engine.route_alerts(&state);
    assert_eq!(engine.alert_failures.get(&1), Some(&1));
    for n in 0..30 {
        event(&state, 101 + n);
    }
    let mut resumed = LoopState::bootstrap(&state);
    resumed.route_alerts(&state);
    assert_eq!(resumed.alert_failures.get(&1), Some(&2));
    resumed.route_alerts(&state);
    assert!(!resumed.alert_failures.contains_key(&1));
    assert!(resumed.last_routed_event >= 1);
    let deliveries = state.repository.list_alert_deliveries(50).unwrap();
    assert_eq!(deliveries.iter().filter(|d| d.event_id == 1).count(), 3);
}
