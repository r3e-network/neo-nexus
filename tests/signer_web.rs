//! End-to-end workbench coverage for the external signer client.

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use neo_nexus::{
    events::{EventKind, EventSeverity},
    repository::Repository,
    signer_client::{
        BearerCredential, SignerClient, SignerClientConfig, SignerCredential, SignerEndpoint,
    },
    web::{auth::AuthStore, router::build_router, WebState},
};
use serde_json::{json, Value};
use ureq::AgentBuilder;

const WEB_TOKEN: &str = "web-test-token";
const SIGNER_TOKEN: &str = "signer-admin-token";

#[derive(Clone, Default)]
struct MockSigner {
    generated: Arc<AtomicUsize>,
    saved_policy: Arc<Mutex<Option<Value>>>,
    last_generate: Arc<Mutex<Option<Value>>>,
    /// When false, `/health` fails, so the workbench sees an unreachable signer.
    health_ok: Arc<std::sync::atomic::AtomicBool>,
}

struct Rig {
    base_url: String,
    signer: MockSigner,
    repository: Repository,
    _runtime: tokio::runtime::Runtime,
    _home: tempfile::TempDir,
}

fn spawn() -> Rig {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("test runtime");
    let home = tempfile::tempdir().expect("temporary workspace");
    let signer = MockSigner::default();
    signer.health_ok.store(true, Ordering::SeqCst);
    let signer_router = Router::new()
        .route("/health", get(signer_health))
        .route("/signer/api/v1/keys", get(signer_keys).post(generate_key))
        .route(
            "/signer/api/v1/keys/{id}/policy",
            get(key_policy).post(save_policy),
        )
        .route(
            "/signer/api/v1/callers",
            get(signer_callers).post(create_caller),
        )
        .route("/signer/api/v1/audit", get(signer_audit))
        .with_state(signer.clone());
    let signer_address = runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("signer listener");
        let address = listener.local_addr().expect("signer address");
        tokio::spawn(async move {
            axum::serve(listener, signer_router)
                .await
                .expect("mock signer server");
        });
        address
    });

    let db_path = home.path().join("neonexus.db");
    let repository = Repository::open(&db_path).expect("workspace repository");
    let repository_for_tests = repository.clone();
    let signer_client = SignerClient::new(SignerClientConfig {
        endpoint: SignerEndpoint::parse(&format!("http://{signer_address}"))
            .expect("loopback signer endpoint"),
        credential: SignerCredential::Bearer(
            BearerCredential::new(SIGNER_TOKEN).expect("signer bearer"),
        ),
    });
    let state = WebState::new(
        repository,
        home.path().to_path_buf(),
        AuthStore::from_token(WEB_TOKEN),
    )
    .with_signer_client(signer_client);
    let web_address = runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("workbench listener");
        let address = listener.local_addr().expect("workbench address");
        tokio::spawn(async move {
            axum::serve(listener, build_router(state))
                .await
                .expect("workbench server");
        });
        address
    });
    Rig {
        base_url: format!("http://{web_address}"),
        signer,
        repository: repository_for_tests,
        _runtime: runtime,
        _home: home,
    }
}

fn agent() -> ureq::Agent {
    AgentBuilder::new()
        .redirects(0)
        .timeout(Duration::from_secs(10))
        .build()
}

fn response(result: Result<ureq::Response, ureq::Error>) -> ureq::Response {
    let (response, error) = match result {
        Ok(response) => (Some(response), None),
        Err(ureq::Error::Status(_, response)) => (Some(response), None),
        Err(error) => (None, Some(error)),
    };
    assert!(response.is_some(), "request failed: {error:?}");
    response.expect("request should reach the web server")
}

fn login(http: &ureq::Agent, base_url: &str) -> String {
    let response = response(
        http.post(&format!("{base_url}/login"))
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string(&format!("token={WEB_TOKEN}")),
    );
    response
        .header("set-cookie")
        .and_then(|header| header.split(';').next())
        .expect("session cookie")
        .to_string()
}

