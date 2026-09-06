use super::*;

use std::{fs, time::Duration};

use crate::signer_client::{
    Eip191Fulfillment, Eip191FulfillmentRequest, Outcome, SignerClient, SignerConfig,
};

#[test]
fn backend_kinds_are_explicit_and_have_stable_labels() {
    let kinds = SignerBackendKind::ALL;
    assert_eq!(kinds.len(), 3);
    assert_eq!(SignerBackendKind::LocalWallet.slug(), "local-wallet");
    assert_eq!(SignerBackendKind::LocalSigner.slug(), "local-signer");
    assert_eq!(SignerBackendKind::NeoOsService.slug(), "neo-os-service");
    assert!(!SignerBackendKind::LocalWallet.has_http_custody_api());
    assert!(!SignerBackendKind::LocalSigner.has_http_custody_api());
    assert!(SignerBackendKind::NeoOsService.has_http_custody_api());
}

#[test]
fn local_wallet_capabilities_do_not_claim_remote_controls() {
    let capabilities = SignerCapabilities::local_wallet(true, false, false);
    assert!(capabilities.neo_n3_transaction);
    assert!(!capabilities.neo_n3_consensus);
    assert!(!capabilities.neo_n3_raw);
    assert!(!capabilities.neox_transaction);
    assert!(!capabilities.key_administration);
    assert!(!capabilities.policy_administration);
    assert!(!capabilities.caller_administration);
    assert!(!capabilities.durable_audit);
    assert!(!capabilities.public_relay);
}

#[test]
fn key_references_include_the_owning_backend() {
    let local = SignerKeyRef::new("wallet", "same-key").unwrap();
    let remote = SignerKeyRef::new("neo-os", "same-key").unwrap();
    assert_ne!(local, remote);
    assert!(SignerKeyRef::new("../escape", "key").is_err());
    assert!(SignerKeyRef::new("backend", "key/escape").is_err());
}

#[test]
fn local_wallet_environment_is_complete_and_fail_closed() {
    let incomplete = LocalWalletEnvironment {
        wallet_path: Some("wallet.json".to_string()),
        ..LocalWalletEnvironment::default()
    };
    assert!(LocalWalletConfig::resolve_environment(incomplete).is_err());

    let configured = LocalWalletConfig::resolve_environment(LocalWalletEnvironment {
        wallet_path: Some("wallet.json".to_string()),
        password_file: Some("wallet.password".to_string()),
        network: Some("private".to_string()),
        network_magic: Some("1230404".to_string()),
        ..LocalWalletEnvironment::default()
    })
    .unwrap()
    .expect("all local wallet settings are present");
    assert!(configured.allow_transaction);
    assert!(!configured.allow_consensus);
    assert!(!configured.allow_raw);
}

#[test]
fn registry_holds_all_three_backend_families_without_key_collisions() {
    let home = tempfile::tempdir().expect("temporary signer registry");
    let wallet_path = home.path().join("wallet.json");
    let password_path = home.path().join("wallet.password");
    let address = "NZAvnENCGsGZAN2GssPNftitKaKPfFwd5v";
    fs::write(
        &wallet_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "name": "registry fixture",
            "version": "3.0",
            "scrypt": { "n": 16384, "r": 8, "p": 8 },
            "accounts": [{
                "address": address,
                "label": "local",
                "isDefault": true,
                "lock": false,
                "key": "6PYRs1PSxgTKGgoYnCfpMkb93WDTjmPgsneJgRWDxQL8D1RWjS6mAoMUxd",
                "contract": {
                    "script": "0c21031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e4156e7b327",
                    "parameters": [{ "name": "signature", "type": "Signature" }],
                    "deployed": false
                },
                "extra": null
            }],
            "extra": null
        }))
        .expect("wallet JSON"),
    )
    .expect("wallet file");
    fs::write(&password_path, "correct horse battery staple\n").expect("password file");
    restrict_secret(&password_path);

    let wallet = LocalWalletSigner::open(LocalWalletConfig {
        wallet_path,
        password_file: password_path,
        account: Some(address.to_string()),
        network: "private".to_string(),
        network_magic: 1_230_404,
        allow_transaction: true,
        allow_consensus: false,
        allow_raw: false,
    })
    .expect("local wallet opens");
    let local_key_id = wallet.key_info().key_id;
    let wallet = ConfiguredSignerBackend::local_wallet(
        SignerBackendProfile::new("wallet", "Local wallet", SignerBackendKind::LocalWallet)
            .unwrap(),
        wallet,
    )
    .unwrap();

    let local_service = ConfiguredSignerBackend::local_signer(
        SignerBackendProfile::new("host-signer", "Host signer", SignerBackendKind::LocalSigner)
            .unwrap(),
        LocalSignerConfig::new(
            "http://127.0.0.1:9991",
            "031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e",
            1_230_404,
        )
        .unwrap(),
    )
    .unwrap();
    let neo_os = ConfiguredSignerBackend::neo_os_service(
        SignerBackendProfile::new("neo-os", "NeoOS signer", SignerBackendKind::NeoOsService)
            .unwrap(),
        ServiceSignerBackend::new(
            SignerClient::new(
                SignerConfig::new("https://signer.example", None, Duration::from_secs(1)).unwrap(),
            ),
            None,
        ),
    )
    .unwrap();

    let registry = SignerRegistry::new(
        [wallet, local_service, neo_os],
        Some("neo-os".to_string()),
        Some("neo-os".to_string()),
    )
    .expect("three backends coexist");
    assert_eq!(registry.profiles().count(), 3);
    assert_eq!(registry.console_backend_id(), Some("neo-os"));
    assert_eq!(registry.relay_backend_id(), Some("neo-os"));

    let key = SignerKeyRef::new("wallet", local_key_id).unwrap();
    assert!(matches!(
        registry.key_info(&key).unwrap(),
        Outcome::Allowed(_)
    ));
    assert!(registry
        .sign_eip191_fulfillment(
            &key,
            &Eip191FulfillmentRequest {
                key_id: key.key_id.clone(),
                request_id: "request-1".to_string(),
                chain_id: 47_763,
                oracle_contract: "0x1111111111111111111111111111111111111111".to_string(),
                fulfillment: Eip191Fulfillment {
                    request_id: "oracle-1".to_string(),
                    app_id: "app".to_string(),
                    module_id: "module".to_string(),
                    operation: "resolve".to_string(),
                    success: true,
                    error: String::new(),
                },
                result_bytes_hex: "00".to_string(),
            }
        )
        .is_err());
    assert!(registry
        .key_info(&SignerKeyRef::new("neo-os", key.key_id.clone()).unwrap())
        .is_err());
}

