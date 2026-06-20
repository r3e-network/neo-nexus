use super::{
    settings::{save_bool, save_display, save_optional_display},
    *,
};

impl Repository {
    pub fn save_runtime_upgrade_policy(&self, policy: &RuntimeUpgradePolicy) -> Result<()> {
        let policy = normalized_runtime_upgrade_policy(policy);
        validate_runtime_upgrade_policy(&policy)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_bool(
            &transaction,
            SETTING_RUNTIME_UPGRADE_ENABLED,
            policy.enabled,
        )?;
        save_setting(
            &transaction,
            SETTING_RUNTIME_UPGRADE_CATALOG_PROFILE_ID,
            policy.catalog_profile_id.as_deref().unwrap_or(""),
        )?;
        save_display(
            &transaction,
            SETTING_RUNTIME_UPGRADE_INTERVAL_MINUTES,
            policy.interval_minutes,
        )?;
        save_bool(
            &transaction,
            SETTING_RUNTIME_UPGRADE_REQUIRE_SIGNED_CATALOG,
            policy.require_signed_catalog,
        )?;
        save_display(
            &transaction,
            SETTING_RUNTIME_UPGRADE_MAX_NODES_PER_RUN,
            policy.max_nodes_per_run,
        )?;
        save_bool(
            &transaction,
            SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_ENABLED,
            policy.maintenance_window_enabled,
        )?;
        save_display(
            &transaction,
            SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_START_MINUTE_UTC,
            policy.maintenance_window_start_minute_utc,
        )?;
        save_display(
            &transaction,
            SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_END_MINUTE_UTC,
            policy.maintenance_window_end_minute_utc,
        )?;
        save_display(
            &transaction,
            SETTING_RUNTIME_UPGRADE_WAVE_DELAY_MINUTES,
            policy.wave_delay_minutes,
        )?;
        save_optional_display(
            &transaction,
            SETTING_RUNTIME_UPGRADE_LAST_CHECKED_AT_UNIX,
            policy.last_checked_at_unix,
        )?;
        save_optional_display(
            &transaction,
            SETTING_RUNTIME_UPGRADE_LAST_APPLIED_AT_UNIX,
            policy.last_applied_at_unix,
        )?;
        transaction.commit()?;
        Ok(())
    }
}
