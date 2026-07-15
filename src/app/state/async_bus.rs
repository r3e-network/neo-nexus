//! Background probe and delivery channels shared by RPC health, federation,
//! and alert webhooks. Keeps async I/O plumbing out of the main UI field list.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

use crate::app::{
    domain::{
        AlertDeliveryReport, AlertRoutingPolicy, RemoteFederationMonitorPolicy,
        RpcHealthMonitorPolicy,
    },
    workflow::{
        AlertRoutingPolicyDraft, RemoteFederationMonitorPolicyDraft,
        RemoteFederationProbeResult, RpcHealthMonitorPolicyDraft, RpcHealthProbeResult,
    },
};

pub(in crate::app) struct AsyncProbeBus {
    pub(in crate::app) alert_routing_policy: AlertRoutingPolicy,
    pub(in crate::app) alert_routing_policy_draft: AlertRoutingPolicyDraft,
    pub(in crate::app) last_alert_preview: Option<crate::app::domain::AlertPreviewReport>,
    pub(in crate::app) last_alert_preview_policy: Option<AlertRoutingPolicy>,
    pub(in crate::app) alert_delivery_pending: usize,
    pub(in crate::app) alert_delivery_results: Receiver<AlertDeliveryReport>,
    pub(in crate::app) alert_delivery_sender: Sender<AlertDeliveryReport>,
    pub(in crate::app) alert_delivery_page: usize,
    pub(in crate::app) alert_delivery_status_filter:
        Option<crate::app::domain::AlertDeliveryStatus>,
    pub(in crate::app) alert_delivery_query: String,
    pub(in crate::app) rpc_health_monitor_policy: RpcHealthMonitorPolicy,
    pub(in crate::app) rpc_health_monitor_policy_draft: RpcHealthMonitorPolicyDraft,
    pub(in crate::app) rpc_health_last_started: BTreeMap<String, Instant>,
    pub(in crate::app) rpc_health_pending: BTreeSet<String>,
    pub(in crate::app) rpc_health_results: Receiver<RpcHealthProbeResult>,
    pub(in crate::app) rpc_health_sender: Sender<RpcHealthProbeResult>,
    pub(in crate::app) remote_federation_monitor_policy: RemoteFederationMonitorPolicy,
    pub(in crate::app) remote_federation_monitor_policy_draft: RemoteFederationMonitorPolicyDraft,
    pub(in crate::app) remote_federation_last_started: BTreeMap<String, Instant>,
    pub(in crate::app) remote_federation_pending: BTreeSet<String>,
    pub(in crate::app) remote_federation_results: Receiver<RemoteFederationProbeResult>,
    pub(in crate::app) remote_federation_sender: Sender<RemoteFederationProbeResult>,
}

impl AsyncProbeBus {
    pub(in crate::app) fn has_in_flight_work(&self) -> bool {
        !self.rpc_health_pending.is_empty()
            || !self.remote_federation_pending.is_empty()
            || self.alert_delivery_pending > 0
    }
}
