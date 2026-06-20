use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

use crate::runtime::RuntimeUpgradePolicy;

use super::super::WORKSPACE_BACKUP_SETTING_KEYS;

pub(in crate::repository) fn load_setting(
    connection: &Connection,
    key: &str,
) -> Result<Option<String>> {
    connection
        .query_row(
            "SELECT value FROM workspace_settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .with_context(|| format!("failed to load workspace setting {key}"))
}

pub(in crate::repository) fn save_setting(
    transaction: &rusqlite::Transaction<'_>,
    key: &str,
    value: &str,
) -> Result<()> {
    transaction.execute(
        "INSERT INTO workspace_settings (key, value)
         VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub(crate) fn validate_backup_setting_key(key: &str) -> Result<()> {
    if WORKSPACE_BACKUP_SETTING_KEYS.contains(&key) {
        Ok(())
    } else {
        anyhow::bail!("unsupported workspace setting in backup: {key}");
    }
}

pub(in crate::repository) fn optional_setting(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(in crate::repository) fn normalized_runtime_upgrade_policy(
    policy: &RuntimeUpgradePolicy,
) -> RuntimeUpgradePolicy {
    let mut normalized = policy.clone();
    normalized.catalog_profile_id = policy
        .catalog_profile_id
        .as_deref()
        .and_then(optional_setting);
    normalized
}

pub(in crate::repository) fn parse_bool_setting(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "enabled"
    )
}
