use super::*;

#[test]
fn persists_neo_wallet_profiles_without_secret_material() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let profile = NeoWalletProfile {
        id: "validator-wallet-1".to_string(),
        label: "Validator wallet 1".to_string(),
        source_path: "/secure/wallets/validator-1.wallet.json".to_string(),
        wallet_version: Some("3.0".to_string()),
        primary_address: "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq".to_string(),
        contract_public_keys: vec![
            "036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0".to_string(),
        ],
        wallet_sha256: "b".repeat(64),
        account_count: 1,
        encrypted_account_count: 1,
        default_account_count: 1,
        watch_only_account_count: 0,
        validated_at_unix: 1_800_000_000,
        last_used_at_unix: None,
    };

    repository.upsert_neo_wallet_profile(&profile).unwrap();
    let persisted = repository.list_neo_wallet_profiles().unwrap();

    assert_eq!(persisted, vec![profile.clone()]);
    let persisted_json = serde_json::to_string(&persisted).unwrap();
    assert!(!persisted_json.contains("6P"));
    assert!(!persisted_json.contains("password"));
    assert!(!persisted_json.contains("private"));

    repository
        .mark_neo_wallet_profile_used(&profile.id, 1_800_000_200)
        .unwrap();
    let used = repository.list_neo_wallet_profiles().unwrap();

    assert_eq!(used[0].last_used_at_unix, Some(1_800_000_200));

    repository
        .delete_neo_wallet_profile("validator-wallet-1")
        .unwrap();
    assert!(repository.list_neo_wallet_profiles().unwrap().is_empty());
}