#[test]
fn signer_page_is_session_bound_and_contains_no_secret_or_signing_form() {
    let rig = spawn();
    let http = agent();
    let anonymous = response(http.get(&format!("{}/signer", rig.base_url)).call());
    assert_eq!(anonymous.status(), 303);
    assert_eq!(anonymous.header("location"), Some("/login"));

    let session = login(&http, &rig.base_url);
    let page = response(
        http.get(&format!("{}/signer", rig.base_url))
            .set("cookie", &session)
            .call(),
    );
    assert_eq!(page.status(), 200);
    let text = page.into_string().expect("signer page");
    assert!(text.contains("NeoOS signer"));
    assert!(text.contains("NeoNexus is a client only"));
    assert!(text.contains("validator"));
    for forbidden in [
        "name=\"private_key\"",
        "name=\"passphrase\"",
        "name=\"unsigned_hex\"",
        "name=\"data_hex\"",
        "action=\"/signer/api/v1/sign",
    ] {
        assert!(!text.contains(forbidden), "{forbidden} reached the browser");
    }
}

#[test]
fn key_generation_and_policy_edits_cross_the_client_boundary() {
    let rig = spawn();
    let http = agent();
    let session = login(&http, &rig.base_url);
    let generated = response(
        http.post(&format!("{}/signer/keys", rig.base_url))
            .set("cookie", &session)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string("label=validator-2&network=testnet&network_magic="),
    );
    assert_eq!(generated.status(), 303);
    assert_eq!(rig.signer.generated.load(Ordering::SeqCst), 1);

    let detail = response(
        http.get(&format!("{}/signer/keys/key-1", rig.base_url))
            .set("cookie", &session)
            .call(),
    );
    assert_eq!(detail.status(), 200);
    let text = detail.into_string().expect("policy page");
    for field in [
        "allow_consensus",
        "allow_transfer",
        "contract_method_whitelist",
        "asset_whitelist",
        "asset_limits",
        "transfer_to_blacklist",
        "max_single_amount",
        "window_max_amount",
        "max_signers",
        "max_system_fee",
        "max_network_fee",
        "signature_window_seconds",
        "signature_window_count",
    ] {
        assert!(text.contains(&format!("name=\"{field}\"")), "{field}");
    }

    let saved = response(
        http.post(&format!("{}/signer/keys/key-1/policy", rig.base_url))
            .set("cookie", &session)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string(
                "allow_transfer=true&asset_whitelist=0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5\
                 &asset_limits=0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5%7C75%7C300%7C250\
                 &max_single_amount=100&window_seconds=60&window_max_amount=500\
                 &max_signers=1&max_system_fee=100000000&max_network_fee=10000000\
                 &signature_window_seconds=60&signature_window_count=25",
            ),
    );
    assert_eq!(saved.status(), 303);
    let policy = rig
        .signer
        .saved_policy
        .lock()
        .expect("saved policy lock")
        .clone()
        .expect("policy reached signer");
    assert_eq!(policy["allow_transfer"], true);
    assert_eq!(policy["max_single_amount"], "100");
    assert_eq!(policy["window_limit"]["seconds"], 60);
    assert_eq!(policy["asset_limits"][0]["max_single_amount"], "75");
    assert_eq!(policy["asset_limits"][0]["window_limit"]["seconds"], 300);
    assert_eq!(policy["max_signers"], 1);
    assert_eq!(policy["max_signatures"]["seconds"], 60);
    assert_eq!(policy["max_signatures"]["count"], 25);
}

