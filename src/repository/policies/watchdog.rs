use super::*;

impl Repository {
    pub fn load_watchdog_policy(&self) -> Result<RestartPolicy> {
        let connection = self.connection()?;
        let default_policy = default_restart_policy();
        let enabled = load_setting(&connection, SETTING_WATCHDOG_ENABLED)?
            .as_deref()
            .map_or(default_policy.enabled, parse_bool_setting);
        let max_restart_attempts = load_setting(&connection, SETTING_WATCHDOG_MAX_ATTEMPTS)?
            .as_deref()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(default_policy.max_restart_attempts);
        let base_delay_seconds = load_setting(&connection, SETTING_WATCHDOG_BASE_DELAY_SECONDS)?
            .as_deref()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(default_policy.base_delay.as_secs());
        let max_delay_seconds = load_setting(&connection, SETTING_WATCHDOG_MAX_DELAY_SECONDS)?
            .as_deref()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(default_policy.max_delay.as_secs());

        Ok(RestartPolicy::with_enabled(
            enabled,
            max_restart_attempts,
            Duration::from_secs(base_delay_seconds),
            Duration::from_secs(max_delay_seconds),
        ))
    }

    pub fn save_watchdog_policy(&self, policy: RestartPolicy) -> Result<()> {
        let policy = policy.normalized();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(
            &transaction,
            SETTING_WATCHDOG_ENABLED,
            if policy.enabled { "true" } else { "false" },
        )?;
        save_setting(
            &transaction,
            SETTING_WATCHDOG_MAX_ATTEMPTS,
            &policy.max_restart_attempts.to_string(),
        )?;
        save_setting(
            &transaction,
            SETTING_WATCHDOG_BASE_DELAY_SECONDS,
            &policy.base_delay.as_secs().to_string(),
        )?;
        save_setting(
            &transaction,
            SETTING_WATCHDOG_MAX_DELAY_SECONDS,
            &policy.max_delay.as_secs().to_string(),
        )?;
        transaction.commit()?;
        Ok(())
    }
}
