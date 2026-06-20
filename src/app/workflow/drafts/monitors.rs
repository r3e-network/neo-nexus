use crate::{federation::RemoteFederationMonitorPolicy, rpc_health::RpcHealthMonitorPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) struct RpcHealthMonitorPolicyDraft {
    pub(in crate::app) enabled: bool,
    pub(in crate::app) interval_seconds: u64,
}

impl RpcHealthMonitorPolicyDraft {
    pub(in crate::app) fn from_policy(policy: RpcHealthMonitorPolicy) -> Self {
        Self {
            enabled: policy.enabled,
            interval_seconds: policy.interval_seconds,
        }
    }

    pub(in crate::app) fn to_policy(self) -> RpcHealthMonitorPolicy {
        RpcHealthMonitorPolicy {
            enabled: self.enabled,
            interval_seconds: self.interval_seconds,
        }
        .normalized()
    }

    pub(in crate::app) fn validation_message(self) -> Option<&'static str> {
        RpcHealthMonitorPolicy {
            enabled: self.enabled,
            interval_seconds: self.interval_seconds,
        }
        .validation_message()
    }

    pub(in crate::app) fn differs_from(self, policy: RpcHealthMonitorPolicy) -> bool {
        self != Self::from_policy(policy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) struct RemoteFederationMonitorPolicyDraft {
    pub(in crate::app) enabled: bool,
    pub(in crate::app) interval_seconds: u64,
}

impl RemoteFederationMonitorPolicyDraft {
    pub(in crate::app) fn from_policy(policy: RemoteFederationMonitorPolicy) -> Self {
        Self {
            enabled: policy.enabled,
            interval_seconds: policy.interval_seconds,
        }
    }

    pub(in crate::app) fn to_policy(self) -> RemoteFederationMonitorPolicy {
        RemoteFederationMonitorPolicy {
            enabled: self.enabled,
            interval_seconds: self.interval_seconds,
        }
        .normalized()
    }

    pub(in crate::app) fn validation_message(self) -> Option<&'static str> {
        RemoteFederationMonitorPolicy {
            enabled: self.enabled,
            interval_seconds: self.interval_seconds,
        }
        .validation_message()
    }

    pub(in crate::app) fn differs_from(self, policy: RemoteFederationMonitorPolicy) -> bool {
        self != Self::from_policy(policy)
    }
}
