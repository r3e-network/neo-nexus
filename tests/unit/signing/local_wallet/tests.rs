use std::{fs, path::Path};

use aes::cipher::{BlockEncrypt, KeyInit};
use p256::ecdsa::{signature::Verifier as _, Signature as P256Signature, VerifyingKey};
use sha2::{Digest, Sha256};

use super::{crypto, LocalWalletConfig, LocalWalletSigner};
use crate::{
    signer_client::{RawSignRequest, SignRequest},
    wallet::crypto::{base58check_encode, double_sha256},
};

const PASSWORD: &str = "correct horse battery staple";
const MAGIC: u32 = 1_230_404;
const FIXED_ADDRESS: &str = "NZAvnENCGsGZAN2GssPNftitKaKPfFwd5v";
const FIXED_NEP2: &str = "6PYRs1PSxgTKGgoYnCfpMkb93WDTjmPgsneJgRWDxQL8D1RWjS6mAoMUxd";
const FIXED_PUBLIC_KEY: &str = "031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e";
const FIXED_VERIFICATION_SCRIPT: &str =
    "0c21031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e4156e7b327";

#[test]
fn encrypted_wallet_signing_is_pinned_and_self_consistent() {
    let fixture = wallet_fixture(PASSWORD);
    let signer = LocalWalletSigner::open(fixture.config()).expect("wallet opens");
    let key = signer.key_info();
    assert_eq!(key.address, fixture.address);
    assert_eq!(key.network_magic, Some(MAGIC));
    assert_eq!(key.chain_family.as_deref(), Some("neo-n3"));

    let unsigned = minimal_unsigned_transaction(&key.script_hash);
    let signed = signer
        .sign_transaction(&SignRequest::neo_n3(
            &key.key_id,
            crypto::hex_encode(&unsigned),
        ))
        .expect("transaction signs");
    let invocation = hex_decode(signed.invocation_script.as_deref().expect("invocation"));
    assert_eq!(&invocation[..2], &[0x0c, 0x40]);
    let signature = P256Signature::from_slice(&invocation[2..]).expect("P-256 signature");
    let public_key = hex_decode(&key.public_key);
    let verifying_key = VerifyingKey::from_sec1_bytes(&public_key).expect("public key");
    let mut preimage = Vec::new();
    preimage.extend_from_slice(&MAGIC.to_le_bytes());
    preimage.extend_from_slice(&Sha256::digest(&unsigned));
    verifying_key
        .verify(&preimage, &signature)
        .expect("signature verifies over the Neo N3 sign-data");
    assert!(signed.additional_fields.contains_key("wallet_sha256"));

    let mut first = SignRequest::neo_n3(&key.key_id, crypto::hex_encode(&unsigned));
    first.request_id = Some("operation-1".to_string());
    signer
        .sign_transaction(&first)
        .expect("first request id signs");
    let mut different = unsigned.clone();
    different[1] ^= 1;
    let mut conflict = SignRequest::neo_n3(&key.key_id, crypto::hex_encode(&different));
    conflict.request_id = first.request_id.clone();
    assert!(signer
        .sign_transaction(&conflict)
        .expect_err("request id cannot change payload")
        .to_string()
        .contains("already used"));

    fs::write(&fixture.wallet_path, b"{}").expect("replace wallet");
    let error = signer
        .sign_transaction(&SignRequest::neo_n3(
            &key.key_id,
            crypto::hex_encode(&unsigned),
        ))
        .expect_err("a replaced wallet must fail closed");
    assert!(error.to_string().contains("changed after startup"));
}

#[test]
fn opens_an_independent_neo_n3_nep2_vector() {
    // Generated outside this crate from the Neo N3 reference algorithm:
    // P-256 key 07..07, canonical scrypt, AES-256-ECB, address version 0x35.
    // Keeping it fixed prevents the decryptor and test encryptor from sharing
    // the same incorrect block mode unnoticed.
    let fixture = wallet_fixture_from_vector(PASSWORD, FIXED_NEP2);
    let signer = LocalWalletSigner::open(fixture.config()).expect("reference NEP-2 opens");
    let key = signer.key_info();
    assert_eq!(key.address, FIXED_ADDRESS);
    assert_eq!(key.public_key, FIXED_PUBLIC_KEY);
    assert_eq!(key.verification_script, FIXED_VERIFICATION_SCRIPT);
}

