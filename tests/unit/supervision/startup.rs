use super::{
    reliability::{immediate_watchdog, node},
    *,
};
use std::io::{Read, Write};

fn webhook(expected: usize) -> (String, thread::JoinHandle<usize>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let worker = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut received = 0;
        while received < expected && Instant::now() < deadline {
            let accepted = listener.accept();
            if accepted
                .as_ref()
                .is_err_and(|error| error.kind() == std::io::ErrorKind::WouldBlock)
            {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            let (mut stream, _) = accepted.expect("webhook accept failed");
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut request = Vec::new();
            loop {
                let mut bytes = [0; 4096];
                let count = stream.read(&mut bytes).unwrap();
                if count == 0 {
                    break;
                }
                request.extend_from_slice(&bytes[..count]);
                if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&request[..end]);
                    let length = header
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
                assert!(request.len() < 64 * 1024);
            }
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .unwrap();
            received += 1;
        }
        received
    });
    (url, worker)
}

fn enable_alerts(state: &EngineState, url: String) {
    state
        .repository
        .save_alert_routing_policy(crate::alerts::AlertRoutingPolicy {
            enabled: true,
            provider: crate::alerts::AlertProvider::Generic,
            min_severity: EventSeverity::Critical,
            webhook_url: Some(url),
            timeout_seconds: 1,
        })
        .unwrap();
}

#[test]
fn corrupt_progress_replays_actual_backlog_deliveries_and_records_a_diagnostic() {
    let (_dir, state) = fixture();
    for stamp in 1..=3 {
        event(&state, stamp);
    }
    state
        .repository
        .save_workspace_section("alert_routing.progress", "broken-json")
        .unwrap();
    let (url, received) = webhook(4);
    enable_alerts(&state, url);
    let mut loop_state = LoopState::bootstrap(&state);
    assert_eq!(loop_state.last_routed_event, 0);
    loop_state.route_alerts(&state);
    assert_eq!(received.join().unwrap(), 4);
    let deliveries = state.repository.list_alert_deliveries(10).unwrap();
    for id in 1..=4 {
        assert!(deliveries
            .iter()
            .any(|delivery| delivery.event_id == id
                && delivery.status == AlertDeliveryStatus::Delivered));
    }
    let events = state.repository.list_events_after(3, 10).unwrap();
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::RuntimeRecovered
            && event.severity == EventSeverity::Critical
            && event.message.contains("oldest retained event")));
    assert_eq!(LoopState::bootstrap(&state).last_routed_event, 4);
}

#[test]
fn invalid_cursor_ranges_cannot_skip_future_events() {
    for cursor in [-1, 50_000] {
        let (_dir, state) = fixture();
        event(&state, 1);
        state
            .repository
            .save_alert_progress(cursor, &BTreeMap::new())
            .unwrap();
        assert_eq!(LoopState::bootstrap(&state).last_routed_event, 0);
    }
}

#[test]
fn startup_missing_processes_retry_once_per_failure_and_reused_pids_stay_isolated() {
    let (_dir, state) = fixture();
    let missing = node(
        &state,
        "missing-at-startup",
        NodeStatus::Running,
        Some(4_000_000),
    );
    let reused = node(
        &state,
        "reused-at-startup",
        NodeStatus::Starting,
        Some(std::process::id()),
    );
    let mut loop_state = LoopState::bootstrap(&state);
    loop_state.watchdog = immediate_watchdog();
    loop_state.reconcile_startup(&state);
    let current = state.nodes();
    assert_eq!(
        current.iter().find(|n| n.id == missing.id).unwrap().status,
        NodeStatus::Crashed
    );
    let foreign = current.iter().find(|n| n.id == reused.id).unwrap();
    assert_eq!(foreign.status, NodeStatus::Error);
    assert_eq!(foreign.pid, Some(std::process::id()));
    assert!(matches!(
        loop_state.watchdog.status(&missing.id, Instant::now()),
        crate::watchdog::WatchdogStatus::Pending { attempt: 1, .. }
    ));
    let last = state.repository.latest_event_id().unwrap();
    loop_state.reconcile_startup(&state);
    loop_state.watch_external_processes(&state);
    assert_eq!(
        state.repository.latest_event_id().unwrap(),
        last,
        "startup and the first tick must not diagnose the same loss twice"
    );
    for _ in 0..2 {
        loop_state.run_due_restarts(&state);
    }
    assert!(matches!(
        loop_state.watchdog.status(&missing.id, Instant::now()),
        crate::watchdog::WatchdogStatus::Exhausted { attempts: 2 }
    ));
    assert!(!loop_state.watchdog.has_pending_restart());
    assert!(crate::supervisor::process_is_live(std::process::id()));
    assert_eq!(
        state.nodes().iter().find(|n| n.id == reused.id).unwrap(),
        foreign
    );
}

#[test]
fn startup_preserves_a_live_orphan_and_does_not_revive_settled_states() {
    let (_dir, state) = fixture();
    for status in [NodeStatus::Stopped, NodeStatus::Crashed, NodeStatus::Error] {
        node(&state, status.label(), status, None);
    }
    let mut live = node(
        &state,
        "live-orphan",
        NodeStatus::Running,
        Some(std::process::id()),
    );
    live.binary_path = std::env::current_exe().unwrap();
    // Only the executable identity changes; this creates a real living orphan
    // record without allowing the test runner to become a managed child.
    let connection = rusqlite::Connection::open(state.data_dir.join("test.db")).unwrap();
    connection
        .execute(
            "UPDATE nodes SET binary_path = ?1 WHERE id = ?2",
            rusqlite::params![live.binary_path.to_string_lossy(), live.id],
        )
        .unwrap();
    drop(connection);
    let before = state.nodes();
    let mut loop_state = LoopState::bootstrap(&state);
    loop_state.reconcile_startup(&state);
    assert_eq!(state.nodes(), before);
    assert!(!loop_state.watchdog.has_pending_restart());
    assert!(crate::supervisor::process_is_live(std::process::id()));
}

#[test]
fn engine_initial_start_routes_recovery_alert_and_respects_disabled_restarts() {
    let (_dir, state) = fixture();
    node(
        &state,
        "missing-with-restarts-disabled",
        NodeStatus::Running,
        Some(4_000_000),
    );
    let mut policy = default_restart_policy();
    policy.enabled = false;
    state.repository.save_watchdog_policy(policy).unwrap();
    let (url, received) = webhook(1);
    enable_alerts(&state, url);
    let engine = Engine::start(state.clone());
    assert_eq!(
        state.nodes()[0].status,
        NodeStatus::Crashed,
        "recovery must finish before start returns"
    );
    assert_eq!(
        received.join().unwrap(),
        1,
        "first-time cursor initialization must not discard recovery alerts"
    );
    drop(engine);
    assert_eq!(state.nodes()[0].status, NodeStatus::Crashed);
    assert!(state.nodes()[0].pid.is_none());
}