#[test]
fn bearer_caller_token_is_returned_once_with_no_store() {
    let rig = spawn();
    let http = agent();
    let session = login(&http, &rig.base_url);
    let created = response(
        http.post(&format!("{}/signer/callers", rig.base_url))
            .set("cookie", &session)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string("label=relayer&capability=sign&grant_mode=only&key_ids=key-1"),
    );
    assert_eq!(created.status(), 200);
    assert_eq!(created.header("cache-control"), Some("no-store, max-age=0"));
    assert!(created
        .into_string()
        .expect("created caller page")
        .contains("one-time-caller-token"));

    let listing = response(
        http.get(&format!("{}/signer", rig.base_url))
            .set("cookie", &session)
            .call(),
    )
    .into_string()
    .expect("signer listing");
    assert!(!listing.contains("one-time-caller-token"));
}

async fn signer_health(State(state): State<MockSigner>) -> impl IntoResponse {
    if state.health_ok.load(Ordering::SeqCst) {
        (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "degraded"})),
        )
            .into_response()
    }
}

async fn signer_keys(headers: HeaderMap) -> impl IntoResponse {
    authenticated(
        &headers,
        json!({
            "allowed": true,
            "keys": [key_json()]
        }),
    )
}

async fn generate_key(
    State(state): State<MockSigner>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    if authorized(&headers) {
        if !state.health_ok.load(Ordering::SeqCst) {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "allowed": false,
                    "code": "signer-degraded",
                    "message": "custody is degraded"
                })),
            )
                .into_response();
        }
        state.generated.fetch_add(1, Ordering::SeqCst);
        *state.last_generate.lock().expect("generate lock") = Some(body.clone());
        assert_eq!(body["label"], "validator-2");
        assert_eq!(body["network"], "testnet");
    }
    authenticated(&headers, merge_allowed(key_json())).into_response()
}

async fn key_policy(Path(_id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    authenticated(
        &headers,
        json!({
            "allowed": true,
            "key_id": "key-1",
            "label": "validator",
            "network": "testnet",
            "network_magic": 894710606u32,
            "public_key": "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "script_hash": "0x1111111111111111111111111111111111111111",
            "address": "NTest",
            "verification_script": "0c2102aa4156e7b327",
            "signing_enabled": true,
            "problems": [],
            "policy": empty_policy()
        }),
    )
}

async fn save_policy(
    State(state): State<MockSigner>,
    Path(_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    if authorized(&headers) {
        *state.saved_policy.lock().expect("policy lock") = Some(body.clone());
    }
    authenticated(
        &headers,
        json!({"allowed": true, "problems": [], "policy": body}),
    )
}

async fn signer_callers(headers: HeaderMap) -> impl IntoResponse {
    authenticated(
        &headers,
        json!({
            "allowed": true,
            "callers": [{
                "id": "caller-admin",
                "label": "workbench",
                "auth_mode": "bearer",
                "workload_public_key": null,
                "workload_subject": null,
                "key_grant": {"mode": "any", "key_ids": []},
                "capabilities": ["admin"],
                "allowed_origins": [],
                "created_at_unix": 1700000000u64,
                "disabled": false
            }]
        }),
    )
}

async fn create_caller(headers: HeaderMap, Json(body): Json<Value>) -> Response {
    if authorized(&headers) && body["label"] == "refuse-me" {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "allowed": false,
                "code": "policy-denied",
                "message": "caller label refused"
            })),
        )
            .into_response();
    }
    authenticated(
        &headers,
        json!({
            "allowed": true,
            "caller": {
                "id": "caller-relayer",
                "label": "relayer",
                "auth_mode": "bearer",
                "workload_public_key": null,
                "workload_subject": null,
                "key_grant": {"mode": "only", "key_ids": ["key-1"]},
                "capabilities": ["sign"],
                "allowed_origins": [],
                "created_at_unix": 1700000000u64,
                "disabled": false
            },
            "token": "one-time-caller-token"
        }),
    )
    .into_response()
}

async fn signer_audit(headers: HeaderMap) -> impl IntoResponse {
    authenticated(&headers, json!({"allowed": true, "entries": []}))
}