#[test]
fn opens_a_bounded_noncanonical_nep6_scrypt_profile() {
    let fixture = wallet_fixture_with_scrypt(PASSWORD, 32_768, 4, 8);
    let signer = LocalWalletSigner::open(fixture.config()).expect("bounded profile opens");
    assert_eq!(signer.key_info().address, fixture.address);
}

#[test]
fn unlocked_key_survives_password_file_removal_and_reports_disabled_lanes() {
    let fixture = wallet_fixture(PASSWORD);
    let mut config = fixture.config();
    config.allow_transaction = false;
    let signer = LocalWalletSigner::open(config).expect("wallet opens");
    fs::remove_file(&fixture.password_path).expect("remove startup-only password file");
    assert!(!signer.key_info().signing_enabled);
    assert!(signer
        .sign_transaction(&SignRequest::neo_n3(&signer.key_info().key_id, "0001",))
        .is_err());
}

#[test]
fn native_node_wallet_material_is_rechecked_and_debug_redacted() {
    let fixture = wallet_fixture(PASSWORD);
    let signer = LocalWalletSigner::open(fixture.config()).expect("wallet opens");
    let runtime = signer
        .native_service_wallet()
        .expect("native wallet material is available");
    assert_eq!(runtime.path, fixture.wallet_path.to_string_lossy());
    assert_eq!(runtime.password.as_deref(), Some(PASSWORD));
    assert!(!format!("{runtime:?}").contains(PASSWORD));

    fs::write(&fixture.wallet_path, b"{}").expect("replace wallet");
    assert!(signer.native_service_wallet().is_err());
}

#[test]
fn wrong_password_and_wrong_account_are_rejected_at_startup() {
    let fixture = wallet_fixture(PASSWORD);
    fs::write(&fixture.password_path, "wrong password\n").expect("replace password");
    restrict(&fixture.password_path);
    assert!(LocalWalletSigner::open(fixture.config()).is_err());

    let fixture = wallet_fixture(PASSWORD);
    let mut config = fixture.config();
    config.account = Some("Nunknown".to_string());
    assert!(LocalWalletSigner::open(config).is_err());
}

#[test]
fn risky_lanes_are_closed_unless_explicitly_enabled() {
    let fixture = wallet_fixture(PASSWORD);
    let signer = LocalWalletSigner::open(fixture.config()).expect("wallet opens");
    let key = signer.key_info();
    assert!(signer
        .sign_consensus(&SignRequest::neo_n3(&key.key_id, "0001"))
        .is_err());
    assert!(signer
        .sign_raw(&RawSignRequest::new(&key.key_id, "0001"))
        .is_err());
}

#[test]
fn raw_digest_matches_the_service_contract_and_unicode_hex_is_rejected() {
    let fixture = wallet_fixture(PASSWORD);
    let mut config = fixture.config();
    config.allow_raw = true;
    let signer = LocalWalletSigner::open(config).expect("wallet opens");
    let key = signer.key_info();
    let raw = signer
        .sign_raw(&RawSignRequest::new(&key.key_id, "0001"))
        .expect("raw signing is enabled");
    assert_eq!(
        raw.digest,
        format!("0x{}", crypto::hex_encode(&Sha256::digest([0_u8, 1])))
    );

    assert!(signer
        .sign_raw(&RawSignRequest::new(&key.key_id, "€0"))
        .is_err());
    assert!(signer
        .sign_transaction(&SignRequest::neo_n3(&key.key_id, "€0"))
        .is_err());
}

struct Fixture {
    _home: tempfile::TempDir,
    wallet_path: std::path::PathBuf,
    password_path: std::path::PathBuf,
    address: String,
}

impl Fixture {
    fn config(&self) -> LocalWalletConfig {
        LocalWalletConfig {
            wallet_path: self.wallet_path.clone(),
            password_file: self.password_path.clone(),
            account: Some(self.address.clone()),
            network: "private".to_string(),
            network_magic: MAGIC,
            allow_transaction: true,
            allow_consensus: false,
            allow_raw: false,
        }
    }
}

fn wallet_fixture(password: &str) -> Fixture {
    wallet_fixture_with_scrypt(password, 16_384, 8, 8)
}

