use super::*;

#[test]
fn runtime_release_catalog_rejects_insecure_download_urls() {
    let platform = RuntimePlatform::current();
    let catalog_text = serde_json::json!({
        "schema_version": 1,
        "releases": [
            {
                "id": "neo-rs-insecure",
                "label": "neo-rs insecure",
                "node_type": "neo-rs",
                "version": "v1.0.0",
                "platform": { "os": platform.os, "arch": platform.arch },
                "url": "http://downloads.example.com/neo-node",
                "file_name": "neo-node",
                "executable_name": "neo-node",
                "expected_sha256": "d".repeat(64)
            }
        ]
    })
    .to_string();

    let error = RuntimeReleaseCatalog::from_json(&catalog_text).unwrap_err();

    assert!(error.to_string().contains("must use HTTPS"));
}

#[test]
fn runtime_release_catalog_requires_signature_for_https_sources() {
    let request = RuntimeCatalogLoadRequest {
        source: "https://downloads.example.com/runtime-catalog.json".to_string(),
        signature_source: None,
        ed25519_public_key: None,
        max_bytes: RuntimePackageManager::DEFAULT_CATALOG_MAX_BYTES,
    };

    let error = validate_catalog_load_request(&request).unwrap_err();

    assert!(error
        .to_string()
        .contains("remote runtime catalogs require"));
}
