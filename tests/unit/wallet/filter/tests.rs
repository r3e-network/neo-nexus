use super::*;

#[test]
fn wallet_profile_filter_matches_operator_fields() {
    let profiles = [
        profile("validator-a", "Validator A", "AQLa", false),
        profile("backup-b", "Backup B", "AK2n", true),
    ];

    assert_ids(
        &profiles,
        NeoWalletProfileFilter::new(None, "validator"),
        &["validator-a"],
    );
    assert_ids(
        &profiles,
        NeoWalletProfileFilter::new(None, "AK2n"),
        &["backup-b"],
    );
    assert_ids(
        &profiles,
        NeoWalletProfileFilter::new(None, "PUBKEY-b"),
        &["backup-b"],
    );
    assert_ids(
        &profiles,
        NeoWalletProfileFilter::new(None, "unused"),
        &["validator-a"],
    );
}

#[test]
fn wallet_profile_filter_combines_usage_and_query() {
    let profiles = [
        profile("validator-a", "Validator A", "AQLa", true),
        profile("validator-b", "Validator B", "AK2n", false),
        profile("observer", "Observer", "Abcd", true),
    ];
    let filter = NeoWalletProfileFilter::new(Some(true), "validator");

    assert_ids(&profiles, filter, &["validator-a"]);
}

fn assert_ids(profiles: &[NeoWalletProfile], filter: NeoWalletProfileFilter, ids: &[&str]) {
    let filtered = filter_neo_wallet_profiles(profiles, &filter);
    let actual = filtered
        .iter()
        .map(|profile| profile.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(actual.as_slice(), ids);
}

fn profile(id: &str, label: &str, address: &str, used: bool) -> NeoWalletProfile {
    NeoWalletProfile {
        id: id.to_string(),
        label: label.to_string(),
        source_path: format!("/wallets/{id}.json"),
        wallet_version: Some("3.0".to_string()),
        primary_address: address.to_string(),
        contract_public_keys: vec![format!("PUBKEY-{id}")],
        wallet_sha256: "a".repeat(64),
        account_count: 1,
        encrypted_account_count: 1,
        default_account_count: 1,
        watch_only_account_count: 0,
        validated_at_unix: 1_800_000_000,
        last_used_at_unix: used.then_some(1_800_000_100),
    }
}
