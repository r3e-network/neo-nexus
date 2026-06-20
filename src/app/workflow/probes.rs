use crate::{
    federation::{RemoteServerProbeReport, RemoteServerProfile},
    rpc_health::RpcHealthReport,
    types::NodeConfig,
};

#[derive(Debug)]
pub(in crate::app) struct RpcHealthProbeResult {
    pub(in crate::app) node: NodeConfig,
    pub(in crate::app) report: RpcHealthReport,
}

#[derive(Debug)]
pub(in crate::app) struct RemoteFederationProbeResult {
    pub(in crate::app) profile: RemoteServerProfile,
    pub(in crate::app) report: Result<RemoteServerProbeReport, String>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::app) enum StartMode {
    Manual,
    Watchdog { attempt: u32 },
}
