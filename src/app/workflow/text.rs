use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::app::domain::NeoWalletProfile;

pub(in crate::app) fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    if seconds == 0 {
        "now".to_string()
    } else {
        format!("{seconds}s")
    }
}

pub(in crate::app) fn optional_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(in crate::app) fn non_empty_text(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

pub(in crate::app) fn committee_keys_with_wallet_profile(
    existing: &str,
    public_key: &str,
) -> String {
    let mut keys = existing
        .split(|character: char| character == ',' || character == ';' || character.is_whitespace())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !keys
        .iter()
        .any(|key| key.eq_ignore_ascii_case(public_key.trim()))
    {
        keys.push(public_key.trim().to_string());
    }
    keys.join("\n")
}

pub(in crate::app) fn signer_refs_with_wallet_profile(
    existing: &str,
    profile: &NeoWalletProfile,
) -> String {
    let Some(public_key) = profile.contract_public_keys.first() else {
        return existing.to_string();
    };
    if signer_refs_has_public_key(existing, public_key) {
        return existing.to_string();
    }
    let reference = format!("{}|{}", public_key.trim(), profile.source_path.trim());
    let trimmed = existing.trim_end();
    if trimmed.is_empty() {
        reference
    } else {
        format!("{trimmed}\n{reference}")
    }
}

pub(in crate::app) fn signer_refs_has_public_key(existing: &str, public_key: &str) -> bool {
    existing.lines().any(|line| {
        line.split('|')
            .next()
            .map(str::trim)
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(public_key.trim()))
    })
}

pub(in crate::app) fn current_unix_time() -> anyhow::Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow::anyhow!("system clock is before Unix epoch"))?
        .as_secs())
}

pub(in crate::app) fn data_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("NEONEXUS_DATA_DIR") {
        return PathBuf::from(path);
    }

    dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("NeoNexus")
}
