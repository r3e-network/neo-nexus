use super::*;

#[test]
fn runtime_release_catalog_selects_latest_compatible_release() {
    let platform = RuntimePlatform::current();
    let catalog_text = serde_json::json!({
        "schema_version": 1,
        "generated_at_unix": 1_800_000_000u64,
        "releases": [
            {
                "id": "neo-rs-v1-2",
                "label": "neo-rs v1.2",
                "node_type": "neo-rs",
                "version": "v1.2.0",
                "platform": { "os": platform.os, "arch": platform.arch },
                "download_url": "https://downloads.example.com/neo-rs/v1.2/neo-node",
                "download_file_name": "neo-node",
                "executable_name": "neo-node",
                "expected_sha256": "a".repeat(64),
                "max_bytes": 2048
            },
            {
                "id": "neo-rs-v1-10",
                "label": "neo-rs v1.10",
                "node_type": "neo-rs",
                "version": "v1.10.0",
                "platform": { "os": platform.os, "arch": platform.arch },
                "url": "https://downloads.example.com/neo-rs/v1.10/neo-node",
                "file_name": "neo-node",
                "executable_name": "neo-node",
                "expected_sha256": "b".repeat(64)
            },
            {
                "id": "neo-go-other-platform",
                "label": "neo-go other",
                "node_type": "neo-go",
                "version": "v9.0.0",
                "platform": { "os": "other-os", "arch": "other-arch" },
                "url": "https://downloads.example.com/neo-go",
                "file_name": "neo-go",
                "executable_name": "neo-go",
                "expected_sha256": "c".repeat(64)
            }
        ]
    })
    .to_string();

    let catalog = RuntimeReleaseCatalog::from_json(&catalog_text).unwrap();
    let compatible = catalog.compatible_releases(&platform);
    let latest = catalog.latest_for(NodeType::NeoRs, &platform).unwrap();
    let request = latest.download_request();
    let manifest = latest.manifest_for_source(PathBuf::from("/tmp/neo-node"));

    assert_eq!(compatible.len(), 2);
    assert_eq!(compatible[0].id, "neo-rs-v1-10");
    assert_eq!(latest.id, "neo-rs-v1-10");
    assert_eq!(
        request.url,
        "https://downloads.example.com/neo-rs/v1.10/neo-node"
    );
    assert_eq!(
        request.max_bytes,
        RuntimePackageManager::DEFAULT_DOWNLOAD_MAX_BYTES
    );
    assert_eq!(manifest.node_type, NodeType::NeoRs);
    assert_eq!(manifest.version, "v1.10.0");
    assert_eq!(manifest.executable_name, "neo-node");
}
