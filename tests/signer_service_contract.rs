//! Cross-repository contract probe against the real Rust signer process.
//!
//! Standalone NeoNexus CI does not clone `neo-os-services`, so this test is
//! ignored by default. The NeoOS workspace gate builds the sibling binary and
//! runs this target explicitly.

use std::{
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use ed25519_dalek::SigningKey;
use neo_nexus::signer_client::{
    AssetLimit, AuditFilter, BearerCredential, CreateCallerRequest, CreateWorkloadCallerRequest,
    GenerateKeyRequest, KeyGrant, RawSignRequest, SignRequest, SignatureRateLimit, SignerClient,
    SignerClientConfig, SignerCredential, SignerEndpoint, SignerOutcome, SignerPolicy, WindowLimit,
    WorkloadCredential,
};

const NEO_DISPLAY: &str = "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";

#[test]
#[ignore = "requires the sibling neo-os-services signer binary"]
fn real_rust_signer_and_neonexus_client_agree() {
    let binary = signer_binary();
    assert!(
        binary.is_file(),
        "build the signer first: cargo build -p neo-signer --bin signer_service ({})",
        binary.display()
    );
    let home = tempfile::tempdir().expect("signer workspace");
    let token = provision_admin(&binary, home.path());
    let port = unused_port();
    let _process = SignerProcess::start(&binary, home.path(), port);
    let endpoint = format!("http://127.0.0.1:{port}");
    let admin = bearer_client(&endpoint, &token);
    wait_until_healthy(&admin);

    let key = allowed(admin.generate_key(&GenerateKeyRequest {
        label: "integration-validator".to_string(),
        network: "testnet".to_string(),
        network_magic: None,
    }));
    assert_eq!(allowed(admin.list_keys()).len(), 1);
    let closed = allowed(admin.key_policy(&key.key_id));
    assert!(!closed.policy.allow_transfer);

    let policy = SignerPolicy {
        allow_consensus: true,
        allow_transfer: true,
        asset_whitelist: vec![format!("0x{NEO_DISPLAY}")],
        asset_limits: vec![AssetLimit {
            asset: format!("0x{NEO_DISPLAY}"),
            max_single_amount: Some("500".to_string()),
            window_limit: Some(WindowLimit {
                seconds: 3_600,
                max_amount: "2000".to_string(),
            }),
        }],
        max_single_amount: Some("1000".to_string()),
        max_signers: Some(1),
        max_system_fee: Some("100000000".to_string()),
        max_network_fee: Some("100000000".to_string()),
        max_signatures: Some(SignatureRateLimit {
            seconds: 3_600,
            count: 10,
        }),
        ..SignerPolicy::default()
    };
    let saved = allowed(admin.save_policy(&key.key_id, &policy));
    assert_eq!(saved.policy, policy);

    let created = allowed(admin.create_caller(&CreateCallerRequest {
        label: "integration-bearer".to_string(),
        key_grant: KeyGrant::only(vec![key.key_id.clone()]),
        capabilities: vec!["sign".to_string()],
        allowed_origins: Vec::new(),
    }));
    let signer = bearer_client(&endpoint, &created.token);
    assert_eq!(
        allowed(signer.key_info(&key.key_id)).public_key,
        key.public_key
    );

    let account = display_hash_to_wire(&key.script_hash);
    let transaction = transfer_transaction(account, 100, 7);
    let transaction_request = SignRequest {
        key_id: key.key_id.clone(),
        unsigned_hex: lower_hex(&transaction),
        request_id: Some("integration-transfer-1".to_string()),
    };
    let witness = allowed(signer.sign_transaction(&transaction_request));
    assert_eq!(witness.key_id, key.key_id);
    assert_eq!(
        allowed(signer.sign_transaction(&transaction_request)),
        witness,
        "an exact retry returns the stored witness"
    );
    let conflict = signer
        .sign_transaction(&SignRequest {
            key_id: key.key_id.clone(),
            unsigned_hex: lower_hex(&transfer_transaction(account, 100, 8)),
            request_id: transaction_request.request_id.clone(),
        })
        .expect("idempotency conflict");
    let conflict = match conflict {
        SignerOutcome::Refused(conflict) => Some(conflict),
        SignerOutcome::Allowed(_) => None,
    };
    assert!(conflict.is_some(), "changed bytes reused an idempotency id");
    let conflict = conflict.expect("typed idempotency conflict");
    assert_eq!(conflict.code, "request-id-conflict");
    assert_eq!(conflict.status, 409);

    let consensus = consensus_payload(account, 200);
    let consensus_witness = allowed(signer.sign_consensus(&SignRequest {
        key_id: key.key_id.clone(),
        unsigned_hex: lower_hex(&consensus),
        request_id: Some("integration-consensus-1".to_string()),
    }));
    assert_eq!(consensus_witness.key_id, key.key_id);

    let raw_key = allowed(admin.generate_key(&GenerateKeyRequest {
        label: "integration-raw-role".to_string(),
        network: "testnet".to_string(),
        network_magic: None,
    }));
    allowed(admin.save_policy(
        &raw_key.key_id,
        &SignerPolicy {
            allow_raw: true,
            ..SignerPolicy::default()
        },
    ));
    let raw_created = allowed(admin.create_caller(&CreateCallerRequest {
        label: "integration-raw-caller".to_string(),
        key_grant: KeyGrant::only(vec![raw_key.key_id.clone()]),
        capabilities: vec!["raw_sign".to_string()],
        allowed_origins: Vec::new(),
    }));
    let raw_signer = bearer_client(&endpoint, &raw_created.token);
    let raw = allowed(raw_signer.sign_raw(&RawSignRequest {
        key_id: raw_key.key_id.clone(),
        data_hex: "010203".to_string(),
        request_id: Some("integration-raw-1".to_string()),
    }));
    assert_eq!(raw.signature.len(), 128);

    let workload_key = SigningKey::from_bytes(&[17u8; 32]);
    let workload = allowed(admin.create_workload_caller(&CreateWorkloadCallerRequest {
        caller: CreateCallerRequest {
            label: "integration-workload".to_string(),
            key_grant: KeyGrant::only(vec![key.key_id.clone()]),
            capabilities: vec!["sign".to_string()],
            allowed_origins: Vec::new(),
        },
        workload_public_key: lower_hex(&workload_key.verifying_key().to_bytes()),
        workload_subject: Some("neo-nexus:contract-test".to_string()),
    }));
    let workload_client = SignerClient::new(SignerClientConfig {
        endpoint: SignerEndpoint::parse(&endpoint).expect("endpoint"),
        credential: SignerCredential::Workload(Box::new(
            WorkloadCredential::from_seed(
                workload.caller.id.clone(),
                Some("neo-nexus:contract-test".to_string()),
                workload_key.to_bytes(),
            )
            .expect("workload credential"),
        )),
    });
    assert_eq!(
        allowed(workload_client.key_info(&key.key_id)).key_id,
        key.key_id
    );

    let disabled = allowed(admin.set_key_disabled(&raw_key.key_id, true));
    assert!(!disabled.signing_enabled);
    let refusal = raw_signer
        .sign_raw(&RawSignRequest {
            key_id: raw_key.key_id.clone(),
            data_hex: "04".to_string(),
            request_id: None,
        })
        .expect("disabled-key refusal");
    let refusal = match refusal {
        SignerOutcome::Refused(refusal) => Some(refusal),
        SignerOutcome::Allowed(_) => None,
    };
    assert!(refusal.is_some(), "disabled key produced a signature");
    let refusal = refusal.expect("typed disabled-key refusal");
    assert_eq!(refusal.code, "signer-key-disabled");
    assert!(allowed(admin.set_key_disabled(&raw_key.key_id, false)).signing_enabled);

    let rotated = allowed(admin.rotate_caller(&created.caller.id));
    let rotated_signer = bearer_client(&endpoint, &rotated.token);
    assert_eq!(
        allowed(rotated_signer.key_info(&key.key_id)).key_id,
        key.key_id
    );
    assert!(matches!(
        signer.key_info(&key.key_id).expect("stale-token refusal"),
        SignerOutcome::Refused(_)
    ));

    assert!(allowed(admin.list_callers()).len() >= 3);
    let audit = allowed(admin.audit(&AuditFilter {
        key_id: None,
        limit: Some(500),
    }));
    assert!(audit
        .iter()
        .any(|entry| entry.action == "signer-transaction-signed"));
    assert!(audit
        .iter()
        .any(|entry| entry.action == "signer-consensus-signed"));
    assert!(audit
        .iter()
        .any(|entry| entry.action == "signer-raw-signed"));

    allowed(admin.delete_caller(&created.caller.id));
    allowed(admin.delete_caller(&raw_created.caller.id));
    allowed(admin.delete_caller(&workload.caller.id));
    allowed(admin.delete_key(&key.key_id));
    allowed(admin.delete_key(&raw_key.key_id));
    assert!(allowed(admin.list_keys()).is_empty());
}

fn signer_binary() -> PathBuf {
    let executable = if cfg!(windows) {
        "signer_service.exe"
    } else {
        "signer_service"
    };
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("NeoOS workspace root")
        .join("neo-os-services")
        .join("target")
        .join("debug")
        .join(executable)
}

fn unused_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("ephemeral port")
        .local_addr()
        .expect("ephemeral address")
        .port()
}

