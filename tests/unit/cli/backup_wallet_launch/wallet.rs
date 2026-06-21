use super::super::*;

#[test]
fn wallet_validation_cli_reports_encrypted_nep6_wallet() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let wallet_path = temp_dir.path().join("validator.wallet.json");
    std::fs::write(&wallet_path, valid_nep6_wallet_json())?;
    let wallet_arg = wallet_path.display().to_string();

    let text_action = action_from_args(["neo-nexus", "--validate-wallet", &wallet_arg])?;
    assert!(
        matches!(text_action, CliAction::PrintWithExitCode { text, exit_code: 0 } if text.contains("wallet-validation: ok") && text.contains("encrypted-accounts: 1") && text.contains(VALID_NEP6_CONTRACT_PUBLIC_KEY))
    );

    let json_action = action_from_args(["neo-nexus", "--validate-wallet-json", &wallet_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code: 0 } = json_action else {
        anyhow::bail!("expected wallet validation JSON action");
    };
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["success"], true);
    assert_eq!(value["report"]["encrypted_account_count"], 1);
    assert_eq!(
        value["report"]["contract_public_keys"][0],
        VALID_NEP6_CONTRACT_PUBLIC_KEY
    );
    Ok(())
}

#[test]
fn wallet_profile_import_cli_validates_and_persists_encrypted_metadata() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let wallet_path = temp_dir.path().join("validator.wallet.json");
    std::fs::write(&wallet_path, valid_nep6_wallet_json())?;
    let db_arg = db_path.display().to_string();
    let wallet_arg = wallet_path.display().to_string();

    let action = action_from_args([
        "neo-nexus",
        "--import-wallet-profile",
        &db_arg,
        &wallet_arg,
        "validator-wallet-1",
        "Validator wallet 1",
    ])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 0 } if text.contains("wallet-profile-import: ok") && text.contains("profile-id: validator-wallet-1") && text.contains(VALID_NEP6_CONTRACT_PUBLIC_KEY))
    );
    let profiles = Repository::open(&db_path)?.list_neo_wallet_profiles()?;
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].id, "validator-wallet-1");
    assert_eq!(
        profiles[0].primary_address,
        "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq"
    );
    assert_eq!(
        profiles[0].contract_public_keys[0],
        VALID_NEP6_CONTRACT_PUBLIC_KEY
    );
    assert_eq!(profiles[0].encrypted_account_count, 1);
    assert_eq!(profiles[0].wallet_sha256.len(), 64);
    let persisted_json = serde_json::to_string(&profiles)?;
    assert!(!persisted_json.contains("6PYW"));
    assert!(!persisted_json.contains("password"));
    Ok(())
}

#[test]
fn wallet_validation_cli_rejects_plaintext_secret_material() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let wallet_path = temp_dir.path().join("plaintext.wallet.json");
    std::fs::write(
        &wallet_path,
        serde_json::json!({
            "name": "Unsafe wallet",
            "version": "3.0",
            "scrypt": {"n": 16384, "r": 8, "p": 8},
            "accounts": [{
                "address": "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq",
                "isDefault": true,
                "key": "Kzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
                "contract": {"script": "00"}
            }],
            "private_key": "Kzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
        })
        .to_string(),
    )?;
    let wallet_arg = wallet_path.display().to_string();

    let action = action_from_args(["neo-nexus", "--validate-wallet", &wallet_arg])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 1 } if text.contains("wallet-validation: failed") && text.contains("plaintext"))
    );
    Ok(())
}

#[test]
fn wallet_validation_cli_rejects_address_contract_mismatch() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let wallet_path = temp_dir.path().join("mismatched.wallet.json");
    let mut wallet: serde_json::Value = serde_json::from_str(&valid_nep6_wallet_json())?;
    wallet["accounts"][0]["contract"]["script"] =
        serde_json::Value::String(format!("2102{}ac", "a".repeat(64)));
    std::fs::write(&wallet_path, serde_json::to_string_pretty(&wallet)?)?;
    let wallet_arg = wallet_path.display().to_string();

    let action = action_from_args(["neo-nexus", "--validate-wallet", &wallet_arg])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 1 } if text.contains("wallet-validation: failed") && text.contains("address does not match contract script"))
    );
    Ok(())
}
