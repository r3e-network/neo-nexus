//! Contract tests for the client-only NeoOS signer integration.

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use ed25519_dalek::{Signature, Signer as _, SigningKey, Verifier as _, VerifyingKey};
use neo_nexus::signer_client::{
    body_sha256, workload_signing_message, workload_signing_message_v2, BearerCredential,
    GenerateKeyRequest, RawSignRequest, SignRequest, SignerClient, SignerClientConfig,
    SignerClientErrorKind, SignerCredential, SignerEndpoint, SignerOutcome, WorkloadCredential,
};
use serde_json::json;

fn client(endpoint: &str, credential: SignerCredential) -> SignerClient {
    SignerClient::new(SignerClientConfig {
        endpoint: SignerEndpoint::parse(endpoint).expect("valid test endpoint"),
        credential,
    })
}

fn bearer() -> SignerCredential {
    SignerCredential::Bearer(BearerCredential::new("test-only-token").expect("token"))
}

fn spawn(router: Router) -> (String, tokio::runtime::Runtime) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("test runtime");
    let address = runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test listener");
        let address = listener.local_addr().expect("listener address");
        tokio::spawn(async move {
            axum::serve(listener, router).await.expect("test server");
        });
        address
    });
    (format!("http://{address}"), runtime)
}

#[test]
fn endpoint_validation_keeps_cleartext_on_loopback() {
    assert!(SignerEndpoint::parse("http://127.0.0.1:8081").is_ok());
    assert!(SignerEndpoint::parse("http://[::1]:8081").is_ok());
    assert!(SignerEndpoint::parse("https://custody.example").is_ok());

    for refused in [
        "http://custody.example",
        "ftp://127.0.0.1/signer",
        "https://user:secret@custody.example",
        "https://custody.example/signer",
        "https://custody.example?target=other",
        "https://custody.example#fragment",
    ] {
        assert!(
            SignerEndpoint::parse(refused).is_err(),
            "{refused} crossed the endpoint boundary"
        );
    }
}

#[test]
fn authentication_secrets_are_redacted_from_debug_output() {
    let bearer = BearerCredential::new("token-that-must-not-print").expect("bearer");
    let workload =
        WorkloadCredential::from_seed("caller-1", Some("neo-nexus:test".to_string()), [7u8; 32])
            .expect("workload");

    let bearer_debug = format!("{bearer:?}");
    let workload_debug = format!("{workload:?}");
    assert!(!bearer_debug.contains("token-that-must-not-print"));
    assert!(!workload_debug.contains(&"07".repeat(32)));
    assert!(bearer_debug.contains("REDACTED"));
    assert!(workload_debug.contains("REDACTED"));
}

#[test]
fn unsupported_identity_providers_fail_before_any_credential_is_sent() {
    use neo_nexus::signer_client::{
        ApiKeyCallerRequest, ApiKeyCredential, KeyGrant, OidcCallerRequest, OidcCredential,
    };
    for credential in [
        SignerCredential::Oidc(OidcCredential::new("oidc.test.token").unwrap()),
        SignerCredential::ApiKey(ApiKeyCredential::new("api-id", "api-secret").unwrap()),
    ] {
        let error = client("http://127.0.0.1:1", credential)
            .list_keys()
            .unwrap_err();
        assert_eq!(error.kind(), SignerClientErrorKind::Configuration);
        assert!(error.to_string().contains("does not support"));
        assert!(!error.to_string().contains("oidc.test.token"));
        assert!(!error.to_string().contains("api-secret"));
    }
    let client = client("http://127.0.0.1:1", bearer());
    assert_eq!(
        client
            .create_api_key_caller(&ApiKeyCallerRequest {
                label: "api".to_string(),
                key_grant: KeyGrant::any(),
                capabilities: vec![],
                allowed_origins: vec![],
                expires_at: None,
            })
            .unwrap_err()
            .kind(),
        SignerClientErrorKind::Request
    );
    assert_eq!(
        client
            .create_oidc_caller(&OidcCallerRequest {
                label: "oidc".to_string(),
                key_grant: KeyGrant::any(),
                capabilities: vec![],
                allowed_origins: vec![],
                oidc_issuer: "https://issuer.example".to_string(),
                oidc_audience: "audience".to_string(),
                oidc_subject_pattern: None,
            })
            .unwrap_err()
            .kind(),
        SignerClientErrorKind::Request
    );
}

#[derive(Clone)]
struct WorkloadState {
    verifying_key: VerifyingKey,
}

