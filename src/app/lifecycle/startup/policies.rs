use super::super::super::*;

use crate::watchdog::RestartPolicy;

pub(super) struct StartupPolicies {
    pub(super) watchdog: RestartPolicy,
    pub(super) runtime_upgrade: RuntimeUpgradePolicy,
    pub(super) rpc_health_monitor: RpcHealthMonitorPolicy,
    pub(super) remote_federation_monitor: RemoteFederationMonitorPolicy,
    pub(super) alert_routing: AlertRoutingPolicy,
    pub(super) allow_external_sidecars: bool,
    pub(super) theme: Theme,
    pub(super) notice: Option<String>,
}

impl StartupPolicies {
    pub(super) fn load(repository: &Repository) -> Self {
        let (watchdog, watchdog_notice) = load_watchdog_policy(repository);
        let (runtime_upgrade, runtime_upgrade_notice) = load_runtime_upgrade_policy(repository);
        let (rpc_health_monitor, rpc_health_monitor_notice) =
            load_rpc_health_monitor_policy(repository);
        let (remote_federation_monitor, remote_federation_monitor_notice) =
            load_remote_federation_monitor_policy(repository);
        let (alert_routing, alert_routing_notice) = load_alert_routing_policy(repository);
        let (allow_external_sidecars, sidecar_policy_notice) =
            load_sidecar_execution_policy(repository);
        let (theme, theme_notice) = load_theme(repository);

        Self {
            watchdog,
            runtime_upgrade,
            rpc_health_monitor,
            remote_federation_monitor,
            alert_routing,
            allow_external_sidecars,
            theme,
            notice: first_notice([
                watchdog_notice,
                runtime_upgrade_notice,
                rpc_health_monitor_notice,
                remote_federation_monitor_notice,
                alert_routing_notice,
                sidecar_policy_notice,
                theme_notice,
            ]),
        }
    }
}

fn load_theme(repository: &Repository) -> (Theme, Option<String>) {
    match repository.load_app_dark_mode() {
        Ok(dark) => (Theme::from_dark_mode(dark), None),
        Err(error) => (
            Theme::default(),
            Some(format!("Using default theme: {error}")),
        ),
    }
}

fn load_watchdog_policy(repository: &Repository) -> (RestartPolicy, Option<String>) {
    match repository.load_watchdog_policy() {
        Ok(policy) => (policy, None),
        Err(error) => (
            default_restart_policy(),
            Some(format!("Using default watchdog policy: {error}")),
        ),
    }
}

fn load_runtime_upgrade_policy(repository: &Repository) -> (RuntimeUpgradePolicy, Option<String>) {
    match repository.load_runtime_upgrade_policy() {
        Ok(policy) => (policy, None),
        Err(error) => (
            RuntimeUpgradePolicy::disabled(),
            Some(format!("Using default runtime upgrade policy: {error}")),
        ),
    }
}

fn load_rpc_health_monitor_policy(
    repository: &Repository,
) -> (RpcHealthMonitorPolicy, Option<String>) {
    match repository.load_rpc_health_monitor_policy() {
        Ok(policy) => (policy, None),
        Err(error) => (
            RpcHealthMonitorPolicy::enabled_default(),
            Some(format!("Using default RPC health monitor policy: {error}")),
        ),
    }
}

fn load_remote_federation_monitor_policy(
    repository: &Repository,
) -> (RemoteFederationMonitorPolicy, Option<String>) {
    match repository.load_remote_federation_monitor_policy() {
        Ok(policy) => (policy, None),
        Err(error) => (
            RemoteFederationMonitorPolicy::enabled_default(),
            Some(format!(
                "Using default remote Federation monitor policy: {error}"
            )),
        ),
    }
}

fn load_alert_routing_policy(repository: &Repository) -> (AlertRoutingPolicy, Option<String>) {
    match repository.load_alert_routing_policy() {
        Ok(policy) => (policy, None),
        Err(error) => (
            AlertRoutingPolicy::default(),
            Some(format!("Using default alert routing policy: {error}")),
        ),
    }
}

fn load_sidecar_execution_policy(repository: &Repository) -> (bool, Option<String>) {
    match repository.load_private_network_allow_external_sidecars() {
        Ok(allow_external) => (allow_external, None),
        Err(error) => (
            false,
            Some(format!("Using default sidecar execution policy: {error}")),
        ),
    }
}

fn first_notice(notices: [Option<String>; 7]) -> Option<String> {
    notices.into_iter().flatten().next()
}
