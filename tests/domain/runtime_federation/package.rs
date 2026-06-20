use super::*;

#[test]
fn runtime_package_manager_installs_verified_local_binary_for_current_platform() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("neo-node-source");
    std::fs::write(&source, "neo-rs executable bytes").unwrap();
    let (sha256, bytes) = sha256_file(&source).unwrap();
    let manifest = RuntimePackageManifest {
        id: "neo-rs-v1".to_string(),
        label: "neo-rs v1".to_string(),
        node_type: NodeType::NeoRs,
        version: "v1.0.0".to_string(),
        platform: RuntimePlatform::current(),
        source_path: source,
        executable_name: "neo-node".to_string(),
        expected_sha256: sha256.clone(),
        signature_path: None,
        ed25519_public_key: None,
    };

    let verification = RuntimePackageManager::verify(&manifest).unwrap();
    let installation =
        RuntimePackageManager::install(&manifest, temp_dir.path().join("runtimes")).unwrap();

    assert!(verification.matches);
    assert!(verification.platform_matches);
    assert_eq!(installation.package_id, "neo-rs-v1");
    assert_eq!(installation.node_type, NodeType::NeoRs);
    assert_eq!(installation.version, "v1.0.0");
    assert_eq!(installation.sha256, sha256);
    assert_eq!(installation.bytes, bytes);
    assert!(installation.binary_path.ends_with("neo-node"));
    assert!(installation.binary_path.is_file());
    assert_eq!(
        std::fs::read_to_string(&installation.binary_path).unwrap(),
        "neo-rs executable bytes"
    );
    let manifest_text = std::fs::read_to_string(
        installation
            .binary_path
            .with_file_name("runtime-install.json"),
    )
    .unwrap();
    assert!(manifest_text.contains("\"package_id\": \"neo-rs-v1\""));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&installation.binary_path)
            .unwrap()
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0);
    }
}

#[test]
fn runtime_package_manager_rejects_checksum_mismatch_before_install() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("neo-go");
    std::fs::write(&source, "neo-go executable bytes").unwrap();
    let manifest = RuntimePackageManifest {
        id: "neo-go-bad".to_string(),
        label: "neo-go bad".to_string(),
        node_type: NodeType::NeoGo,
        version: "v0.1.0".to_string(),
        platform: RuntimePlatform::current(),
        source_path: source,
        executable_name: "neo-go".to_string(),
        expected_sha256: "0".repeat(64),
        signature_path: None,
        ed25519_public_key: None,
    };

    let result = RuntimePackageManager::install(&manifest, temp_dir.path().join("runtimes"));

    assert!(result
        .unwrap_err()
        .to_string()
        .contains("checksum mismatch"));
    assert!(!temp_dir.path().join("runtimes").join("neo-go").exists());
}

#[test]
fn runtime_package_manager_verifies_detached_ed25519_signature() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("neo-node");
    std::fs::write(&source, "signed neo runtime").unwrap();
    let (sha256, _) = sha256_file(&source).unwrap();
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let signature = signing_key.sign(b"signed neo runtime");
    let signature_path = temp_dir.path().join("neo-node.sig");
    std::fs::write(
        &signature_path,
        BASE64_STANDARD.encode(signature.to_bytes()),
    )
    .unwrap();
    let public_key = BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes());
    let manifest = RuntimePackageManifest {
        id: "neo-rs-signed".to_string(),
        label: "neo-rs signed".to_string(),
        node_type: NodeType::NeoRs,
        version: "v1.2.0".to_string(),
        platform: RuntimePlatform::current(),
        source_path: source,
        executable_name: "neo-node".to_string(),
        expected_sha256: sha256,
        signature_path: Some(signature_path),
        ed25519_public_key: Some(public_key.clone()),
    };

    let verification = RuntimePackageManager::verify(&manifest).unwrap();
    let installation =
        RuntimePackageManager::install(&manifest, temp_dir.path().join("runtimes")).unwrap();

    assert_eq!(verification.signature_verified, Some(true));
    assert!(installation.signature_verified);
    assert_eq!(installation.signer_public_key, Some(public_key));
}

#[test]
fn runtime_package_manager_rejects_invalid_detached_signature() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("neo-node");
    std::fs::write(&source, "tampered neo runtime").unwrap();
    let (sha256, _) = sha256_file(&source).unwrap();
    let signing_key = SigningKey::from_bytes(&[9u8; 32]);
    let signature = signing_key.sign(b"original neo runtime");
    let signature_path = temp_dir.path().join("neo-node.sig");
    std::fs::write(
        &signature_path,
        BASE64_STANDARD.encode(signature.to_bytes()),
    )
    .unwrap();
    let manifest = RuntimePackageManifest {
        id: "neo-rs-bad-signature".to_string(),
        label: "neo-rs bad signature".to_string(),
        node_type: NodeType::NeoRs,
        version: "v1.2.1".to_string(),
        platform: RuntimePlatform::current(),
        source_path: source,
        executable_name: "neo-node".to_string(),
        expected_sha256: sha256,
        signature_path: Some(signature_path),
        ed25519_public_key: Some(BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes())),
    };

    let verification = RuntimePackageManager::verify(&manifest).unwrap();
    let result = RuntimePackageManager::install(&manifest, temp_dir.path().join("runtimes"));

    assert_eq!(verification.signature_verified, Some(false));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("signature verification failed"));
}

#[test]
fn runtime_manifest_rejects_partial_signature_configuration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let source = temp_dir.path().join("neo-go");
    std::fs::write(&source, "neo-go bytes").unwrap();
    let (sha256, _) = sha256_file(&source).unwrap();
    let manifest = RuntimePackageManifest {
        id: "partial-signature".to_string(),
        label: "partial signature".to_string(),
        node_type: NodeType::NeoGo,
        version: "v0.108.0".to_string(),
        platform: RuntimePlatform::current(),
        source_path: source,
        executable_name: "neo-go".to_string(),
        expected_sha256: sha256,
        signature_path: Some(temp_dir.path().join("neo-go.sig")),
        ed25519_public_key: None,
    };

    let error = RuntimePackageManager::verify(&manifest).unwrap_err();

    assert!(error.to_string().contains("provided together"));
}
