pub(in crate::repository) const SETTING_WATCHDOG_ENABLED: &str = "watchdog.enabled";
pub(in crate::repository) const SETTING_WATCHDOG_MAX_ATTEMPTS: &str =
    "watchdog.max_restart_attempts";
pub(in crate::repository) const SETTING_WATCHDOG_BASE_DELAY_SECONDS: &str =
    "watchdog.base_delay_seconds";
pub(in crate::repository) const SETTING_WATCHDOG_MAX_DELAY_SECONDS: &str =
    "watchdog.max_delay_seconds";

pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_ENABLED: &str = "runtime_upgrade.enabled";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_CATALOG_PROFILE_ID: &str =
    "runtime_upgrade.catalog_profile_id";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_INTERVAL_MINUTES: &str =
    "runtime_upgrade.interval_minutes";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_REQUIRE_SIGNED_CATALOG: &str =
    "runtime_upgrade.require_signed_catalog";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_MAX_NODES_PER_RUN: &str =
    "runtime_upgrade.max_nodes_per_run";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_ENABLED: &str =
    "runtime_upgrade.maintenance_window.enabled";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_START_MINUTE_UTC: &str =
    "runtime_upgrade.maintenance_window.start_minute_utc";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_END_MINUTE_UTC: &str =
    "runtime_upgrade.maintenance_window.end_minute_utc";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_WAVE_DELAY_MINUTES: &str =
    "runtime_upgrade.wave_delay_minutes";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_LAST_CHECKED_AT_UNIX: &str =
    "runtime_upgrade.last_checked_at_unix";
pub(in crate::repository) const SETTING_RUNTIME_UPGRADE_LAST_APPLIED_AT_UNIX: &str =
    "runtime_upgrade.last_applied_at_unix";

pub(in crate::repository) const SETTING_RPC_HEALTH_MONITOR_ENABLED: &str =
    "rpc_health_monitor.enabled";
pub(in crate::repository) const SETTING_RPC_HEALTH_MONITOR_INTERVAL_SECONDS: &str =
    "rpc_health_monitor.interval_seconds";
pub(in crate::repository) const SETTING_REMOTE_FEDERATION_MONITOR_ENABLED: &str =
    "remote_federation_monitor.enabled";
pub(in crate::repository) const SETTING_REMOTE_FEDERATION_MONITOR_INTERVAL_SECONDS: &str =
    "remote_federation_monitor.interval_seconds";

pub(in crate::repository) const SETTING_PRIVATE_NETWORK_SIDECARS_ALLOW_EXTERNAL: &str =
    "private_network.sidecars.allow_external";

pub(in crate::repository) const SETTING_ALERT_ROUTING_ENABLED: &str = "alert_routing.enabled";
pub(in crate::repository) const SETTING_ALERT_ROUTING_PROVIDER: &str = "alert_routing.provider";
pub(in crate::repository) const SETTING_ALERT_ROUTING_MIN_SEVERITY: &str =
    "alert_routing.min_severity";
pub(in crate::repository) const SETTING_ALERT_ROUTING_WEBHOOK_URL: &str =
    "alert_routing.webhook_url";
pub(in crate::repository) const SETTING_ALERT_ROUTING_TIMEOUT_SECONDS: &str =
    "alert_routing.timeout_seconds";

pub(in crate::repository) const WORKSPACE_BACKUP_SETTING_KEYS: &[&str] = &[
    SETTING_WATCHDOG_ENABLED,
    SETTING_WATCHDOG_MAX_ATTEMPTS,
    SETTING_WATCHDOG_BASE_DELAY_SECONDS,
    SETTING_WATCHDOG_MAX_DELAY_SECONDS,
    SETTING_RUNTIME_UPGRADE_ENABLED,
    SETTING_RUNTIME_UPGRADE_CATALOG_PROFILE_ID,
    SETTING_RUNTIME_UPGRADE_INTERVAL_MINUTES,
    SETTING_RUNTIME_UPGRADE_REQUIRE_SIGNED_CATALOG,
    SETTING_RUNTIME_UPGRADE_MAX_NODES_PER_RUN,
    SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_ENABLED,
    SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_START_MINUTE_UTC,
    SETTING_RUNTIME_UPGRADE_MAINTENANCE_WINDOW_END_MINUTE_UTC,
    SETTING_RUNTIME_UPGRADE_WAVE_DELAY_MINUTES,
    SETTING_RUNTIME_UPGRADE_LAST_CHECKED_AT_UNIX,
    SETTING_RUNTIME_UPGRADE_LAST_APPLIED_AT_UNIX,
    SETTING_RPC_HEALTH_MONITOR_ENABLED,
    SETTING_RPC_HEALTH_MONITOR_INTERVAL_SECONDS,
    SETTING_REMOTE_FEDERATION_MONITOR_ENABLED,
    SETTING_REMOTE_FEDERATION_MONITOR_INTERVAL_SECONDS,
    SETTING_PRIVATE_NETWORK_SIDECARS_ALLOW_EXTERNAL,
];
