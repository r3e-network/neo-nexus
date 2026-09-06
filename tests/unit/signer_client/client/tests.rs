use std::{
    io::{BufRead, BufReader, Cursor, Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{channel, Receiver},
        Arc,
    },
    thread,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signature as Ed25519Signature, SigningKey};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{
    checked_id, parse_reply, read_limited_body, to_json, SignerClient, MAX_RESPONSE_BODY_BYTES,
};
use crate::signer_client::{
    CallerToken, Eip191Fulfillment, Eip191FulfillmentRequest, GenerateKeyRequest, Grant, KeyPublic,
    Outcome, RawSignRequest, SignRequest, SignerConfig, WorkloadCallerRequest,
};

const ADMIN_TOKEN: &str = "admin-token-cantus-9f31";
const CALLER_TOKEN: &str = "caller-token-relayed-4b77";

const WITNESS: &str = r#"{"allowed":true,"key_id":"key-1","script_hash":"0x1a2b","address":"NcgY","digest":"aabb","invocation_script":"0c40","verification_script":"0c21"}"#;
const RAW_WITNESS: &str = r#"{"allowed":true,"key_id":"key-1","script_hash":"0x1a2b","address":"NcgY","digest":"aabb","signature":"11","public_key":"02ab","invocation_script":"0c4011","verification_script":"0c2102ab"}"#;
const NEOX_SIGNATURE: &str = r#"{"allowed":true,"key_id":"key-x","script_hash":"0x1a2b","address":"0x1234","digest":"aabb","chain_family":"neox","chain_id":47763,"signed_transaction":"02f8","signature":"11","public_key":"04ab","future_receipt":{"block":9}}"#;
const EIP191_SIGNATURE: &str = r#"{"allowed":true,"key_id":"key-x","address":"0x1234","public_key":"02ab","digest":"0x01","message_hash":"0x02","signature":"0x03","chain_family":"neox","chain_id":47763,"oracle_contract":"0x2222222222222222222222222222222222222222","future_proof":{"version":2}}"#;
const KEY: &str = r#"{"allowed":true,"key_id":"key-1","label":"relay","network":"testnet","public_key":"02ab","script_hash":"0x1a2b","address":"NcgY","verification_script":"0c21","signing_enabled":true}"#;
const WORKLOAD_CALLER: &str = r#"{"allowed":true,"caller":{"id":"caller-workload","label":"relayer workload","auth_mode":"workload-ed25519","workload_public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","workload_subject":"relayer-prod","key_grant":{"mode":"only","key_ids":["key-1"]},"capabilities":["sign"],"allowed_origins":[],"created_at_unix":1755000000,"disabled":false,"future_attestation":{"pcr0":"bb"}}}"#;

#[test]
fn cleartext_transport_is_refused_before_any_network_request() {
    let client = SignerClient::new(
        SignerConfig::new(
            "http://127.0.0.1:1",
            Some(ADMIN_TOKEN.to_string()),
            Duration::from_secs(1),
        )
        .expect("loopback URL parses"),
    );
    let admin = client.config().admin().expect("admin identity");
    let error = client
        .list_keys(&admin)
        .expect_err("ordinary clients must not transmit over cleartext");
    assert!(
        error.to_string().contains("refusing cleartext"),
        "{error:#}"
    );
}

// -- the answer, turned into data -----------------------------------------

#[test]
fn a_denial_is_data_and_carries_the_services_status() -> Result<()> {
    let outcome: Outcome<KeyPublic> = parse_reply(
        403,
        r#"{"allowed":false,"code":"recipient-blacklisted","message":"this recipient is on the key's blacklist"}"#,
        "POST",
        "/sign/transaction",
    )
    .expect("a refusal is a completed conversation");

    let refusal = match outcome {
        Outcome::Refused(refusal) => refusal,
        Outcome::Allowed(_) => bail!("a 403 with allowed:false parsed as an allowed answer"),
    };
    assert_eq!(refusal.code, "recipient-blacklisted");
    // Carried, not re-derived: §5's code→status table belongs to the service, and
    // a second copy of it here is a second thing to keep current.
    assert_eq!(refusal.status, 403);
    assert_eq!(
        refusal.summary(),
        "recipient-blacklisted: this recipient is on the key's blacklist"
    );
    Ok(())
}

#[test]
fn only_a_2xx_may_carry_a_yes() {
    // A proxy that repeats the vault's JSON on a redirect or an error page must
    // not become a signature the service never agreed to sign.
    let outcome: Result<Outcome<KeyPublic>> = parse_reply(404, KEY, "GET", "/keys/key-1");
    let error = outcome.expect_err("a 404 cannot hold an allowed answer");
    assert!(error.to_string().contains("reserves for 2xx"), "{error}");
}

#[test]
fn a_refusal_without_a_code_is_reported_rather_than_invented() {
    let outcome: Result<Outcome<KeyPublic>> = parse_reply(
        403,
        r#"{"allowed":false,"message":"no code, so no vocabulary"}"#,
        "GET",
        "/keys/key-1",
    );
    let error = outcome.expect_err("a shape nobody can branch on is not an answer");
    assert!(error.to_string().contains("no code"), "{error}");
}

#[test]
fn an_allowed_answer_ignores_fields_the_contract_does_not_name_yet() -> Result<()> {
    // §5: adding a field is not a breaking change. A client that refused one would
    // turn every service release into a coordinated deployment.
    let outcome: Outcome<KeyPublic> = parse_reply(
        200,
        r#"{"allowed":true,"key_id":"key-1","label":"relay","network":"testnet","public_key":"02ab","script_hash":"0x1a2b","address":"NcgY","verification_script":"0c21","signing_enabled":true,"future_field":{"a":1}}"#,
        "GET",
        "/keys/key-1",
    )
    .expect("an unknown field must not break a known answer");

    let info = match outcome {
        Outcome::Allowed(info) => info,
        Outcome::Refused(refusal) => bail!("allowed answer parsed as refusal: {refusal:?}"),
    };
    assert_eq!(info.key_id, "key-1");
    assert!(info.signing_enabled);
    Ok(())
}

#[test]
fn a_body_that_is_not_the_contract_is_named_by_length_only() {
    // The one error path here can see a response body — and a response body on
    // this contract includes caller tokens.
    let outcome: Result<Outcome<KeyPublic>> = parse_reply(
        200,
        &format!("<html>a proxy page mentioning {ADMIN_TOKEN}</html>"),
        "GET",
        "/keys",
    );
    let error = outcome.expect_err("not JSON is not an answer");
    let text = error.to_string();
    assert!(text.contains("bytes that are not JSON"), "{text}");
    assert!(!text.contains(ADMIN_TOKEN), "{text} quotes the body");
}