fn wallet_fixture_with_scrypt(password: &str, n: u64, r: u32, p: u32) -> Fixture {
    let home = tempfile::tempdir().expect("temporary wallet directory");
    let wallet_path = home.path().join("wallet.json");
    let password_path = home.path().join("wallet.password");
    let private_key = [7_u8; 32];
    let identity = crypto::identity_from_private(&private_key, "private", MAGIC, "fixture")
        .expect("derive identity");
    let encrypted = encrypt_nep2(&private_key, password, &identity.address, n, r, p);
    let document = serde_json::json!({
        "name": "local signer fixture",
        "version": "3.0",
        "scrypt": { "n": n, "r": r, "p": p },
        "accounts": [{
            "address": identity.address,
            "label": "signer",
            "isDefault": true,
            "lock": false,
            "key": encrypted,
            "contract": {
                "script": identity.verification_script,
                "parameters": [{ "name": "signature", "type": "Signature" }],
                "deployed": false
            },
            "extra": null
        }],
        "extra": null
    });
    fs::write(
        &wallet_path,
        serde_json::to_vec_pretty(&document).expect("wallet JSON"),
    )
    .expect("write wallet");
    fs::write(&password_path, format!("{password}\n")).expect("write password");
    restrict(&password_path);
    Fixture {
        _home: home,
        wallet_path,
        password_path,
        address: identity.address,
    }
}

fn wallet_fixture_from_vector(password: &str, encrypted: &str) -> Fixture {
    let home = tempfile::tempdir().expect("temporary wallet directory");
    let wallet_path = home.path().join("wallet.json");
    let password_path = home.path().join("wallet.password");
    let document = serde_json::json!({
        "name": "Neo N3 fixed NEP-2 vector",
        "version": "3.0",
        "scrypt": { "n": 16384, "r": 8, "p": 8 },
        "accounts": [{
            "address": FIXED_ADDRESS,
            "label": "signer",
            "isDefault": true,
            "lock": false,
            "key": encrypted,
            "contract": {
                "script": FIXED_VERIFICATION_SCRIPT,
                "parameters": [{ "name": "signature", "type": "Signature" }],
                "deployed": false
            },
            "extra": null
        }],
        "extra": null
    });
    fs::write(
        &wallet_path,
        serde_json::to_vec_pretty(&document).expect("wallet JSON"),
    )
    .expect("write wallet");
    fs::write(&password_path, format!("{password}\n")).expect("write password");
    restrict(&password_path);
    Fixture {
        _home: home,
        wallet_path,
        password_path,
        address: FIXED_ADDRESS.to_string(),
    }
}

fn encrypt_nep2(
    private_key: &[u8; 32],
    password: &str,
    address: &str,
    n: u64,
    r: u32,
    p: u32,
) -> String {
    let address_digest = double_sha256(address.as_bytes());
    let address_hash = &address_digest[..4];
    let log_n = u8::try_from(n.trailing_zeros()).expect("scrypt work factor");
    let params = scrypt::Params::new(log_n, r, p, 64).expect("scrypt parameters");
    let mut derived = [0_u8; 64];
    scrypt::scrypt(password.as_bytes(), address_hash, &params, &mut derived)
        .expect("derive NEP-2 key");
    let mut block = [0_u8; 32];
    for index in 0..32 {
        block[index] = private_key[index] ^ derived[index];
    }
    let cipher = aes::Aes256::new_from_slice(&derived[32..]).expect("AES parameters");
    for chunk in block.as_chunks_mut::<16>().0 {
        cipher.encrypt_block(chunk.into());
    }
    let mut payload = vec![0x01, 0x42, 0xe0];
    payload.extend_from_slice(address_hash);
    payload.extend_from_slice(&block);
    base58check_encode(&payload)
}

fn hex_decode(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).expect("hex byte"))
        .collect()
}

fn minimal_unsigned_transaction(display_script_hash: &str) -> Vec<u8> {
    let display = display_script_hash
        .strip_prefix("0x")
        .expect("display script hash prefix");
    let mut account = hex_decode(display);
    account.reverse();
    let mut bytes = vec![0];
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.extend_from_slice(&0_i64.to_le_bytes());
    bytes.extend_from_slice(&0_i64.to_le_bytes());
    bytes.extend_from_slice(&100_u32.to_le_bytes());
    bytes.push(1);
    bytes.extend_from_slice(&account);
    bytes.push(0x01);
    bytes.push(0);
    bytes.push(1);
    bytes.push(0x10);
    bytes
}

fn restrict(path: &Path) {
    crate::secret_file::protect_for_test(path).expect("protect credential file");
}