async fn workload_generate(
    State(state): State<WorkloadState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let valid = verify_workload_request(&state.verifying_key, &headers, &body);
    if !valid {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "allowed": false,
                "code": "workload-signature-invalid",
                "message": "invalid assertion"
            })),
        );
    }
    (
        StatusCode::OK,
        Json(json!({
            "allowed": true,
            "key_id": "key-1",
            "label": "validator",
            "network": "testnet",
            "network_magic": 894710606u32,
            "public_key": "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "script_hash": "0x1111111111111111111111111111111111111111",
            "address": "NTest",
            "verification_script": "0c2102aa4156e7b327",
            "signing_enabled": true
        })),
    )
}

fn verify_workload_request(key: &VerifyingKey, headers: &HeaderMap, body: &[u8]) -> bool {
    let text = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
    };
    let Some(caller_id) = text("x-neoos-caller") else {
        return false;
    };
    let Some(timestamp) = text("x-neoos-timestamp").and_then(|value| value.parse::<u64>().ok())
    else {
        return false;
    };
    let Some(nonce) = text("x-neoos-nonce") else {
        return false;
    };
    let Some(signature) = text("x-neoos-signature")
        .and_then(|value| decode_hex::<64>(&value))
        .map(|bytes| Signature::from_bytes(&bytes))
    else {
        return false;
    };
    if headers.contains_key("authorization") || headers.contains_key("origin") || nonce.len() != 32
    {
        return false;
    }
    let Some(audience) = text("x-neoos-audience") else {
        return false;
    };
    if text("x-neoos-workload-protocol").as_deref() != Some("neoos-workload-v2")
        || Some(audience.as_str())
            != text("host")
                .as_ref()
                .map(|host| format!("http://{host}"))
                .as_deref()
    {
        return false;
    }
    let message = workload_signing_message_v2(
        &audience,
        &caller_id,
        Some("neo-nexus:test"),
        timestamp,
        &nonce,
        "POST",
        "/signer/api/v1/keys",
        &body_sha256(body),
    );
    key.verify(&message, &signature).is_ok()
        && body == br#"{"label":"validator","network":"testnet","network_magic":894710606}"#
}

#[test]
fn workload_authentication_binds_the_exact_request_body_and_route() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let router = Router::new()
        .route("/signer/api/v1/keys", post(workload_generate))
        .with_state(WorkloadState {
            verifying_key: signing_key.verifying_key(),
        });
    let (endpoint, _runtime) = spawn(router);
    let credential = WorkloadCredential::from_seed(
        "caller-1",
        Some("neo-nexus:test".to_string()),
        signing_key.to_bytes(),
    )
    .expect("workload credential");
    let client = client(&endpoint, SignerCredential::Workload(Box::new(credential)));

    let outcome = client
        .generate_key(&GenerateKeyRequest {
            label: "validator".to_string(),
            network: "testnet".to_string(),
            network_magic: Some(894_710_606),
            chain_family: None,
            chain_id: None,
        })
        .expect("request reaches signer");
    let key = match outcome {
        SignerOutcome::Allowed(key) => Some(key),
        SignerOutcome::Refused(_) => None,
    };
    assert!(key.is_some(), "valid workload request was refused");
    let key = key.expect("allowed workload key");
    assert_eq!(key.key_id, "key-1");
}

async fn refusal(State(calls): State<Arc<AtomicUsize>>) -> impl IntoResponse {
    calls.fetch_add(1, Ordering::SeqCst);
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "allowed": false,
            "code": "signer-not-provisioned",
            "message": "vault is sealed"
        })),
    )
}