#[test]
fn a_bounded_response_is_read_and_parsed_without_changing_the_contract() -> Result<()> {
    let body = read_limited_body(Cursor::new(KEY.as_bytes()), Some(u64::try_from(KEY.len())?))?;
    let text = std::str::from_utf8(&body).context("the fixture must be UTF-8")?;
    let outcome: Outcome<KeyPublic> = parse_reply(200, text, "GET", "/keys/key-1")?;
    let key = outcome
        .into_parts()
        .context("the bounded valid response should remain allowed")?;
    assert_eq!(key.key_id, "key-1");
    Ok(())
}

#[test]
fn an_oversized_response_is_rejected_without_echoing_its_body() {
    let marker = "caller-token-that-must-not-reach-the-error";
    let mut response = vec![b'x'; MAX_RESPONSE_BODY_BYTES + 1];
    response[..marker.len()].copy_from_slice(marker.as_bytes());

    let error = read_limited_body(Cursor::new(response), None)
        .expect_err("a response over the limit must not be buffered or parsed");
    let message = error.to_string();
    assert!(message.contains("response exceeded"), "{message}");
    assert!(message.contains("byte limit"), "{message}");
    assert!(
        !message.contains(marker),
        "{message} quotes the response body"
    );
}

#[test]
fn the_http_transport_rejects_a_declared_oversized_response_before_json_parsing() {
    let marker = "rotated-caller-token-that-must-stay-in-the-body";
    let response = format!(
        "{{\"allowed\":true,\"token\":\"{marker}\",\"padding\":\"{}\"}}",
        "x".repeat(MAX_RESPONSE_BODY_BYTES)
    );
    let (base_url, _requests) = spawn_stub(move |_| reply(200, &response));
    let client = client(&base_url, None);

    let error = client
        .list_keys(&CallerToken::bearer(CALLER_TOKEN))
        .expect_err("an oversized HTTP response must not reach JSON decoding");
    let message = error.to_string();
    assert!(message.contains("declared more"), "{message}");
    assert!(message.contains("byte limit"), "{message}");
    assert!(
        !message.contains(marker),
        "{message} quotes the response body"
    );
}

// -- the path, checked before it is built ---------------------------------

#[test]
fn a_real_id_is_accepted_as_issued() {
    assert_eq!(
        checked_id("key-8f14e45fceea167a5a36dedd4bea2543").unwrap(),
        "key-8f14e45fceea167a5a36dedd4bea2543"
    );
    assert!(checked_id(" caller-99 ").unwrap().starts_with("caller-99"));
}

#[test]
fn an_id_that_could_add_a_path_segment_is_refused() {
    for candidate in [
        "",
        "   ",
        "..",
        "../../admin",
        "key/../x",
        "key%2F..",
        "key 1",
        &format!("key-{}", "x".repeat(200)),
    ] {
        assert!(
            checked_id(candidate).is_err(),
            "{candidate:?} must not reach a URL path"
        );
    }
}

#[test]
fn a_request_body_that_cannot_be_encoded_says_so_without_quoting_it() {
    // Nothing this module builds can fail here, so the message is about the bug —
    // and it must not become a way to print the body it failed on.
    struct Unencodable;
    impl serde::Serialize for Unencodable {
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("secret-shaped failure"))
        }
    }
    let error = to_json(&Unencodable).expect_err("the test type always fails");
    assert!(
        !error.to_string().contains("secret-shaped"),
        "{error} quotes the failure"
    );
}

// -- the transport, against a stub ----------------------------------------

