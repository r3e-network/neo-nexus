use std::sync::mpsc::{self, Receiver, Sender};

use crate::{alerts::AlertDeliveryReport, app::workflow::*};

pub(super) struct StartupChannels {
    pub(super) rpc_health_results: Receiver<RpcHealthProbeResult>,
    pub(super) rpc_health_sender: Sender<RpcHealthProbeResult>,
    pub(super) remote_federation_results: Receiver<RemoteFederationProbeResult>,
    pub(super) remote_federation_sender: Sender<RemoteFederationProbeResult>,
    pub(super) alert_delivery_results: Receiver<AlertDeliveryReport>,
    pub(super) alert_delivery_sender: Sender<AlertDeliveryReport>,
}

impl StartupChannels {
    pub(super) fn open() -> Self {
        let (rpc_health_sender, rpc_health_results) = mpsc::channel();
        let (remote_federation_sender, remote_federation_results) = mpsc::channel();
        let (alert_delivery_sender, alert_delivery_results) = mpsc::channel();

        Self {
            rpc_health_results,
            rpc_health_sender,
            remote_federation_results,
            remote_federation_sender,
            alert_delivery_results,
            alert_delivery_sender,
        }
    }
}