fn provision_admin(binary: &Path, data_dir: &Path) -> String {
    let output = Command::new(binary)
        .arg("provision-admin")
        .arg("neo-nexus-contract")
        .env("SIGNER_SERVICE_DATA_DIR", data_dir)
        .env_remove("SIGNER_SERVICE_ATTESTED_BOOT")
        .env_remove("SIGNER_SERVICE_MASTER_KEY")
        .output()
        .expect("provision signer admin");
    assert!(
        output.status.success(),
        "admin provisioning failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let token = String::from_utf8(output.stdout).expect("admin token is UTF-8");
    let token = token.trim().to_string();
    assert!(token.starts_with("nsk1_"));
    token
}

struct SignerProcess {
    child: Child,
}

impl SignerProcess {
    fn start(binary: &Path, data_dir: &Path, port: u16) -> Self {
        let child = Command::new(binary)
            .arg("serve")
            .env("SIGNER_SERVICE_DATA_DIR", data_dir)
            .env("SIGNER_SERVICE_HOST", "127.0.0.1")
            .env("SIGNER_SERVICE_PORT", port.to_string())
            .env_remove("SIGNER_SERVICE_ATTESTED_BOOT")
            .env_remove("SIGNER_SERVICE_MASTER_KEY")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start signer service");
        Self { child }
    }
}

impl Drop for SignerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn bearer_client(endpoint: &str, token: &str) -> SignerClient {
    SignerClient::new(SignerClientConfig {
        endpoint: SignerEndpoint::parse(endpoint).expect("signer endpoint"),
        credential: SignerCredential::Bearer(
            BearerCredential::new(token.to_string()).expect("bearer token"),
        ),
    })
}

fn wait_until_healthy(client: &SignerClient) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if client.health().is_ok_and(|health| health.status == "ok") {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        client.health().is_ok_and(|health| health.status == "ok"),
        "real signer did not become healthy"
    );
}

