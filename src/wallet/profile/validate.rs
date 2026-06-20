use anyhow::Result;

use crate::wallet::{
    crypto::{neo_address_payload, valid_compressed_public_key},
    NeoWalletProfile,
};

pub fn validate_neo_wallet_profile(profile: &NeoWalletProfile) -> Result<()> {
    validate_profile_id(&profile.id)?;
    if profile.label.trim().is_empty() {
        anyhow::bail!("Neo wallet profile label is required");
    }
    if profile.source_path.trim().is_empty() {
        anyhow::bail!("Neo wallet profile source path is required");
    }
    if neo_address_payload(&profile.primary_address).is_none() {
        anyhow::bail!("Neo wallet profile primary address is not valid");
    }
    if profile.account_count == 0 {
        anyhow::bail!("Neo wallet profile must reference at least one account");
    }
    if profile.encrypted_account_count == 0 {
        anyhow::bail!("Neo wallet profile must reference at least one encrypted account");
    }
    if profile.encrypted_account_count > profile.account_count {
        anyhow::bail!("Neo wallet profile encrypted account count exceeds account count");
    }
    if profile.default_account_count > profile.account_count {
        anyhow::bail!("Neo wallet profile default account count exceeds account count");
    }
    if profile.watch_only_account_count > profile.account_count {
        anyhow::bail!("Neo wallet profile watch-only account count exceeds account count");
    }
    if profile.wallet_sha256.len() != 64
        || !profile
            .wallet_sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        anyhow::bail!("Neo wallet profile wallet_sha256 must be 64 hexadecimal characters");
    }
    for public_key in &profile.contract_public_keys {
        if !valid_compressed_public_key(public_key) {
            anyhow::bail!("Neo wallet profile contract public key is not compressed hex");
        }
    }
    Ok(())
}

fn validate_profile_id(id: &str) -> Result<()> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Neo wallet profile id is required");
    }
    if trimmed.len() > 96 {
        anyhow::bail!("Neo wallet profile id must be 96 characters or fewer");
    }
    if !trimmed
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        anyhow::bail!("Neo wallet profile id may only contain letters, digits, '.', '_' and '-'");
    }
    Ok(())
}
