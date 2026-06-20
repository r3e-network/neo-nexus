use std::collections::BTreeSet;

use anyhow::Result;

pub(super) fn insert_unique_value(
    seen: &mut BTreeSet<String>,
    label: &str,
    value: &str,
) -> Result<()> {
    if value.is_empty() {
        anyhow::bail!("backup {label} is required");
    }
    if !seen.insert(value.to_string()) {
        anyhow::bail!("duplicate backup {label}: {value}");
    }
    Ok(())
}