#[test]
fn the_relay_forwards_the_callers_own_credential_and_headers() {
    let (base_url, requests) = spawn_stub(|_| reply(200, WITNESS));
    let client = client(&base_url, Some(ADMIN_TOKEN));

    let outcome = client
        .sign_transaction(
            &CallerToken::browser(CALLER_TOKEN, Some("http://localhost:3000"), None),
            "key-1",
            "0001",
        )
        .unwrap();
    assert!(matches!(outcome, Outcome::Allowed(_)), "{outcome:?}");

    let request = requests.recv().expect("the stub never saw a request");
    assert!(
        request.starts_with("POST /signer/api/v1/sign/transaction HTTP"),
        "{request}"
    );
    assert!(request.contains(&format!("authorization: bearer {CALLER_TOKEN}")));
    assert!(request.contains("origin: http://localhost:3000"));
    assert!(!request.contains("referer:"));
    // The relay's own admin credential must not ride along: the service audits
    // whoever signed, and a substituted credential would name the console.
    assert!(!request.contains(ADMIN_TOKEN));
    assert!(request.contains(r#"{"key_id":"key-1","unsigned_hex":"0001"}"#));
}

#[test]
fn an_explicit_sign_request_carries_idempotency_and_neox_identity() {
    let (base_url, requests) = spawn_stub(|_| reply(200, NEOX_SIGNATURE));
    let client = client(&base_url, None);
    let request = SignRequest {
        key_id: "key-x".to_string(),
        unsigned_hex: "02f8".to_string(),
        request_id: Some("request-42".to_string()),
        chain_family: Some("neox".to_string()),
        chain_id: Some(47_763),
    };

    let signed = client
        .sign_transaction_request(&CallerToken::bearer(CALLER_TOKEN), &request)
        .unwrap()
        .into_parts()
        .expect("the signer allowed the NeoX transaction");
    assert_eq!(signed.chain_family.as_deref(), Some("neox"));
    assert_eq!(signed.signed_transaction.as_deref(), Some("02f8"));
    assert_eq!(signed.signature.as_deref(), Some("11"));
    assert_eq!(signed.public_key.as_deref(), Some("04ab"));
    assert_eq!(signed.additional_fields["future_receipt"]["block"], 9);
    assert!(
        !signed.additional_fields.contains_key("allowed"),
        "the transport envelope leaked into the endpoint payload"
    );

    let request = requests.recv().expect("the stub never saw a request");
    assert!(
        request.contains(r#""request_id":"request-42""#),
        "{request}"
    );
    assert!(request.contains(r#""chain_family":"neox""#), "{request}");
    assert!(request.contains(r#""chain_id":47763"#), "{request}");
}

#[test]
fn the_eip191_lane_forwards_only_structured_contract_bound_fields() {
    let (base_url, requests) = spawn_stub(|_| reply(200, EIP191_SIGNATURE));
    let client = client(&base_url, None);
    let request = Eip191FulfillmentRequest {
        key_id: "key-x".to_string(),
        request_id: "relayer:neox:7".to_string(),
        chain_id: 47_763,
        oracle_contract: "0x2222222222222222222222222222222222222222".to_string(),
        fulfillment: Eip191Fulfillment {
            request_id: "7".to_string(),
            app_id: "app:1".to_string(),
            module_id: "oracle.fetch".to_string(),
            operation: "privacy_oracle".to_string(),
            success: true,
            error: String::new(),
        },
        result_bytes_hex: "0x68656c6c6f".to_string(),
    };

    let signed = client
        .sign_eip191_fulfillment(&CallerToken::bearer(CALLER_TOKEN), &request)
        .unwrap()
        .into_parts()
        .expect("the structured fulfillment was allowed");
    assert_eq!(signed.chain_family, "neox");
    assert_eq!(signed.chain_id, 47_763);
    assert_eq!(signed.additional_fields["future_proof"]["version"], 2);

    let request = requests.recv().expect("the stub never saw a request");
    assert!(
        request.starts_with("POST /signer/api/v1/sign/eip191-fulfillment HTTP"),
        "{request}"
    );
    assert!(request.contains(r#""request_id":"relayer:neox:7""#));
    assert!(request.contains(r#""result_bytes_hex":"0x68656c6c6f""#));
    assert!(request.contains(r#""module_id":"oracle.fetch""#));
    assert!(!request.contains(r#""digest""#), "{request}");
    assert!(!request.contains(r#""message_hash""#), "{request}");
}

#[test]
fn a_server_side_caller_sends_no_origin_at_all() {
    let (base_url, requests) = spawn_stub(|_| reply(200, WITNESS));
    let client = client(&base_url, None);

    client
        .sign_consensus(&CallerToken::bearer(CALLER_TOKEN), "key-1", "0001")
        .unwrap();

    let request = requests.recv().expect("the stub never saw a request");
    assert!(request.starts_with("POST /signer/api/v1/sign/consensus HTTP"));
    // Absent rather than empty: a caller declared without origins is refused
    // `origin-unexpected`, so a header invented here would break calls that would
    // otherwise succeed.
    assert!(!request.contains("origin:"), "{request}");
}

#[test]
fn the_raw_lane_uses_data_hex_and_carries_the_extra_v1_fields() {
    let (base_url, requests) = spawn_stub(|_| reply(200, RAW_WITNESS));
    let client = client(&base_url, None);

    let raw = client
        .sign_raw(&CallerToken::bearer(CALLER_TOKEN), "key-1", "cafe")
        .unwrap()
        .into_parts()
        .expect("the remote boundary allowed raw signing");
    assert_eq!(raw.signature, "11");
    assert_eq!(raw.public_key, "02ab");

    let request = requests.recv().expect("the stub never saw a request");
    assert!(request.starts_with("POST /signer/api/v1/sign/raw HTTP"));
    assert!(request.contains(r#"{"key_id":"key-1","data_hex":"cafe"}"#));
    assert!(!request.contains("unsigned_hex"), "{request}");
}

#[test]
fn the_raw_lane_carries_a_request_id_without_inventing_chain_identity() {
    let (base_url, requests) = spawn_stub(|_| reply(200, RAW_WITNESS));
    let client = client(&base_url, None);
    let request = RawSignRequest {
        key_id: "key-1".to_string(),
        data_hex: "cafe".to_string(),
        request_id: Some("raw-9".to_string()),
    };
    client
        .sign_raw_request(&CallerToken::bearer(CALLER_TOKEN), &request)
        .unwrap();

    let request = requests.recv().expect("the stub never saw a request");
    assert!(request.contains(r#""request_id":"raw-9""#), "{request}");
    assert!(!request.contains("chain_family"), "{request}");
    assert!(!request.contains("chain_id"), "{request}");
}

#[test]
fn a_denied_signature_arrives_as_a_decision_rather_than_a_failure() -> Result<()> {
    let (base_url, _requests) = spawn_stub(|_| {
        reply(
            403,
            r#"{"allowed":false,"code":"single-amount-exceeded","message":"the amount is over this key's ceiling"}"#,
        )
    });
    let client = client(&base_url, None);

    let outcome = client
        .sign_transaction(&CallerToken::bearer(CALLER_TOKEN), "key-1", "0001")
        .expect("a refusal is a round trip that worked");

    let refusal = match outcome {
        Outcome::Refused(refusal) => refusal,
        Outcome::Allowed(_) => bail!("the denied signature parsed as an allowed answer"),
    };
    assert_eq!(refusal.code, "single-amount-exceeded");
    assert_eq!(refusal.status, 403);
    Ok(())
}

#[test]
fn the_console_presents_its_admin_credential_and_configured_origin() {
    let (base_url, requests) = spawn_stub(|_| reply(200, r#"{"allowed":true,"keys":[]}"#));
    let client = SignerClient::new(
        SignerConfig::new_insecure_loopback_with_origin_for_test(
            &base_url,
            Some(ADMIN_TOKEN.to_string()),
            Some("https://nexus.example".to_string()),
            Duration::from_secs(5),
        )
        .unwrap(),
    );
    let admin = client.config().admin().expect("configured");

    let keys = client
        .list_keys(&admin)
        .unwrap()
        .into_parts()
        .expect("allowed");
    assert!(keys.is_empty());

    let request = requests.recv().expect("the stub never saw a request");
    assert!(
        request.starts_with("GET /signer/api/v1/keys HTTP"),
        "{request}"
    );
    assert!(request.contains(&format!("authorization: bearer {ADMIN_TOKEN}")));
    assert!(request.contains("origin: https://nexus.example"));
}

#[test]
fn workload_admin_signs_the_exact_native_route_and_body() -> Result<()> {
    let seed = [0x42_u8; 32];
    let verifying_key = SigningKey::from_bytes(&seed).verifying_key();
    let (base_url, requests) = spawn_stub(|_| reply(200, KEY));
    let client = SignerClient::new(SignerConfig::new_insecure_loopback_with_workload_for_test(
        &base_url,
        "nexus-admin-1",
        seed,
        Some("neo-nexus-production".to_string()),
        Duration::from_secs(5),
    )?);
    let admin = client
        .config()
        .admin()
        .context("workload admin credential is missing")?;
    client
        .generate_key_request(
            &admin,
            &GenerateKeyRequest::neo_n3("workload key", "testnet", Some(894_710_606)),
        )?
        .into_parts()?;

    let request = requests.recv().context("the stub never saw a request")?;
    assert!(
        request.starts_with("POST /signer/api/v1/keys HTTP"),
        "{request}"
    );
    assert!(!request.contains("authorization:"), "{request}");
    assert!(!request.contains("origin:"), "{request}");
    let protocol = request_header(&request, "x-neoos-workload-protocol")?;
    let audience = request_header(&request, "x-neoos-audience")?;
    let caller = request_header(&request, "x-neoos-caller")?;
    let timestamp = request_header(&request, "x-neoos-timestamp")?;
    let nonce = request_header(&request, "x-neoos-nonce")?;
    let signature_hex = request_header(&request, "x-neoos-signature")?;
    assert_eq!(protocol, "neoos-workload-v2");
    assert_eq!(audience, base_url);
    assert_eq!(caller, "nexus-admin-1");
    let parsed_timestamp = timestamp.parse::<u64>().context("canonical timestamp")?;
    assert_eq!(parsed_timestamp.to_string(), timestamp);
    assert!((16..=128).contains(&nonce.len()));
    assert!(
        nonce
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')),
        "{nonce}"
    );
    assert_eq!(signature_hex.len(), 128);
    assert!(signature_hex
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')));

    let body = request
        .split_once("\r\n\r\n")
        .context("the workload request has no body delimiter")?
        .1;
    let body_digest = Sha256::digest(body.as_bytes());
    let canonical = format!(
        "neoos-workload-v2\naudience:{audience}\ncaller:{caller}\nsubject:neo-nexus-production\ntimestamp:{timestamp}\nnonce:{nonce}\nmethod:POST\nroute:/signer/api/v1/keys\nbody-sha256:{}\norigin:",
        lowercase_hex(&body_digest)
    );
    let signature = Ed25519Signature::from_bytes(&decode_hex_signature(signature_hex)?);
    verifying_key
        .verify_strict(canonical.as_bytes(), &signature)
        .context("workload signature did not bind the exact signer canonical message")?;
    Ok(())
}

#[test]
fn a_private_networks_magic_travels_on_generate_and_is_absent_when_unnamed() {
    let (base_url, requests) = spawn_stub(|_| reply(200, KEY));
    let client = client(&base_url, Some(ADMIN_TOKEN));
    let admin = client.config().admin().unwrap();

    client
        .generate_key(&admin, "shared-default", "private", None)
        .unwrap()
        .into_parts()
        .expect("custody accepted the key");
    let request = requests.recv().expect("the stub never saw a request");
    assert!(
        !request.contains("network_magic"),
        "an unnamed magic must stay off the wire — the service's canonical \
         value applies, and a zero it could misread as would not: {request}"
    );
    assert!(!request.contains("chain_family"), "{request}");
    assert!(!request.contains("chain_id"), "{request}");

    client
        .generate_key(&admin, "deployment", "private", Some(4_242_424))
        .unwrap()
        .into_parts()
        .expect("custody accepted the key");
    let request = requests.recv().expect("the stub never saw a request");
    assert!(
        request.contains(r#""network_magic":4242424"#),
        "the magic is bound at custody, in the same body that creates the key: {request}"
    );

    client
        .generate_key_request(
            &admin,
            &GenerateKeyRequest {
                label: "NeoX treasury".to_string(),
                network: "mainnet".to_string(),
                chain_family: Some("neox".to_string()),
                chain_id: Some(47_763),
                network_magic: None,
            },
        )
        .unwrap()
        .into_parts()
        .expect("custody accepted the NeoX key");
    let request = requests.recv().expect("the stub never saw a request");
    assert!(request.contains(r#""chain_family":"neox""#), "{request}");
    assert!(request.contains(r#""chain_id":47763"#), "{request}");
    assert!(!request.contains("network_magic"), "{request}");
}

#[test]
fn a_workload_caller_carries_only_public_identity_material() {
    let (base_url, requests) = spawn_stub(|_| reply(200, WORKLOAD_CALLER));
    let client = client(&base_url, Some(ADMIN_TOKEN));
    let admin = client.config().admin().unwrap();
    let request = WorkloadCallerRequest {
        label: "relayer workload".to_string(),
        key_grant: Grant::only(["key-1"]),
        capabilities: vec!["sign".to_string()],
        allowed_origins: Vec::new(),
        workload_public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        workload_subject: Some("relayer-prod".to_string()),
    };

    let created = client
        .create_workload_caller(&admin, &request)
        .unwrap()
        .into_parts()
        .expect("custody accepted the public workload identity");
    assert_eq!(
        created.caller.auth_mode.as_deref(),
        Some("workload-ed25519")
    );
    assert_eq!(
        created.caller.workload_subject.as_deref(),
        Some("relayer-prod")
    );
    assert_eq!(
        created.caller.additional_fields["future_attestation"]["pcr0"],
        "bb"
    );

    let request = requests.recv().expect("the stub never saw a request");
    assert!(
        request.starts_with("POST /signer/api/v1/callers/workload HTTP"),
        "{request}"
    );
    assert!(request.contains(r#""workload_public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#));
    assert!(request.contains(r#""workload_subject":"relayer-prod""#));
    for forbidden in ["private_key", "passphrase", "token\""] {
        assert!(
            !request.contains(forbidden),
            "{forbidden} leaked into {request}"
        );
    }
}

#[test]
fn a_refusals_detail_stops_at_the_audit_route() -> Result<()> {
    // `GET /audit` is §5's named transport for `detail`; a sign refusal is not,
    // and a client that read one from it would be a reason to start sending it.
    let (base_url, _requests) = spawn_stub(|_| {
        reply(
            403,
            r#"{"allowed":false,"code":"asset-blacklisted","message":"no","detail":"asset 0xab"}"#,
        )
    });
    let client = client(&base_url, None);

    let outcome = client
        .sign_transaction(&CallerToken::bearer(CALLER_TOKEN), "key-1", "0001")
        .unwrap();
    let refusal = match outcome {
        Outcome::Refused(refusal) => refusal,
        Outcome::Allowed(_) => bail!("expected the refusal, received an allowed answer"),
    };
    assert_eq!(refusal.code, "asset-blacklisted");
    assert!(
        !format!("{refusal:?}").contains("0xab"),
        "{refusal:?} carries the detail"
    );
    Ok(())
}

#[test]
fn an_audit_filter_travels_as_a_query_string() {
    let (base_url, requests) = spawn_stub(|_| reply(200, r#"{"allowed":true,"entries":[]}"#));
    let client = client(&base_url, Some(ADMIN_TOKEN));
    let admin = client.config().admin().unwrap();

    let rows = client
        .list_audit(&admin, Some("key-1"), Some(25))
        .unwrap()
        .into_parts()
        .expect("allowed");
    assert!(rows.is_empty());

    let request = requests.recv().expect("the stub never saw a request");
    assert!(
        request.starts_with("GET /signer/api/v1/audit?key_id=key-1&limit=25 HTTP"),
        "{request}"
    );
}

#[test]
fn a_redirect_is_not_a_second_endpoint() {
    // §5.1 counts a redirect as a different service. Following one would deliver
    // the bearer to whoever the proxy pointed at, so the second hop must not be
    // asked for at all.
    let counter = Arc::new(AtomicUsize::new(0));
    let (base_url, _requests) = spawn_stub_counting(Arc::clone(&counter), |_| {
        reply(302, "<html>moved, and the token would go here</html>")
    });
    let client = client(&base_url, Some(ADMIN_TOKEN));
    let admin = client.config().admin().unwrap();

    let error = client
        .list_keys(&admin)
        .expect_err("a 302 is not the contract");
    // The whole chain: the transport adds a context line, and a refusal to speak
    // the contract is the cause underneath it.
    let text = format!("{error:#}");
    assert!(
        text.contains("not JSON"),
        "expected a shape complaint, got {text}"
    );
    assert!(text.contains("302"), "expected the status, got {text}");

    // The client has returned, so any second request has already been written to
    // the socket; the wait covers only the accept that would record it.
    thread::sleep(Duration::from_millis(100));
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "the client followed the redirect"
    );
}

#[test]
fn an_unreachable_service_is_an_error_rather_than_a_refusal() {
    // Port 1: nothing listens there. The console has to say "custody is not
    // there" in a different sentence from "custody said no".
    let config = SignerConfig::new_insecure_loopback_for_test(
        "http://127.0.0.1:1",
        None,
        Duration::from_millis(400),
    )
    .unwrap();
    let client = SignerClient::new(config);

    let error = client
        .list_keys(&CallerToken::bearer(CALLER_TOKEN))
        .expect_err("nothing is listening");
    let text = format!("{error:#}");
    assert!(
        text.contains("could not reach the signer service"),
        "{text}"
    );
    assert!(text.contains("127.0.0.1:1"), "{text}");
}

#[test]
fn a_relayed_body_names_only_what_the_contract_allows() {
    // The service `deny_unknown_fields`es every request, so a field added here on
    // the way to a refactor is a `signer-request-unreadable` in production.
    let (base_url, requests) = spawn_stub(|_| reply(200, WITNESS));
    let client = client(&base_url, None);
    client
        .sign_transaction(&CallerToken::bearer(CALLER_TOKEN), "key-1", "00")
        .unwrap();

    let request = requests.recv().expect("the stub never saw a request");
    let body = request
        .split_once("\r\n\r\n")
        .expect("a request with a body")
        .1;
    let sent: Value = serde_json::from_str(body).expect("the client sent JSON");
    assert_eq!(sent.as_object().map(|object| object.len()), Some(2));
    assert!(sent.get("key_id").is_some() && sent.get("unsigned_hex").is_some());
}

// -- the fixture ----------------------------------------------------------

/// This client against the real service, on every v1 route this pure consumer
/// supports.
///
/// Ignored because it needs a deployment: point
/// `NEONEXUS_SIGNER_URL` and one protected admin profile at a disposable
/// development signer, then run the exact ignored test named in
/// `docs/signer-service-design.md`. The two import routes are intentionally
/// absent: this consumer accepts no private-key or passphrase input.
#[test]
#[ignore = "needs a running signer service (see NEONEXUS_SIGNER_URL)"]
fn a_real_service_and_this_client_agree_on_every_supported_route() -> Result<()> {
    use crate::signer_client::{
        Eip191Fulfillment, Eip191FulfillmentRequest, Grant, Policy, WindowLimit,
        WorkloadCallerRequest,
    };

    let mut config =
        SignerConfig::from_env()?.expect("set NEONEXUS_SIGNER_URL and an admin profile");
    if config.uses_cleartext() && config.is_loopback() {
        config.enable_insecure_loopback_test_transport()?;
    }
    let client = SignerClient::new(config);
    let admin = client
        .config()
        .admin()
        .expect("configure a protected bearer or workload admin profile");

    // 1. A key, generated rather than imported, so the probe leaves nothing of a
    //    real wallet behind.
    let key = expect_allowed(
        "POST /keys",
        client.generate_key(&admin, "neo-nexus client contract probe", "testnet", None)?,
    )?;
    assert!(key.key_id.starts_with("key-"), "{key:?}");
    let mut cleanup = LiveCleanup::new(&client, &key.key_id);

    // 2. A boundary that was never written reads back as everything closed.
    let boundary = expect_allowed(
        "GET /keys/{id}/policy",
        client.key_boundary(&admin, &key.key_id)?,
    )?;
    assert_eq!(boundary.key.key_id, key.key_id);
    assert_eq!(
        boundary.policy,
        Policy::default(),
        "a blank boundary must not mean anything the client guesses"
    );

    // 3. A ceiling the size of an i128, which is the reason amounts are text.
    let ceiling = "170141183460469231731687303715884105727";
    let wanted = Policy {
        allow_raw: true,
        // Raw signing is intentionally a dedicated-key authority. The amount
        // fields still exercise lossless i128-shaped text round-tripping and
        // the returned policy advice explains that transfer rules are inert.
        allow_transfer: false,
        max_single_amount: Some(ceiling.to_string()),
        window_limit: Some(WindowLimit {
            seconds: 3_600,
            max_amount: "1000".to_string(),
        }),
        ..Policy::default()
    };
    let saved = expect_allowed(
        "POST /keys/{id}/policy",
        client.save_policy(&admin, &key.key_id, &wanted)?,
    )?;
    assert_eq!(saved.policy, wanted);
    // The advice is the service's judgement, carried through rather than
    // recomputed here: inert transfer ceilings on a raw-only key have to arrive
    // named, on the write and again on the read.
    let codes: Vec<&str> = saved
        .problems
        .iter()
        .map(|problem| problem.code.as_str())
        .collect();
    assert!(
        codes.contains(&"transfer-rules-without-transfer"),
        "the boundary's advice was lost on the way through the client: {codes:?}"
    );
    let reread = expect_allowed(
        "GET /keys/{id}/policy",
        client.key_boundary(&admin, &key.key_id)?,
    )?;
    assert_eq!(
        reread.policy, wanted,
        "the stored boundary is the one asked for"
    );
    assert_eq!(
        reread.problems, saved.problems,
        "the page that shows a boundary saved earlier must say what the save said"
    );

    // The same live probe covers the second chain family. NeoX Testnet T4 is
    // chain-bound at generation and the policy repeats that identity before any
    // EIP-1559 transaction can be signed.
    const NEOX_CHAIN_ID: u64 = 12_227_332;
    const NEOX_ORACLE_CONTRACT: &str = "0x2222222222222222222222222222222222222222";
    const FULFILL_REQUEST_SELECTOR: &str = "a4c3baa4";
    let neox_key = expect_allowed(
        "POST /keys NeoX",
        client.generate_key_request(
            &admin,
            &GenerateKeyRequest {
                label: "neo-nexus NeoX contract probe".to_string(),
                network: "testnet".to_string(),
                chain_family: Some("neox".to_string()),
                chain_id: Some(NEOX_CHAIN_ID),
                network_magic: None,
            },
        )?,
    )?;
    cleanup.neox_key_id = Some(neox_key.key_id.clone());
    assert_eq!(neox_key.chain_family.as_deref(), Some("neox"));
    assert_eq!(neox_key.chain_id, Some(NEOX_CHAIN_ID));

    let neox_policy = Policy {
        allow_transfer: true,
        allow_contract_call: true,
        contract_whitelist: vec![NEOX_ORACLE_CONTRACT.to_string()],
        chain_family: Some("neox".to_string()),
        evm_max_gas_price: Some("2".to_string()),
        evm_max_gas_limit: Some(21_000),
        evm_method_whitelist: vec![FULFILL_REQUEST_SELECTOR.to_string()],
        evm_chain_id: Some(NEOX_CHAIN_ID),
        ..Policy::default()
    };
    let saved_neox = expect_allowed(
        "POST /keys/{id}/policy NeoX",
        client.save_policy(&admin, &neox_key.key_id, &neox_policy)?,
    )?;
    assert_eq!(saved_neox.policy, neox_policy);
    let reread_neox = expect_allowed(
        "GET /keys/{id}/policy NeoX",
        client.key_boundary(&admin, &neox_key.key_id)?,
    )?;
    assert_eq!(reread_neox.key.chain_family.as_deref(), Some("neox"));
    assert_eq!(reread_neox.key.chain_id, Some(NEOX_CHAIN_ID));
    assert_eq!(reread_neox.policy, neox_policy);

    // 4. The identity route is guarded by `sign`, so the console's admin
    //    credential must be refused there — by name, not as an unknown key.
    let identity = client.key_info(&admin, &key.key_id)?;
    let refusal = match identity {
        Outcome::Allowed(info) => bail!("an admin credential signed with {info:?}"),
        Outcome::Refused(refusal) => refusal,
    };
    assert_eq!(refusal.code, "admin-capability-missing", "{refusal:?}");
    assert_eq!(refusal.status, 403);

    // 5. The key is in the list a whole-vault caller reads.
    let keys = expect_allowed("GET /keys", client.list_keys(&admin)?)?;
    assert!(
        keys.iter().any(|info| info.key_id == key.key_id),
        "the new key is missing from {} keys",
        keys.len()
    );
    assert!(
        keys.iter().any(|info| {
            info.key_id == neox_key.key_id
                && info.chain_family.as_deref() == Some("neox")
                && info.chain_id == Some(NEOX_CHAIN_ID)
        }),
        "the chain-bound NeoX key is missing from {keys:?}"
    );

    // 6. Semantic and raw signing are separate credentials as well as separate
    //    key policies. The service deliberately permits exactly one capability
    //    per non-admin caller, so this probe must exercise both identities.
    let created = expect_allowed(
        "POST /callers",
        client.create_caller(
            &admin,
            "neo-nexus client contract probe",
            &Grant::only([key.key_id.clone(), neox_key.key_id.clone()]),
            &["sign".to_string()],
            &[],
        )?,
    )?;
    cleanup.caller_id = Some(created.caller.id.clone());
    assert_eq!(created.caller.capabilities, vec!["sign".to_string()]);
    assert_eq!(
        created.caller.key_grant.key_ids,
        Grant::only([key.key_id.clone(), neox_key.key_id.clone()]).key_ids
    );
    let raw_created = expect_allowed(
        "POST /callers raw",
        client.create_caller(
            &admin,
            "neo-nexus raw contract probe",
            &Grant::only([key.key_id.clone()]),
            &["raw_sign".to_string()],
            &[],
        )?,
    )?;
    cleanup.raw_caller_id = Some(raw_created.caller.id.clone());
    assert_eq!(
        raw_created.caller.capabilities,
        vec!["raw_sign".to_string()]
    );

    // A workload identity carries only its public verification key. This is a
    // separate supported management route, not a secret-import path.
    let workload = expect_allowed(
        "POST /callers/workload",
        client.create_workload_caller(
            &admin,
            &WorkloadCallerRequest {
                label: "neo-nexus workload contract probe".to_string(),
                key_grant: Grant::only([key.key_id.clone()]),
                capabilities: vec!["sign".to_string()],
                allowed_origins: Vec::new(),
                workload_public_key:
                    "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a".to_string(),
                workload_subject: Some("neo-nexus-live-probe".to_string()),
            },
        )?,
    )?;
    cleanup.workload_caller_id = Some(workload.caller.id.clone());
    assert_eq!(
        workload.caller.auth_mode.as_deref(),
        Some("workload-ed25519")
    );
    assert_eq!(
        workload.caller.workload_subject.as_deref(),
        Some("neo-nexus-live-probe")
    );

    // 7. The signing caller can read the identity it was granted.
    let identity = expect_allowed(
        "GET /keys/{id}",
        client.key_info(&CallerToken::bearer(&created.token), &key.key_id)?,
    )?;
    assert_eq!(identity.key_id, key.key_id);
    assert_eq!(identity.public_key, key.public_key);

    // 8. Raw signing is a current v1 lane. It is allowed only because the
    //    boundary above opened `allow_raw`, and the extra raw response fields
    //    must survive this client's parsing.
    let raw = expect_allowed(
        "POST /sign/raw",
        client.sign_raw(
            &CallerToken::bearer(&raw_created.token),
            &key.key_id,
            "6e656f2d6e657875732d636f6d706174",
        )?,
    )?;
    assert_eq!(raw.key_id, key.key_id);
    assert_eq!(raw.signature.len(), 128, "{raw:?}");
    assert_eq!(raw.public_key, key.public_key);

    // The semantic dBFT lane uses the same key only after the raw-only
    // boundary is replaced. This proves that the client and service agree on
    // a well-formed, sender-bound consensus payload, not merely on the shape
    // of a refusal for junk bytes.
    let consensus_policy = Policy {
        allow_consensus: true,
        ..Policy::default()
    };
    let saved_consensus = expect_allowed(
        "POST /keys/{id}/policy consensus",
        client.save_policy(&admin, &key.key_id, &consensus_policy)?,
    )?;
    assert_eq!(saved_consensus.policy, consensus_policy);
    let consensus_payload = live_consensus_payload(&key.script_hash)?;
    let consensus = expect_allowed(
        "POST /sign/consensus",
        client.sign_consensus(
            &CallerToken::bearer(&created.token),
            &key.key_id,
            &consensus_payload,
        )?,
    )?;
    assert_eq!(consensus.key_id, key.key_id);
    assert_eq!(consensus.script_hash, key.script_hash);
    assert!(!consensus.digest.is_empty(), "{consensus:?}");

    // The bytes are the signer test fixture's canonical EIP-1559 unsigned
    // transaction: Testnet T4, nonce 1, fees 1/2, gas 21000, 60 native units to
    // 0x22…22, empty calldata and access list.
    let neox_unsigned = "02e283ba93040101028252089422222222222222222222222222222222222222223c80c0";
    let neox_signature = expect_allowed(
        "POST /sign/transaction NeoX",
        client.sign_transaction_request(
            &CallerToken::bearer(&created.token),
            &SignRequest {
                key_id: neox_key.key_id.clone(),
                unsigned_hex: neox_unsigned.to_string(),
                request_id: Some("neo-nexus-live-neox-1".to_string()),
                chain_family: Some("neox".to_string()),
                chain_id: Some(NEOX_CHAIN_ID),
            },
        )?,
    )?;
    assert_eq!(neox_signature.chain_family.as_deref(), Some("neox"));
    let signed_transaction = neox_signature
        .signed_transaction
        .as_deref()
        .context("NeoX success omitted signed_transaction")?;
    assert!(signed_transaction.starts_with("0x02"), "{neox_signature:?}");
    assert_eq!(
        neox_signature.signature_hex.as_deref(),
        Some(signed_transaction),
        "the deprecated alias must still name the same complete transaction"
    );

    // The second NeoX signing lane is deliberately semantic: the caller sends
    // the fulfillment fields, not a digest or prehash. Custody binds the EIP-191
    // signature to this chain, exact oracle contract and fulfillRequest selector.
    let eip191 = expect_allowed(
        "POST /sign/eip191-fulfillment NeoX",
        client.sign_eip191_fulfillment(
            &CallerToken::bearer(&created.token),
            &Eip191FulfillmentRequest {
                key_id: neox_key.key_id.clone(),
                request_id: "neo-nexus-live-eip191-1".to_string(),
                chain_id: NEOX_CHAIN_ID,
                oracle_contract: NEOX_ORACLE_CONTRACT.to_string(),
                fulfillment: Eip191Fulfillment {
                    request_id: "7".to_string(),
                    app_id: "app:1".to_string(),
                    module_id: "oracle.fetch".to_string(),
                    operation: "privacy_oracle".to_string(),
                    success: true,
                    error: String::new(),
                },
                result_bytes_hex: "0x68656c6c6f".to_string(),
            },
        )?,
    )?;
    assert_eq!(eip191.chain_family, "neox");
    assert_eq!(eip191.chain_id, NEOX_CHAIN_ID);
    assert_eq!(eip191.oracle_contract, NEOX_ORACLE_CONTRACT);
    assert!(eip191.digest.starts_with("0x"), "{eip191:?}");
    assert!(eip191.message_hash.starts_with("0x"), "{eip191:?}");
    assert!(
        matches!(eip191.signature.get(130..), Some("1b" | "1c")),
        "{eip191:?}"
    );

    // 9. Bytes that are not a transaction or payload: refusals arriving as
    //    decisions with the service's own statuses rather than failed requests.
    let attempted =
        client.sign_transaction(&CallerToken::bearer(&created.token), &key.key_id, "00")?;
    let refusal = match attempted {
        Outcome::Allowed(witness) => bail!("the service signed junk bytes: {witness:?}"),
        Outcome::Refused(refusal) => refusal,
    };
    assert_eq!(refusal.code, "signer-transaction-unparsable", "{refusal:?}");
    assert_eq!(refusal.status, 400);
    let attempted =
        client.sign_consensus(&CallerToken::bearer(&created.token), &key.key_id, "00")?;
    let refusal = match attempted {
        Outcome::Allowed(witness) => bail!("the service signed a junk payload: {witness:?}"),
        Outcome::Refused(refusal) => refusal,
    };
    assert_eq!(refusal.code, "signer-payload-unparsable", "{refusal:?}");
    assert_eq!(refusal.status, 400);

    // 10. The caller is visible before the mutation surface rotates, disables,
    //     and removes it.
    let callers = expect_allowed("GET /callers", client.list_callers(&admin)?)?;
    assert!(
        callers.iter().any(|caller| caller.id == created.caller.id),
        "the created caller is missing from {} callers",
        callers.len()
    );
    assert!(
        callers.iter().any(|caller| caller.id == workload.caller.id),
        "the workload caller is missing from {} callers",
        callers.len()
    );
    assert!(
        callers
            .iter()
            .any(|caller| caller.id == raw_created.caller.id),
        "the raw caller is missing from {} callers",
        callers.len()
    );

    // 11. The mutation surface: rotate the credential, disable and remove both
    //    rows, so a rerun of this probe is not a pile of abandoned keys.
    let rotated = expect_allowed(
        "POST /callers/{id}/rotate",
        client.rotate_caller_token(&admin, &created.caller.id)?,
    )?;
    assert_eq!(rotated.caller_id, created.caller.id);
    assert_ne!(
        rotated.token, created.token,
        "a rotation must change the token"
    );
    let disabled = expect_allowed(
        "POST /callers/{id}/state",
        client.set_caller_disabled(&admin, &created.caller.id, true)?,
    )?;
    assert!(disabled.disabled);
    expect_allowed(
        "DELETE /callers/{id} workload",
        client.delete_caller(&admin, &workload.caller.id)?,
    )?;
    cleanup.workload_caller_id = None;
    expect_allowed(
        "DELETE /callers/{id}",
        client.delete_caller(&admin, &created.caller.id)?,
    )?;
    cleanup.caller_id = None;
    expect_allowed(
        "DELETE /callers/{id} raw",
        client.delete_caller(&admin, &raw_created.caller.id)?,
    )?;
    cleanup.raw_caller_id = None;
    let key_switched = expect_allowed(
        "POST /keys/{id}/state",
        client.set_key_disabled(&admin, &key.key_id, true)?,
    )?;
    assert!(!key_switched.signing_enabled);
    let key_restored = expect_allowed(
        "POST /keys/{id}/state enable",
        client.set_key_disabled(&admin, &key.key_id, false)?,
    )?;
    assert!(key_restored.signing_enabled);
    let key_switched = expect_allowed(
        "POST /keys/{id}/state disable again",
        client.set_key_disabled(&admin, &key.key_id, true)?,
    )?;
    assert!(!key_switched.signing_enabled);
    let neox_key_switched = expect_allowed(
        "POST /keys/{id}/state NeoX",
        client.set_key_disabled(&admin, &neox_key.key_id, true)?,
    )?;
    assert!(!neox_key_switched.signing_enabled);
    expect_allowed(
        "DELETE /keys/{id} NeoX",
        client.delete_key(&admin, &neox_key.key_id)?,
    )?;
    cleanup.neox_key_id = None;
    expect_allowed("DELETE /keys/{id}", client.delete_key(&admin, &key.key_id)?)?;
    cleanup.key_id = None;

    // 12. Every key step above left a row, and the filter finds them. Caller rows
    //    carry no `key_id` — the audit of a credential is about the credential —
    //    so they are deliberately absent from a filtered read.
    let rows = expect_allowed(
        "GET /audit?key_id",
        client.list_audit(&admin, Some(&key.key_id), Some(50))?,
    )?;
    let actions: Vec<&str> = rows.iter().map(|row| row.action.as_str()).collect();
    for expected in [
        "signer-key-generated",
        "signer-policy-saved",
        "signer-key-viewed",
        "signer-raw-signed",
        "signer-transaction-signed",
        "signer-consensus-signed",
        "signer-key-disabled",
        "signer-key-deleted",
    ] {
        assert!(
            actions.contains(&expected),
            "{expected} missing from {actions:?}"
        );
    }
    Ok(())
}

/// Best-effort cleanup for a live contract assertion that returns early or
/// panics. A failed probe must not fill a development vault with abandoned
/// keys and callers; successful explicit deletes clear these ids before Drop.
struct LiveCleanup {
    client: SignerClient,
    key_id: Option<String>,
    caller_id: Option<String>,
    raw_caller_id: Option<String>,
    workload_caller_id: Option<String>,
    neox_key_id: Option<String>,
}

impl LiveCleanup {
    fn new(client: &SignerClient, key_id: &str) -> Self {
        LiveCleanup {
            client: client.clone(),
            key_id: Some(key_id.to_string()),
            caller_id: None,
            raw_caller_id: None,
            workload_caller_id: None,
            neox_key_id: None,
        }
    }
}

impl Drop for LiveCleanup {
    fn drop(&mut self) {
        let Some(credentials) = self.client.config().admin() else {
            return;
        };
        if let Some(caller_id) = self.workload_caller_id.take() {
            let _ = self.client.delete_caller(&credentials, &caller_id);
        }
        if let Some(caller_id) = self.caller_id.take() {
            let _ = self.client.delete_caller(&credentials, &caller_id);
        }
        if let Some(caller_id) = self.raw_caller_id.take() {
            let _ = self.client.delete_caller(&credentials, &caller_id);
        }
        if let Some(key_id) = self.neox_key_id.take() {
            let _ = self.client.set_key_disabled(&credentials, &key_id, true);
            let _ = self.client.delete_key(&credentials, &key_id);
        }
        if let Some(key_id) = self.key_id.take() {
            let _ = self.client.delete_key(&credentials, &key_id);
        }
    }
}

/// Unwrap an allowed answer, or stop the probe with the refusal in full. Used
/// only by the live test, where a `no` from a route that should answer `yes` is a
/// failure rather than a branch.
fn expect_allowed<T>(route: &str, outcome: Outcome<T>) -> Result<T> {
    match outcome {
        Outcome::Allowed(payload) => Ok(payload),
        Outcome::Refused(refusal) => Err(anyhow::anyhow!("{route} refused: {}", refusal.summary())),
    }
}

/// One well-formed dBFT PrepareResponse extensible payload, built only from
/// public identity returned by the service. Neo N3 displays UInt160 values in
/// reverse of their wire order, so the sender is reversed before serialization.
fn live_consensus_payload(script_hash: &str) -> Result<String> {
    let display = script_hash.strip_prefix("0x").unwrap_or(script_hash);
    if display.len() != 40 {
        bail!("signer returned a non-UInt160 script hash: {script_hash}");
    }
    let mut sender = Vec::with_capacity(20);
    for pair in display.as_bytes().chunks_exact(2) {
        let high = hex_nibble(pair[0]).context("script hash contains non-hexadecimal data")?;
        let low = hex_nibble(pair[1]).context("script hash contains non-hexadecimal data")?;
        sender.push((high << 4) | low);
    }
    sender.reverse();

    const BLOCK_INDEX: u32 = 200;
    let data = [
        vec![0x21], // dBFT PrepareResponse
        BLOCK_INDEX.to_le_bytes().to_vec(),
        vec![0, 0], // validator index, view number
        vec![0x01; 32],
    ]
    .concat();
    let mut payload = Vec::with_capacity(1 + 4 + 4 + 4 + 20 + 1 + data.len());
    payload.push(4);
    payload.extend_from_slice(b"dBFT");
    payload.extend_from_slice(&0_u32.to_le_bytes());
    payload.extend_from_slice(&BLOCK_INDEX.to_le_bytes());
    payload.extend_from_slice(&sender);
    payload.push(u8::try_from(data.len()).context("dBFT fixture exceeded a one-byte length")?);
    payload.extend_from_slice(&data);

    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(payload.len() * 2);
    for byte in payload {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(encoded)
}

fn client(base_url: &str, admin_token: Option<&str>) -> SignerClient {
    SignerClient::new(
        SignerConfig::new_insecure_loopback_for_test(
            base_url,
            admin_token.map(str::to_string),
            Duration::from_secs(5),
        )
        .unwrap(),
    )
}

fn request_header<'a>(request: &'a str, wanted: &str) -> Result<&'a str> {
    request
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find_map(|(name, value)| name.eq_ignore_ascii_case(wanted).then_some(value.trim()))
        .with_context(|| format!("request omitted {wanted}"))
}

fn decode_hex_signature(encoded: &str) -> Result<[u8; 64]> {
    if encoded.len() != 128 {
        bail!("workload signature is not 64 bytes");
    }
    let mut decoded = [0_u8; 64];
    for (index, pair) in encoded.as_bytes().chunks_exact(2).enumerate() {
        let high = hex_nibble(pair[0]).context("signature contains non-hexadecimal data")?;
        let low = hex_nibble(pair[1]).context("signature contains non-hexadecimal data")?;
        decoded[index] = (high << 4) | low;
    }
    Ok(decoded)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn reply(status: u16, body: &str) -> (u16, String) {
    (status, body.to_string())
}

/// A loopback HTTP server that answers from `reply_for` and reports every request
/// it received over a channel — so an assertion can be about the bytes the service
/// would see rather than about this client's own opinion of them.
fn spawn_stub(
    reply_for: impl Fn(&str) -> (u16, String) + Send + 'static,
) -> (String, Receiver<String>) {
    spawn_stub_counting(Arc::new(AtomicUsize::new(0)), reply_for)
}

fn spawn_stub_counting(
    counter: Arc<AtomicUsize>,
    reply_for: impl Fn(&str) -> (u16, String) + Send + 'static,
) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a free loopback port");
    let address = listener.local_addr().expect("the bound address");
    let (sender, receiver) = channel();
    thread::spawn(move || {
        // Four accepts covers every test here, and a client that asks for a fifth
        // has already failed the assertion that cares.
        for _ in 0..4 {
            let Ok((mut stream, _peer)) = listener.accept() else {
                return;
            };
            let Ok(request) = read_request(&mut stream) else {
                return;
            };
            counter.fetch_add(1, Ordering::SeqCst);
            let (status, body) = reply_for(&request);
            if sender.send(request).is_err() {
                return;
            }
            let head = format!(
                "HTTP/1.1 {status} Answer\r\nContent-Type: application/json\r\nContent-Length: \
                 {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            if stream.write_all(head.as_bytes()).is_err()
                || stream.write_all(body.as_bytes()).is_err()
            {
                return;
            }
        }
    });
    (format!("http://{address}"), receiver)
}

/// The request as one string: the request line untouched, the header names
/// lowercased so an assertion is about presence rather than capitalization, and
/// the body verbatim.
fn read_request(stream: &mut std::net::TcpStream) -> Result<String> {
    let mut reader = BufReader::new(&mut *stream);
    let mut request = String::new();
    let mut content_length = 0_usize;
    let mut first_line = true;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if let Some(value) = line
            .split_once(':')
            .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        {
            content_length = value;
        }
        if first_line {
            first_line = false;
            request.push_str(&line);
        } else {
            request.push_str(&line.to_lowercase());
        }
        if line == "\r\n" {
            break;
        }
    }
    if content_length > 0 {
        let mut body = vec![0_u8; content_length];
        reader.read_exact(&mut body)?;
        request.push_str(&String::from_utf8_lossy(&body));
    }
    Ok(request)
}
