use anyhow::Result;

use super::RuntimeUpgradePolicy;

pub fn validate_runtime_upgrade_policy(policy: &RuntimeUpgradePolicy) -> Result<()> {
    if policy.enabled
        && policy
            .catalog_profile_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
    {
        anyhow::bail!("enabled runtime upgrade policy requires a catalog profile");
    }
    if !(RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES..=RuntimeUpgradePolicy::MAX_INTERVAL_MINUTES)
        .contains(&policy.interval_minutes)
    {
        anyhow::bail!(
            "runtime upgrade interval must be between {} and {} minutes",
            RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES,
            RuntimeUpgradePolicy::MAX_INTERVAL_MINUTES
        );
    }
    if policy.max_nodes_per_run == 0
        || policy.max_nodes_per_run > RuntimeUpgradePolicy::MAX_NODES_PER_RUN
    {
        anyhow::bail!(
            "runtime upgrade batch size must be between 1 and {} nodes",
            RuntimeUpgradePolicy::MAX_NODES_PER_RUN
        );
    }
    if policy.maintenance_window_enabled {
        if policy.maintenance_window_start_minute_utc >= RuntimeUpgradePolicy::MINUTES_PER_DAY
            || policy.maintenance_window_end_minute_utc >= RuntimeUpgradePolicy::MINUTES_PER_DAY
        {
            anyhow::bail!(
                "runtime upgrade maintenance window minutes must be between 0 and {}",
                RuntimeUpgradePolicy::MINUTES_PER_DAY - 1
            );
        }
        if policy.maintenance_window_start_minute_utc == policy.maintenance_window_end_minute_utc {
            anyhow::bail!("runtime upgrade maintenance window must have a non-zero duration");
        }
    }
    if policy.wave_delay_minutes > RuntimeUpgradePolicy::MAX_WAVE_DELAY_MINUTES {
        anyhow::bail!(
            "runtime upgrade wave delay must be no more than {} minutes",
            RuntimeUpgradePolicy::MAX_WAVE_DELAY_MINUTES
        );
    }
    Ok(())
}