fn allowed<T>(result: Result<SignerOutcome<T>, neo_nexus::signer_client::SignerClientError>) -> T {
    let (value, refusal) = match result.expect("signer client transport") {
        SignerOutcome::Allowed(value) => (Some(value), None),
        SignerOutcome::Refused(refusal) => (None, Some(refusal)),
    };
    let refusal_message = refusal
        .as_ref()
        .map(|refusal| format!("{}: {}", refusal.code, refusal.message))
        .unwrap_or_default();
    assert!(
        value.is_some(),
        "signer request was refused: {refusal_message}"
    );
    value.expect("signer request should be allowed")
}

fn display_hash_to_wire(value: &str) -> [u8; 20] {
    let value = value.strip_prefix("0x").unwrap_or(value);
    let mut bytes = decode_hex::<20>(value);
    bytes.reverse();
    bytes
}

fn transfer_transaction(account: [u8; 20], amount: u8, nonce: u32) -> Vec<u8> {
    let asset = display_hash_to_wire(NEO_DISPLAY);
    let recipient = [0x22u8; 20];
    let mut script = vec![0x0b, 0x00, amount];
    script.extend(push_bytes(&recipient));
    script.extend(push_bytes(&account));
    script.extend([0x14, 0xc0, 0x1f]);
    script.extend(push_bytes(b"transfer"));
    script.extend(push_bytes(&asset));
    script.push(0x41);
    script.extend([0x62, 0x7d, 0x5b, 0x52]);

    let mut unsigned = vec![0u8];
    unsigned.extend(nonce.to_le_bytes());
    unsigned.extend(0i64.to_le_bytes());
    unsigned.extend(0i64.to_le_bytes());
    unsigned.extend(0u32.to_le_bytes());
    unsigned.push(1);
    unsigned.extend(account);
    unsigned.push(0x01);
    unsigned.push(0);
    unsigned.push(script.len() as u8);
    unsigned.extend(script);
    unsigned
}

fn consensus_payload(sender: [u8; 20], height: u32) -> Vec<u8> {
    let mut payload = vec![4];
    payload.extend(b"dBFT");
    payload.extend(0u32.to_le_bytes());
    payload.extend(height.to_le_bytes());
    payload.extend(sender);
    let mut data = vec![0x21];
    data.extend(height.to_le_bytes());
    data.extend([0, 0, 1]);
    payload.push(data.len() as u8);
    payload.extend(data);
    payload
}

fn push_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut pushed = vec![0x0c, bytes.len() as u8];
    pushed.extend(bytes);
    pushed
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_hex<const N: usize>(value: &str) -> [u8; N] {
    assert_eq!(value.len(), N * 2);
    let mut decoded = [0u8; N];
    for (index, byte) in decoded.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).expect("hex");
    }
    decoded
}
