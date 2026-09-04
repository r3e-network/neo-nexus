use super::*;
use crate::{
    repository::Repository,
    signer_client::{BearerCredential, SignerClientConfig, SignerCredential, SignerEndpoint},
    supervisor::ProcessSupervisor,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

fn engine(dir: &std::path::Path) -> EngineState {
    EngineState {
        repository: Repository::open(dir.join("monitor.db")).unwrap(),
        data_dir: dir.into(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
    }
}

#[test]
fn background_probe_reports_transitions_without_browser_requests_and_is_rate_limited() {
    let dir = tempfile::tempdir().unwrap();
    let state = engine(dir.path());
    let health = Arc::new(AtomicUsize::new(0));
    let calls = Arc::new(AtomicUsize::new(0));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let router = Router::new()
        .route(
            "/health",
            get(
                |State((health, calls)): State<(Arc<AtomicUsize>, Arc<AtomicUsize>)>,
                 headers: HeaderMap| async move {
                    assert!(!headers.contains_key("authorization"));
                    calls.fetch_add(1, Ordering::SeqCst);
                    match health.load(Ordering::SeqCst) {
                        0 => (StatusCode::OK, Json(serde_json::json!({"status":"ok"}))),
                        1 => (StatusCode::OK, Json(serde_json::json!({"status":"sealed"}))),
                        _ => (
                            StatusCode::SERVICE_UNAVAILABLE,
                            Json(serde_json::json!({"status":"down"})),
                        ),
                    }
                },
            ),
        )
        .with_state((health.clone(), calls.clone()));
    let address = runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        address
    });
    let client = SignerClient::new(SignerClientConfig {
        endpoint: SignerEndpoint::parse(&format!("http://{address}")).unwrap(),
        credential: SignerCredential::Bearer(BearerCredential::new("test-health-token").unwrap()),
    });
    let mut monitor = SignerMonitor::with_client(client);
    let initial = Instant::now();
    monitor.tick_at(&state, initial);
    monitor.tick_at(&state, initial + Duration::from_secs(29));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    monitor.tick_at(&state, initial + Duration::from_secs(30));
    assert_eq!(state.repository.list_recent_events(100).unwrap().len(), 1);
    health.store(1, Ordering::SeqCst);
    monitor.tick_at(&state, initial + Duration::from_secs(60));
    health.store(2, Ordering::SeqCst);
    monitor.tick_at(&state, initial + Duration::from_secs(90));
    monitor.tick_at(&state, initial + Duration::from_secs(120));
    health.store(0, Ordering::SeqCst);
    monitor.tick_at(&state, initial + Duration::from_secs(150));
    let events = state.repository.list_recent_events(100).unwrap();
    assert_eq!(events.len(), 4);
    assert!(events
        .iter()
        .all(|event| event.kind == EventKind::SignerHealthChanged));
    assert_eq!(
        events
            .iter()
            .filter(|event| event.severity == EventSeverity::Warning)
            .count(),
        2
    );
    assert!(events
        .iter()
        .any(|event| event.message.contains("background probe")));
}

#[test]
fn unconfigured_monitor_is_quiet_and_invalid_configuration_is_reported_once() {
    let dir = tempfile::tempdir().unwrap();
    let state = engine(dir.path());
    let mut monitor = SignerMonitor {
        client: Ok(None),
        next_probe: None,
        last: None,
    };
    monitor.tick(&state);
    assert!(state.repository.list_recent_events(10).unwrap().is_empty());
    monitor.client = Err("missing workload profile field".into());
    let initial = Instant::now();
    monitor.tick_at(&state, initial);
    monitor.tick_at(&state, initial + Duration::from_secs(31));
    assert_eq!(state.repository.list_recent_events(10).unwrap().len(), 1);
}

#[test]
fn health_probe_honors_the_explicit_short_deadline() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (_socket, _) = listener.accept().unwrap();
        std::thread::sleep(Duration::from_millis(250));
    });
    let client = SignerClient::new(SignerClientConfig {
        endpoint: SignerEndpoint::parse(&endpoint).unwrap(),
        credential: SignerCredential::Bearer(BearerCredential::new("test-health-token").unwrap()),
    });
    let started = Instant::now();
    assert!(client
        .health_with_timeout(Duration::from_millis(30))
        .is_err());
    assert!(started.elapsed() < Duration::from_millis(200));
    server.join().unwrap();
}