#[test]
fn a_signing_refusal_is_data_and_is_not_retried() {
    let calls = Arc::new(AtomicUsize::new(0));
    let router = Router::new()
        .route("/signer/api/v1/sign/transaction", post(refusal))
        .with_state(Arc::clone(&calls));
    let (endpoint, _runtime) = spawn(router);
    let outcome = client(&endpoint, bearer())
        .sign_transaction(&SignRequest {
            key_id: "key-1".to_string(),
            unsigned_hex: "00".to_string(),
            request_id: None,
            chain_family: None,
            chain_id: None,
        })
        .expect("refusal is a valid signer response");
    let refusal = match outcome {
        SignerOutcome::Refused(refusal) => Some(refusal),
        SignerOutcome::Allowed(_) => None,
    };
    assert!(
        refusal.is_some(),
        "service refusal became an allowed result"
    );
    let refusal = refusal.expect("typed signer refusal");
    assert_eq!(refusal.status, 503);
    assert_eq!(refusal.code, "signer-not-provisioned");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

fn evm_request() -> SignRequest {
    SignRequest {
        key_id: "evm-key".to_string(),
        unsigned_hex: "02c0".to_string(),
        request_id: Some("evm-request-1".to_string()),
        chain_family: Some("neox".to_string()),
        chain_id: Some(4_294_967_300),
    }
}

fn evm_witness() -> serde_json::Value {
    // SignatureBody in neo-os-services/workers/neo-signer/src/api.rs.
    json!({
        "allowed": true, "key_id": "evm-key", "script_hash": "0x00",
        "address": "0x1111111111111111111111111111111111111111",
        "digest": "abcd", "invocation_script": "", "verification_script": "",
        "chain_family": "neox", "signed_transaction": "0x02c0"
    })
}

#[test]
fn neox_transaction_names_chain_and_preserves_canonical_signed_bytes() {
    let router = Router::new().route(
        "/signer/api/v1/sign/transaction",
        post(|Json(body): Json<serde_json::Value>| async move {
            assert_eq!(
                body,
                json!({
                    "key_id": "evm-key", "unsigned_hex": "02c0", "request_id": "evm-request-1",
                    "chain_family": "neox", "chain_id": 4_294_967_300u64
                })
            );
            Json(evm_witness())
        }),
    );
    let (endpoint, _runtime) = spawn(router);
    let signed = client(&endpoint, bearer())
        .sign_transaction(&evm_request())
        .expect("NeoX response")
        .allowed()
        .expect("allowed");
    assert_eq!(signed.signed_transaction.as_deref(), Some("0x02c0"));
    assert!(signed.signature_hex.is_none());
}

#[test]
fn neox_witness_rejects_wrong_identity_missing_bytes_and_conflicting_alias() {
    for (field, value) in [
        ("key_id", json!("different-key")),
        ("chain_family", json!("neo-n3")),
        ("signed_transaction", serde_json::Value::Null),
        ("signed_transaction", json!("0xnot-hex")),
        ("signature_hex", json!("0xdeadbeef")),
    ] {
        let mut response = evm_witness();
        response[field] = value;
        let router = Router::new().route(
            "/signer/api/v1/sign/transaction",
            post(move || {
                let response = response.clone();
                async move { Json(response) }
            }),
        );
        let (endpoint, _runtime) = spawn(router);
        assert_eq!(
            client(&endpoint, bearer())
                .sign_transaction(&evm_request())
                .expect_err(field)
                .kind(),
            SignerClientErrorKind::Protocol
        );
    }
}

#[test]
fn neox_legacy_transaction_alias_is_read_without_treating_it_as_a_signature() {
    let mut response = evm_witness();
    response
        .as_object_mut()
        .unwrap()
        .remove("signed_transaction");
    response["signature_hex"] = json!("0x02c0");
    let router = Router::new().route(
        "/signer/api/v1/sign/transaction",
        post(move || {
            let response = response.clone();
            async move { Json(response) }
        }),
    );
    let (endpoint, _runtime) = spawn(router);
    let signed = client(&endpoint, bearer())
        .sign_transaction(&evm_request())
        .expect("legacy reply")
        .allowed()
        .unwrap();
    assert_eq!(signed.signed_transaction.as_deref(), Some("0x02c0"));
}

#[test]
fn unsupported_signing_lanes_and_missing_chain_ids_fail_before_transport() {
    let client = client("http://127.0.0.1:1", bearer());
    let request = evm_request();
    assert_eq!(
        client.sign_consensus(&request).unwrap_err().kind(),
        SignerClientErrorKind::Request
    );
    let mut missing = request.clone();
    missing.chain_id = None;
    assert_eq!(
        client.sign_transaction(&missing).unwrap_err().kind(),
        SignerClientErrorKind::Request
    );
    let mut n3 = request;
    n3.chain_family = None;
    assert_eq!(
        client.sign_transaction(&n3).unwrap_err().kind(),
        SignerClientErrorKind::Request
    );
    let raw = RawSignRequest {
        key_id: "raw-key".to_string(),
        data_hex: "ab".to_string(),
        request_id: None,
        chain_family: Some("neox".to_string()),
    };
    assert_eq!(
        client.sign_raw(&raw).unwrap_err().kind(),
        SignerClientErrorKind::Request
    );
    let n3_raw = RawSignRequest {
        chain_family: Some("neo-n3".to_string()),
        ..raw
    };
    assert_eq!(
        serde_json::to_value(n3_raw).unwrap(),
        json!({"key_id": "raw-key", "data_hex": "ab"})
    );
}

#[test]
fn signer_key_metadata_keeps_evm_chain_identity_separate_from_network_magic() {
    let key: neo_nexus::signer_client::SignerKey = serde_json::from_value(json!({
        "key_id": "evm-key", "label": "private validator", "network": "private",
        "chain_family": "neox", "chain_id": 4_294_967_300u64, "network_magic": 5195086,
        "public_key": "02ab", "script_hash": "0x00", "address": "0x01",
        "verification_script": "", "signing_enabled": true
    }))
    .unwrap();
    assert_eq!(key.chain_family.as_deref(), Some("neox"));
    assert_eq!(key.chain_id, Some(4_294_967_300));
}

#[test]
fn redirects_and_malformed_successes_fail_closed() {
    let redirect = Router::new().route(
        "/signer/api/v1/keys",
        get(|| async {
            (
                StatusCode::TEMPORARY_REDIRECT,
                [("location", "http://127.0.0.1:1/stolen")],
            )
        }),
    );
    let (endpoint, _runtime) = spawn(redirect);
    let error = client(&endpoint, bearer())
        .list_keys()
        .expect_err("authenticated redirect must fail");
    assert_eq!(error.kind(), SignerClientErrorKind::Protocol);

    let malformed = Router::new().route(
        "/signer/api/v1/keys",
        get(|| async { (StatusCode::OK, Json(json!({"keys": []}))) }),
    );
    let (endpoint, _runtime) = spawn(malformed);
    let error = client(&endpoint, bearer())
        .list_keys()
        .expect_err("missing allowed discriminator must fail");
    assert_eq!(error.kind(), SignerClientErrorKind::Protocol);
}

#[test]
fn client_only_source_has_no_custody_implementation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let module = std::fs::read_to_string(root.join("src/signer_client.rs")).expect("module");
    let auth = std::fs::read_to_string(root.join("src/signer_client/auth.rs")).expect("auth");
    let client = std::fs::read_to_string(root.join("src/signer_client/client.rs")).expect("client");
    let model = std::fs::read_to_string(root.join("src/signer_client/model.rs")).expect("model");
    let source = format!("{module}\n{auth}\n{client}\n{model}").to_ascii_lowercase();

    for forbidden in [
        "p256::",
        "secp256k1::",
        "aes_gcm",
        "scrypt::",
        "open_private_key",
        "sealedprivatekey",
        "create table signer_",
        "insert into signer_",
    ] {
        assert!(
            !source.contains(forbidden),
            "signer client crossed into custody with {forbidden}"
        );
    }
}

