use super::{
    settings::{load_bool_or, load_number_or, load_optional_number, load_optional_text},
    *,
};

impl Repository {
    pub fn load_runtime_upgrade_policy(&self) -> Result<RuntimeUpgradePolicy> {
        let connection = self.connection()?;
        let default_policy = RuntimeUpgradePolicy::disabled();
        let policy = RuntimeUpgradePolicy {
            enabled: load_bool_or(
                &connection,
                SETTING_RUNTIME_UPGRADE_ENABLED,
                default_policy.enabled,
            )?,
            catalog_profile_id: load_optional_text(
                &connection,
                SETTING_RUNTIME_UPGRADE_CATALOG_PROFILE_ID,
            )?,
            interval_minutes: load_number_or(
                &connection,
                SETTING_RUNTIME_UPGRADE_INTERVAL_MINUTES,
                default_policy.interval_minutes,
            )?,
            require_signed_catalog: load_bool_or(
                &connection,
                SETTING_RUNTIME_UPGRADE_REQUIRE_SIGNED_CATALOG,
                default_policy.require_signed_catalog,
            )?,
            max_nodes_per_run: load_number_or(
                &connection,
                SETTING_RUNTIME_UPGRADE_MAX_NODES_PER_RUN,
                default_policy.max_nodes_per_run,
            )?,
            maintenance_window_enabled: load_bool_or(
                &connection,
                SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_ENABLED,
                default_policy.maintenance_window_enabled,
            )?,
            maintenance_window_start_minute_utc: load_number_or(
                &connection,
                SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_START_MINUTE_UTC,
                default_policy.maintenance_window_start_minute_utc,
            )?,
            maintenance_window_end_minute_utc: load_number_or(
                &connection,
                SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_END_MINUTE_UTC,
                default_policy.maintenance_window_end_minute_utc,
            )?,
            wave_delay_minutes: load_number_or(
                &connection,
                SETTING_RUNTIME_UPGRADE_WAVE_DELAY_MINUTES,
                default_policy.wave_delay_minutes,
            )?,
            last_checked_at_unix: load_optional_number(
                &connection,
                SETTING_RUNTIME_UPGRADE_LAST_CHECKED_AT_UNIX,
            )?,
            last_applied_at_unix: load_optional_number(
                &connection,
                SETTING_RUNTIME_UPGRADE_LAST_APPLIED_AT_UNIX,
            )?,
        };
        validate_runtime_upgrade_policy(&policy)?;
        Ok(policy)
    }
}
