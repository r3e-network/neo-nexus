use super::*;

impl Repository {
    pub fn load_rpc_health_monitor_policy(&self) -> Result<RpcHealthMonitorPolicy> {
        let connection = self.connection()?;
        let default_policy = RpcHealthMonitorPolicy::enabled_default();
        let policy = RpcHealthMonitorPolicy {
            enabled: load_setting(&connection, SETTING_RPC_HEALTH_MONITOR_ENABLED)?
                .as_deref()
                .map_or(default_policy.enabled, parse_bool_setting),
            interval_seconds: load_setting(
                &connection,
                SETTING_RPC_HEALTH_MONITOR_INTERVAL_SECONDS,
            )?
            .as_deref()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(default_policy.interval_seconds),
        }
        .normalized();
        Ok(policy)
    }

    pub fn save_rpc_health_monitor_policy(&self, policy: RpcHealthMonitorPolicy) -> Result<()> {
        let policy = policy.normalized();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(
            &transaction,
            SETTING_RPC_HEALTH_MONITOR_ENABLED,
            if policy.enabled { "true" } else { "false" },
        )?;
        save_setting(
            &transaction,
            SETTING_RPC_HEALTH_MONITOR_INTERVAL_SECONDS,
            &policy.interval_seconds.to_string(),
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn load_remote_federation_monitor_policy(&self) -> Result<RemoteFederationMonitorPolicy> {
        let connection = self.connection()?;
        let default_policy = RemoteFederationMonitorPolicy::enabled_default();
        let policy = RemoteFederationMonitorPolicy {
            enabled: load_setting(&connection, SETTING_REMOTE_FEDERATION_MONITOR_ENABLED)?
                .as_deref()
                .map_or(default_policy.enabled, parse_bool_setting),
            interval_seconds: load_setting(
                &connection,
                SETTING_REMOTE_FEDERATION_MONITOR_INTERVAL_SECONDS,
            )?
            .as_deref()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(default_policy.interval_seconds),
        }
        .normalized();
        Ok(policy)
    }

    pub fn save_remote_federation_monitor_policy(
        &self,
        policy: RemoteFederationMonitorPolicy,
    ) -> Result<()> {
        let policy = policy.normalized();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        save_setting(
            &transaction,
            SETTING_REMOTE_FEDERATION_MONITOR_ENABLED,
            if policy.enabled { "true" } else { "false" },
        )?;
        save_setting(
            &transaction,
            SETTING_REMOTE_FEDERATION_MONITOR_INTERVAL_SECONDS,
            &policy.interval_seconds.to_string(),
        )?;
        transaction.commit()?;
        Ok(())
    }
}