fn authenticated(headers: &HeaderMap, body: Value) -> (StatusCode, Json<Value>) {
    if authorized(headers) {
        (StatusCode::OK, Json(body))
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(json!({
                "allowed": false,
                "code": "unknown-token",
                "message": "unknown caller"
            })),
        )
    }
}

fn authorized(headers: &HeaderMap) -> bool {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        == Some(&format!("Bearer {SIGNER_TOKEN}"))
}

fn merge_allowed(mut key: Value) -> Value {
    key.as_object_mut()
        .expect("key object")
        .insert("allowed".to_string(), Value::Bool(true));
    key
}

fn key_json() -> Value {
    json!({
        "key_id": "key-1",
        "label": "validator",
        "network": "testnet",
        "network_magic": 894710606u32,
        "public_key": "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "script_hash": "0x1111111111111111111111111111111111111111",
        "address": "NTest",
        "verification_script": "0c2102aa4156e7b327",
        "signing_enabled": true
    })
}

fn empty_policy() -> Value {
    json!({
        "allow_consensus": false,
        "allow_transfer": false,
        "allow_contract_call": false,
        "allow_global_scope": false,
        "allow_raw": false,
        "contract_whitelist": [],
        "contract_blacklist": [],
        "contract_method_whitelist": [],
        "contract_method_blacklist": [],
        "asset_whitelist": [],
        "asset_blacklist": [],
        "asset_limits": [],
        "transfer_to_whitelist": [],
        "transfer_to_blacklist": [],
        "max_single_amount": null,
        "window_limit": null,
        "max_signers": null,
        "max_system_fee": null,
        "max_network_fee": null,
        "max_signatures": null
    })
}

#[test]
fn neo_x_key_generation_and_evm_policy_fields_cross_the_boundary() {
    let rig = spawn();
    let http = agent();
    let session = login(&http, &rig.base_url);

    let generated = response(
        http.post(&format!("{}/signer/keys", rig.base_url))
            .set("cookie", &session)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string("label=validator-2&network=testnet&network_magic=&chain_family=neo-x"),
    );
    assert_eq!(generated.status(), 303);
    let generate_body = rig
        .signer
        .last_generate
        .lock()
        .expect("generate lock")
        .clone()
        .expect("generate reached the signer");
    assert_eq!(generate_body["chain_family"], "neox");
    let events = rig
        .repository
        .list_recent_events(50)
        .expect("event journal");
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::SignerKeyCreated
            && event.message.contains("generated key key-1 for Neo X")));

    let saved = response(
        http.post(&format!("{}/signer/keys/key-1/policy", rig.base_url))
            .set("cookie", &session)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string(
                "chain_family=neo-x&evm_chain_id=12227332&evm_max_gas_price=50000000000                 &evm_max_gas_limit=30000000&evm_method_whitelist=0xabc%3Atransfer                 &evm_method_blacklist=0xdef%3Akill",
            ),
    );
    assert_eq!(saved.status(), 303);
    let policy = rig
        .signer
        .saved_policy
        .lock()
        .expect("policy lock")
        .clone()
        .expect("policy reached the signer");
    assert_eq!(policy["chain_family"], "neox");
    assert_eq!(policy["evm_chain_id"], 12227332);
    assert_eq!(policy["evm_max_gas_price"], "50000000000");
    assert_eq!(policy["evm_max_gas_limit"], 30000000);
    assert_eq!(policy["evm_method_whitelist"][0], "0xabc:transfer");
    assert_eq!(policy["evm_method_blacklist"][0], "0xdef:kill");
}

