use super::*;

#[test]
fn persists_runtime_catalog_profiles_and_load_metadata() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let catalog_path = temp_dir.path().join("runtime-catalog.json");
    let profile = RuntimeCatalogProfile {
        id: "official-neo-rs".to_string(),
        label: "Official neo-rs catalog".to_string(),
        source: catalog_path.display().to_string(),
        signature_source: None,
        ed25519_public_key: None,
        max_bytes: 1024 * 1024,
        enabled: true,
        last_loaded_at_unix: None,
        last_signature_verified: None,
        last_bytes: None,
    };

    repository.upsert_runtime_catalog_profile(&profile).unwrap();
    let persisted = repository.list_runtime_catalog_profiles().unwrap();

    assert_eq!(persisted, vec![profile.clone()]);

    let load = RuntimeCatalogLoad {
        catalog: RuntimeReleaseCatalog {
            schema_version: 1,
            generated_at_unix: None,
            releases: Vec::new(),
        },
        source: profile.source.clone(),
        bytes: 512,
        signature_verified: None,
        loaded_at_unix: 1_800_000_100,
    };
    repository
        .mark_runtime_catalog_profile_loaded(&profile.id, &load)
        .unwrap();
    let loaded = repository.list_runtime_catalog_profiles().unwrap();

    assert_eq!(loaded[0].last_loaded_at_unix, Some(1_800_000_100));
    assert_eq!(loaded[0].last_signature_verified, None);
    assert_eq!(loaded[0].last_bytes, Some(512));

    repository
        .delete_runtime_catalog_profile("official-neo-rs")
        .unwrap();
    assert!(repository
        .list_runtime_catalog_profiles()
        .unwrap()
        .is_empty());
}

#[test]
fn persists_runtime_signer_profiles_and_usage_metadata() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let signing_key = SigningKey::from_bytes(&[31u8; 32]);
    let profile = RuntimeSignerProfile {
        id: "official-neo-rs-signer".to_string(),
        label: "Official neo-rs signer".to_string(),
        ed25519_public_key: BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes()),
        enabled: true,
        created_at_unix: 1_800_000_000,
        last_used_at_unix: None,
    };

    repository.upsert_runtime_signer_profile(&profile).unwrap();
    let persisted = repository.list_runtime_signer_profiles().unwrap();

    assert_eq!(persisted, vec![profile.clone()]);

    repository
        .mark_runtime_signer_profile_used(&profile.id, 1_800_000_200)
        .unwrap();
    let used = repository.list_runtime_signer_profiles().unwrap();

    assert_eq!(used[0].last_used_at_unix, Some(1_800_000_200));

    repository
        .delete_runtime_signer_profile("official-neo-rs-signer")
        .unwrap();
    assert!(repository
        .list_runtime_signer_profiles()
        .unwrap()
        .is_empty());
}