fn decode_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if value.len() != N * 2 {
        return None;
    }
    let mut decoded = [0u8; N];
    for (index, slot) in decoded.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(decoded)
}

#[test]
fn canonical_workload_message_matches_the_service_protocol() {
    let seed = [3u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let digest = body_sha256(br#"{"key_id":"key-1","unsigned_hex":"00"}"#);
    let message = workload_signing_message(
        "caller-1",
        Some("neo-nexus:test"),
        1_700_000_000,
        "0123456789abcdef",
        "post",
        "/signer/api/v1/sign/transaction",
        &digest,
    );
    let signature = signing_key.sign(&message);
    assert!(signing_key
        .verifying_key()
        .verify(&message, &signature)
        .is_ok());
    assert!(String::from_utf8(message)
        .expect("canonical message is UTF-8")
        .starts_with("neoos-workload-v1\ncaller:caller-1\nsubject:neo-nexus:test\n"));
}

#[test]
fn workload_v2_signature_is_bound_to_the_exact_signer_origin() {
    let signing_key = SigningKey::from_bytes(&[3u8; 32]);
    let digest = body_sha256(b"request-body");
    let message = |audience: &str| {
        workload_signing_message_v2(
            audience,
            "caller-1",
            Some("neo-nexus:test"),
            1_700_000_000,
            "0123456789abcdef",
            "post",
            "/signer/api/v1/sign/transaction",
            &digest,
        )
    };
    let original = message("https://signer.example");
    assert!(String::from_utf8(original.clone())
        .unwrap()
        .starts_with("neoos-workload-v2\naudience:https://signer.example\ncaller:caller-1\n"));
    let signature = signing_key.sign(&original);
    assert!(signing_key
        .verifying_key()
        .verify(&original, &signature)
        .is_ok());
    assert!(signing_key
        .verifying_key()
        .verify(&message("https://other.example"), &signature)
        .is_err());
}

#[test]
fn client_timeouts_are_bounded_by_the_transport() {
    let router = Router::new().route(
        "/health",
        get(|| async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            Json(json!({"status": "ok"}))
        }),
    );
    let (endpoint, _runtime) = spawn(router);
    assert_eq!(
        client(&endpoint, bearer())
            .health()
            .expect("bounded health request")
            .status,
        "ok"
    );
}