#[test]
fn registry_file_routes_control_plane_and_keeps_node_signing_identity_available() {
    let home = tempfile::tempdir().expect("temporary registry document");
    let path = home.path().join("neo-os-admin.token");
    fs::write(&path, format!("{}\n", "c".repeat(48))).expect("credential file");
    restrict_secret(&path);
    let registry_path = home.path().join("signers.toml");
    fs::write(
        &registry_path,
        r#"version = 1
console_backend = "neo-os"
relay_backend = "neo-os"

[[backends]]
id = "host-signer"
label = "Host signer"
kind = "local-signer"
endpoint = "http://127.0.0.1:9991"
public_key = "031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e"
network_magic = 860833102

[[backends]]
id = "neo-os"
label = "NeoOS signer"
kind = "neo-os-service"
url = "https://signer.example"
admin_token_file = "neo-os-admin.token"
"#,
    )
    .expect("registry file");

    let registry = SignerRegistry::from_file(&registry_path).expect("registry opens");
    assert_eq!(registry.profiles().count(), 2);
    assert_eq!(registry.console_backend_id(), Some("neo-os"));
    assert_eq!(registry.relay_backend_id(), Some("neo-os"));
    assert!(registry
        .backend("host-signer")
        .unwrap()
        .local_signer_config()
        .is_some());
}

#[test]
fn service_backend_rejects_a_signing_transport_outside_its_trust_boundary() {
    let client = |url: &str| {
        SignerClient::new(SignerConfig::new(url, None, Duration::from_secs(1)).unwrap())
    };

    let remote =
        SignerBackendProfile::new("neo-os", "NeoOS signer", SignerBackendKind::NeoOsService)
            .unwrap();
    assert!(ConfiguredSignerBackend::neo_os_service(
        remote,
        ServiceSignerBackend::new(
            client("https://signer.example"),
            Some(client("https://other.example")),
        ),
    )
    .is_err());
}

#[test]
fn registry_file_rejects_aliased_credentials_and_reused_workload_callers() {
    let home = tempfile::tempdir().expect("temporary registry document");
    let shared = home.path().join("shared.token");
    fs::write(&shared, format!("{}\n", "a".repeat(48))).expect("shared credential");
    restrict_secret(&shared);
    let aliased = home.path().join("aliased.toml");
    fs::write(
        &aliased,
        r#"version = 1

[[backends]]
id = "service"
label = "Service"
kind = "neo-os-service"
url = "https://signer.example"
admin_token_file = "shared.token"
signing_token_file = "./shared.token"
"#,
    )
    .expect("aliased registry");
    let error = SignerRegistry::from_file(&aliased)
        .expect_err("one credential file cannot represent two identities");
    assert!(format!("{error:#}").contains("different credential files"));

    for (name, digit) in [("admin.seed", "1"), ("signing.seed", "2")] {
        let path = home.path().join(name);
        fs::write(&path, format!("{}\n", digit.repeat(64))).expect("workload seed");
        restrict_secret(&path);
    }
    let repeated = home.path().join("repeated-caller.toml");
    fs::write(
        &repeated,
        r#"version = 1

[[backends]]
id = "service"
label = "Service"
kind = "neo-os-service"
url = "https://signer.example"
admin_caller_id = "neo-nexus"
admin_workload_key_file = "admin.seed"
signing_caller_id = "neo-nexus"
signing_workload_key_file = "signing.seed"
"#,
    )
    .expect("reused caller registry");
    let error = SignerRegistry::from_file(&repeated)
        .expect_err("admin and signing workload identities must differ");
    assert!(format!("{error:#}").contains("different callers"));
}

fn restrict_secret(path: &std::path::Path) {
    crate::secret_file::protect_for_test(path).expect("secret permissions");
}