#[test]
fn private_neox_key_generation_requires_and_forwards_u64_chain_id() {
    let rig = spawn();
    let http = agent();
    let session = login(&http, &rig.base_url);
    for fields in [
        "chain_family=neo-x&chain_id=",
        "chain_family=neo-n3&chain_id=4294967300",
        "chain_family=neo-x&chain_id=4294967300&network_magic=123",
    ] {
        response(
            http.post(&format!("{}/signer/keys", rig.base_url))
                .set("cookie", &session)
                .set("content-type", "application/x-www-form-urlencoded")
                .send_string(&format!("label=private-validator&network=private&{fields}")),
        );
        assert!(rig.signer.last_generate.lock().unwrap().is_none());
    }
    response(
        http.post(&format!("{}/signer/keys", rig.base_url))
            .set("cookie", &session)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string(
                "label=private-validator&network=private&chain_family=neo-x&chain_id=4294967300",
            ),
    );
    let body = rig.signer.last_generate.lock().unwrap().clone().unwrap();
    assert_eq!(body["chain_family"], "neox");
    assert_eq!(body["chain_id"], 4_294_967_300u64);
    assert!(body.get("network_magic").is_none());
}

#[test]
fn refused_signer_requests_are_journaled_as_warnings() {
    let rig = spawn();
    let http = agent();
    let session = login(&http, &rig.base_url);

    // The signer answers a caller creation with a policy refusal.
    let refused = response(
        http.post(&format!("{}/signer/callers", rig.base_url))
            .set("cookie", &session)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string("label=refuse-me&capability=sign&grant_mode=any"),
    );
    assert_eq!(refused.status(), 303);

    // The signer answers a key generation with a degraded-service refusal.
    rig.signer.health_ok.store(false, Ordering::SeqCst);
    let degraded = response(
        http.post(&format!("{}/signer/keys", rig.base_url))
            .set("cookie", &session)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string("label=validator-2&network=testnet&network_magic="),
    );
    assert_eq!(degraded.status(), 303);

    let events = rig
        .repository
        .list_recent_events(50)
        .expect("event journal");
    assert!(events.iter().any(|event| {
        event.kind == EventKind::SignerRequestFailed
            && event.severity == EventSeverity::Warning
            && event.message.contains("caller creation")
            && event.message.contains("policy-denied")
    }));
    assert!(events.iter().any(|event| {
        event.kind == EventKind::SignerRequestFailed
            && event.message.contains("key generation")
            && event.message.contains("signer-degraded")
    }));
}

#[test]
fn signer_health_transitions_are_journaled_once_per_change() {
    let rig = spawn();
    let http = agent();
    let session = login(&http, &rig.base_url);
    let health_events = |rig: &Rig| {
        rig.repository
            .list_recent_events(50)
            .expect("event journal")
            .into_iter()
            .filter(|event| event.kind == EventKind::SignerHealthChanged)
            .collect::<Vec<_>>()
    };
    let view_signer = |http: &ureq::Agent, rig: &Rig, session: &str| {
        response(
            http.get(&format!("{}/signer", rig.base_url))
                .set("cookie", session)
                .call(),
        )
    };

    view_signer(&http, &rig, &session);
    let events = health_events(&rig);
    assert_eq!(
        events.len(),
        1,
        "the first observation is the baseline event"
    );
    assert_eq!(events[0].severity, EventSeverity::Info);

    rig.signer.health_ok.store(false, Ordering::SeqCst);
    view_signer(&http, &rig, &session);
    view_signer(&http, &rig, &session);
    let events = health_events(&rig);
    assert_eq!(
        events.len(),
        2,
        "repeating an observation must not re-journal it"
    );
    let outage = events
        .iter()
        .find(|event| event.severity == EventSeverity::Critical)
        .expect("the outage transition is recorded as Critical");

    rig.signer.health_ok.store(true, Ordering::SeqCst);
    view_signer(&http, &rig, &session);
    let events = health_events(&rig);
    assert_eq!(events.len(), 3, "recovery is a transition and is recorded");
    let recovery = events
        .iter()
        .find(|event| event.severity == EventSeverity::Info && event.id > outage.id)
        .expect("the recovery is recorded after the outage");
    assert!(recovery.message.contains("signer reports ok"));
}
