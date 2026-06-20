use super::*;

impl Repository {
    pub fn load_alert_routing_policy(&self) -> Result<AlertRoutingPolicy> {
        let connection = self.connection()?;
        let default_policy = AlertRoutingPolicy::default();
        let webhook_url = load_setting(&connection, SETTING_ALERT_ROUTING_WEBHOOK_URL)?
            .and_then(|value| normalized_webhook_url(&value).ok());
        let policy = AlertRoutingPolicy {
            enabled: load_setting(&connection, SETTING_ALERT_ROUTING_ENABLED)?
                .as_deref()
                .map_or(default_policy.enabled, parse_bool_setting),
            provider: load_setting(&connection, SETTING_ALERT_ROUTING_PROVIDER)?
                .as_deref()
                .and_then(|value| AlertProvider::from_str(value).ok())
                .unwrap_or(default_policy.provider),
            min_severity: load_setting(&connection, SETTING_ALERT_ROUTING_MIN_SEVERITY)?
                .as_deref()
                .and_then(|value| EventSeverity::from_str(value).ok())
                .unwrap_or(default_policy.min_severity),
            webhook_url,
            timeout_seconds: load_setting(&connection, SETTING_ALERT_ROUTING_TIMEOUT_SECONDS)?
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(default_policy.timeout_seconds),
        }
        .normalized();
        Ok(policy)
    }

    pub fn save_alert_routing_policy(&self, policy: AlertRoutingPolicy) -> Result<()> {
        if let Some(message) = policy.validation_message() {
            anyhow::bail!(message);
        }
        let policy = policy.normalized();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(
            &transaction,
            SETTING_ALERT_ROUTING_ENABLED,
            if policy.enabled { "true" } else { "false" },
        )?;
        save_setting(
            &transaction,
            SETTING_ALERT_ROUTING_PROVIDER,
            &policy.provider.to_string(),
        )?;
        save_setting(
            &transaction,
            SETTING_ALERT_ROUTING_MIN_SEVERITY,
            &policy.min_severity.to_string(),
        )?;
        save_setting(
            &transaction,
            SETTING_ALERT_ROUTING_WEBHOOK_URL,
            policy.webhook_url.as_deref().unwrap_or(""),
        )?;
        save_setting(
            &transaction,
            SETTING_ALERT_ROUTING_TIMEOUT_SECONDS,
            &policy.timeout_seconds.to_string(),
        )?;
        transaction.commit()?;
        Ok(())
    }
}
