use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

pub(super) fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_secs())
}
