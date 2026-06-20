use super::*;

#[test]
fn runtime_release_catalog_loads_signed_local_catalog() {
    let temp_dir = tempfile::tempdir().unwrap();
    let platform = RuntimePlatform::current();
    let catalog_text = serde_json::json!({
        "schema_version": 1,
        "generated_at_unix": 1_800_000_001u64,
        "releases": [
            {
                "id": "neo-rs-signed-catalog",
                "label": "neo-rs signed catalog",
                "node_type": "neo-rs",
                "version": "v1.0.0",
                "platform": { "os": platform.os, "arch": platform.arch },
                "url": "https://downloads.example.com/neo-node",
                "file_name": "neo-node",
                "executable_name": "neo-node",
                "expected_sha256": "e".repeat(64)
            }
        ]
    })
    .to_string();
    let catalog_path = temp_dir.path().join("runtime-catalog.json");
    let signature_path = temp_dir.path().join("runtime-catalog.json.sig");
    let signing_key = SigningKey::from_bytes(&[11u8; 32]);
    let signature = signing_key.sign(catalog_text.as_bytes());
    std::fs::write(&catalog_path, catalog_text).unwrap();
    std::fs::write(
        &signature_path,
        BASE64_STANDARD.encode(signature.to_bytes()),
    )
    .unwrap();

    let load = RuntimePackageManager::load_release_catalog(&RuntimeCatalogLoadRequest {
        source: catalog_path.display().to_string(),
        signature_source: Some(signature_path.display().to_string()),
        ed25519_public_key: Some(BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes())),
        max_bytes: RuntimePackageManager::DEFAULT_CATALOG_MAX_BYTES,
    })
    .unwrap();

    assert_eq!(load.signature_verified, Some(true));
    assert_eq!(load.catalog.releases.len(), 1);
    assert_eq!(load.catalog.releases[0].id, "neo-rs-signed-catalog");
    assert!(load.bytes > 0);
}

#[test]
fn runtime_release_catalog_rejects_tampered_signature() {
    let temp_dir = tempfile::tempdir().unwrap();
    let platform = RuntimePlatform::current();
    let original_catalog = serde_json::json!({
        "schema_version": 1,
        "releases": [
            {
                "id": "neo-rs-original",
                "label": "neo-rs original",
                "node_type": "neo-rs",
                "version": "v1.0.0",
                "platform": { "os": platform.os, "arch": platform.arch },
                "url": "https://downloads.example.com/neo-node",
                "file_name": "neo-node",
                "executable_name": "neo-node",
                "expected_sha256": "f".repeat(64)
            }
        ]
    })
    .to_string();
    let tampered_catalog = original_catalog.replace("neo-rs-original", "neo-rs-tampered");
    let catalog_path = temp_dir.path().join("runtime-catalog.json");
    let signature_path = temp_dir.path().join("runtime-catalog.json.sig");
    let signing_key = SigningKey::from_bytes(&[13u8; 32]);
    let signature = signing_key.sign(original_catalog.as_bytes());
    std::fs::write(&catalog_path, tampered_catalog).unwrap();
    std::fs::write(
        &signature_path,
        BASE64_STANDARD.encode(signature.to_bytes()),
    )
    .unwrap();

    let error = RuntimePackageManager::load_release_catalog(&RuntimeCatalogLoadRequest {
        source: catalog_path.display().to_string(),
        signature_source: Some(signature_path.display().to_string()),
        ed25519_public_key: Some(BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes())),
        max_bytes: RuntimePackageManager::DEFAULT_CATALOG_MAX_BYTES,
    })
    .unwrap_err();

    assert!(error.to_string().contains("signature verification failed"));
}
