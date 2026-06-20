use super::*;

#[test]
fn runtime_download_cache_rejects_non_https_urls() {
    let request = RuntimeDownloadRequest {
        url: "http://example.com/neo-node".to_string(),
        file_name: "neo-node".to_string(),
        expected_sha256: "a".repeat(64),
        max_bytes: 1024,
    };

    let error = validate_download_request(&request).unwrap_err();

    assert!(error.to_string().contains("must use HTTPS"));
}

#[test]
fn runtime_download_redirect_policy_allows_only_https_targets() {
    let current = url::Url::parse("https://downloads.example.com/releases/neo-node").unwrap();
    let relative = validate_https_redirect(&current, "../assets/neo-node").unwrap();
    let downgrade = validate_https_redirect(&current, "http://downloads.example.com/neo-node");

    assert_eq!(
        relative.as_str(),
        "https://downloads.example.com/assets/neo-node"
    );
    assert!(downgrade
        .unwrap_err()
        .to_string()
        .contains("must stay on HTTPS"));
}

#[test]
fn runtime_download_cache_publishes_only_matching_hash() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bytes = b"downloaded runtime bytes";
    let source = temp_dir.path().join("source.bin");
    std::fs::write(&source, bytes).unwrap();
    let (sha256, expected_bytes) = sha256_file(&source).unwrap();
    let request = RuntimeDownloadRequest {
        url: "https://downloads.example.com/neo-node.tar.gz".to_string(),
        file_name: "neo-node.tar.gz".to_string(),
        expected_sha256: sha256.clone(),
        max_bytes: 1024,
    };

    let download = RuntimePackageManager::cache_download_from_reader(
        &request,
        temp_dir.path().join("downloads"),
        Cursor::new(bytes),
    )
    .unwrap();

    assert_eq!(download.sha256, sha256);
    assert_eq!(download.bytes, expected_bytes);
    assert!(download.path.is_file());
    assert!(download
        .path
        .ends_with(format!("neo-node.tar-{}.gz", &download.sha256[..12])));
    assert_eq!(std::fs::read(&download.path).unwrap(), bytes);
}

#[test]
fn runtime_download_cache_rejects_hash_mismatch_without_publish() {
    let temp_dir = tempfile::tempdir().unwrap();
    let request = RuntimeDownloadRequest {
        url: "https://downloads.example.com/neo-go".to_string(),
        file_name: "neo-go".to_string(),
        expected_sha256: "0".repeat(64),
        max_bytes: 1024,
    };

    let result = RuntimePackageManager::cache_download_from_reader(
        &request,
        temp_dir.path().join("downloads"),
        Cursor::new(b"real bytes"),
    );

    assert!(result
        .unwrap_err()
        .to_string()
        .contains("checksum mismatch"));
    let files = std::fs::read_dir(temp_dir.path().join("downloads"))
        .unwrap()
        .collect::<Vec<_>>();
    assert!(files.is_empty());
}

#[test]
fn runtime_download_cache_stops_when_size_limit_is_exceeded() {
    let temp_dir = tempfile::tempdir().unwrap();
    let request = RuntimeDownloadRequest {
        url: "https://downloads.example.com/neo-node".to_string(),
        file_name: "neo-node".to_string(),
        expected_sha256: "0".repeat(64),
        max_bytes: 4,
    };

    let result = RuntimePackageManager::cache_download_from_reader(
        &request,
        temp_dir.path().join("downloads"),
        Cursor::new(b"too large"),
    );

    assert!(result
        .unwrap_err()
        .to_string()
        .contains("exceeded size limit"));
}
