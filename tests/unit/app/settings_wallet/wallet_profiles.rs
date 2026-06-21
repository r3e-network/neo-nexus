use super::super::*;
use crate::wallet::NeoWalletProfile;

#[test]
fn wallet_profile_actions_import_use_delete_and_audit_events() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let wallet_path = temp_dir.path().join("validator.wallet.json");
    std::fs::write(&wallet_path, valid_nep6_wallet_json())?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.wallet_profile_source = wallet_path.display().to_string();
    app.wallet_profile_id = "validator-wallet-1".to_string();
    app.wallet_profile_label = "Validator wallet 1".to_string();

    app.import_neo_wallet_profile_from_form();

    assert_eq!(app.neo_wallet_profiles.len(), 1);
    assert_eq!(
        app.selected_neo_wallet_profile.as_deref(),
        Some("validator-wallet-1")
    );
    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("Wallet profile imported")));
    let Some(profile) = app.selected_neo_wallet_profile() else {
        anyhow::bail!("wallet profile should be selected after import");
    };
    assert_eq!(
        profile.primary_address,
        "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq"
    );
    assert_eq!(
        profile.contract_public_keys,
        vec![VALID_NEP6_CONTRACT_PUBLIC_KEY.to_string()]
    );

    app.mark_selected_neo_wallet_profile_used();
    assert!(app
        .selected_neo_wallet_profile()
        .and_then(|profile| profile.last_used_at_unix)
        .is_some());

    app.delete_selected_neo_wallet_profile();
    assert!(app.neo_wallet_profiles.is_empty());
    assert!(app.repository.list_neo_wallet_profiles()?.is_empty());

    let events = app
        .repository
        .list_events(RuntimeEventFilter::new(None, "wallet-profile", 20))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::NeoWalletProfileImported));
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::NeoWalletProfileUsed));
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::NeoWalletProfileDeleted));

    Ok(())
}

#[test]
fn wallet_profile_action_adds_private_network_signer_reference() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let wallet_path = temp_dir.path().join("validator.wallet.json");
    std::fs::write(&wallet_path, valid_nep6_wallet_json())?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.wallet_profile_source = wallet_path.display().to_string();
    app.wallet_profile_id = "validator-wallet-1".to_string();
    app.wallet_profile_label = "Validator wallet 1".to_string();
    app.import_neo_wallet_profile_from_form();

    app.use_selected_neo_wallet_profile_for_private_network_signer_refs();

    assert_eq!(app.selected_view, View::Roles);
    assert_eq!(
        app.private_network_committee_keys.trim(),
        VALID_NEP6_CONTRACT_PUBLIC_KEY
    );
    assert_eq!(
        app.private_network_signer_refs.trim(),
        format!(
            "{}|{}",
            VALID_NEP6_CONTRACT_PUBLIC_KEY,
            wallet_path.display()
        )
    );
    let roster = CommitteeRoster::from_public_keys_and_references(
        &app.private_network_committee_keys,
        &app.private_network_signer_refs,
    )?
    .ok_or_else(|| anyhow::anyhow!("roster should be built from wallet profile"))?;
    assert_eq!(
        roster.signers[0].wallet_path.as_deref(),
        Some(wallet_path.as_path())
    );

    app.use_selected_neo_wallet_profile_for_private_network_signer_refs();
    assert_eq!(
        app.private_network_signer_refs
            .lines()
            .filter(|line| line.contains(VALID_NEP6_CONTRACT_PUBLIC_KEY))
            .count(),
        1
    );
    assert!(app
        .selected_neo_wallet_profile()
        .and_then(|profile| profile.last_used_at_unix)
        .is_some());
    assert!(app.notice.as_deref().is_some_and(|notice| {
        notice.contains("Wallet profile added to signer references")
            || notice.contains("already exists")
    }));

    let events =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "signer references", 20))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::NeoWalletProfileUsed));

    Ok(())
}

#[test]
fn wallet_profile_filter_limits_visible_registry() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    repository.upsert_neo_wallet_profile(&wallet_profile("validator-a", "Validator A", None))?;
    repository.upsert_neo_wallet_profile(&wallet_profile(
        "validator-b",
        "Validator B",
        Some(1_800_000_200),
    ))?;
    repository.upsert_neo_wallet_profile(&wallet_profile(
        "observer",
        "Observer",
        Some(1_800_000_300),
    ))?;
    let mut app = NeoNexusApp::new(repository);
    app.wallet_profile_query = "validator".to_string();
    app.wallet_profile_used_filter = Some(true);

    let visible = app.filtered_neo_wallet_profiles();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, "validator-b");

    app.wallet_profile_page = 6;
    app.ensure_valid_neo_wallet_profile_selection();
    assert_eq!(app.wallet_profile_page, 0);

    Ok(())
}

fn wallet_profile(id: &str, label: &str, last_used_at_unix: Option<u64>) -> NeoWalletProfile {
    NeoWalletProfile {
        id: id.to_string(),
        label: label.to_string(),
        source_path: format!("/wallets/{id}.json"),
        wallet_version: Some("3.0".to_string()),
        primary_address: "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq".to_string(),
        contract_public_keys: vec![VALID_NEP6_CONTRACT_PUBLIC_KEY.to_string()],
        wallet_sha256: "a".repeat(64),
        account_count: 1,
        encrypted_account_count: 1,
        default_account_count: 1,
        watch_only_account_count: 0,
        validated_at_unix: 1_800_000_000,
        last_used_at_unix,
    }
}
