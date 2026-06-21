use std::{fmt, str::FromStr};

macro_rules! define_event_kinds {
    ($($variant:ident => $label:literal,)+) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum EventKind {
            $($variant,)+
        }

        impl EventKind {
            pub const ALL: &[Self] = &[
                $(Self::$variant,)+
            ];

            pub fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)+
                }
            }
        }

        impl FromStr for EventKind {
            type Err = anyhow::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($label => Ok(Self::$variant),)+
                    other => anyhow::bail!("unsupported event kind: {other}"),
                }
            }
        }
    };
}

define_event_kinds! {
    NodeCreated => "node-created",
    NodeUpdated => "node-updated",
    NodeDeleted => "node-deleted",
    NodeStarted => "node-started",
    NodeRestarted => "node-restarted",
    NodeStopped => "node-stopped",
    NodePortsAssigned => "node-ports-assigned",
    NodeExited => "node-exited",
    NodeStartFailed => "node-start-failed",
    PluginUpdated => "plugin-updated",
    PluginInstalled => "plugin-installed",
    RoleApplied => "role-applied",
    PrivateNetworkMaterialized => "private-network-materialized",
    PrivateNetworkLaunchPackExported => "private-network-launch-pack-exported",
    PrivateNetworkLaunchPackValidated => "private-network-launch-pack-validated",
    PrivateNetworkSignerSidecarStarted => "private-network-signer-sidecar-started",
    PrivateNetworkSignerSidecarStopped => "private-network-signer-sidecar-stopped",
    PrivateNetworkSignerSidecarExited => "private-network-signer-sidecar-exited",
    PrivateNetworkSignerSidecarStartFailed => "private-network-signer-sidecar-start-failed",
    PrivateNetworkSignerSidecarExecutionBlocked => "private-network-signer-sidecar-execution-blocked",
    PrivateNetworkSignerSidecarPolicyUpdated => "private-network-signer-sidecar-policy-updated",
    PrivateNetworkSignerSidecarHealthChecked => "private-network-signer-sidecar-health-checked",
    SnapshotSaved => "snapshot-saved",
    SnapshotVerified => "snapshot-verified",
    SnapshotDownloaded => "snapshot-downloaded",
    SnapshotCached => "snapshot-cached",
    SnapshotApplied => "snapshot-applied",
    ConfigExported => "config-exported",
    ConfigApplied => "config-applied",
    LogCleared => "log-cleared",
    BackupExported => "backup-exported",
    BackupValidated => "backup-validated",
    BackupImported => "backup-imported",
    RuntimeDownloaded => "runtime-downloaded",
    RuntimeInstalled => "runtime-installed",
    RuntimeApplied => "runtime-applied",
    RuntimeRecovered => "runtime-recovered",
    RuntimeStateReconciled => "runtime-state-reconciled",
    RuntimeSmokeTested => "runtime-smoke-tested",
    RuntimeFleetUpgradeRun => "runtime-fleet-upgrade-run",
    RuntimeUpgradePolicyUpdated => "runtime-upgrade-policy-updated",
    RuntimeUpgradePolicyRun => "runtime-upgrade-policy-run",
    NeoWalletProfileImported => "neo-wallet-profile-imported",
    NeoWalletProfileUsed => "neo-wallet-profile-used",
    NeoWalletProfileDeleted => "neo-wallet-profile-deleted",
    RpcHealthChecked => "rpc-health-checked",
    RpcHealthMonitorPolicyUpdated => "rpc-health-monitor-policy-updated",
    AlertRoutingPolicyUpdated => "alert-routing-policy-updated",
    RemoteServerCreated => "remote-server-created",
    RemoteServerUpdated => "remote-server-updated",
    RemoteServerProbed => "remote-server-probed",
    RemoteFederationMonitorPolicyUpdated => "remote-federation-monitor-policy-updated",
    RemoteServerDeleted => "remote-server-deleted",
    WatchdogScheduled => "watchdog-scheduled",
    WatchdogRestarted => "watchdog-restarted",
    WatchdogExhausted => "watchdog-exhausted",
    WatchdogSkipped => "watchdog-skipped",
    WatchdogPolicyUpdated => "watchdog-policy-updated",
    WorkspaceReadinessReportExported => "workspace-readiness-report-exported",
    WorkspaceIntegrityChecked => "workspace-integrity-checked",
    SupportBundleExported => "support-bundle-exported",
    EventJournalExported => "event-journal-exported",
    ReleasePackaged => "release-packaged",
    ReleasePackageVerified => "release-package-verified",
    EventsPruned => "events-pruned",
}

impl fmt::Display for EventKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[cfg(test)]
mod tests;
