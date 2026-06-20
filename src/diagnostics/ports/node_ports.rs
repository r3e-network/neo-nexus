use crate::types::NodeConfig;

pub(super) trait NodePorts {
    fn ports(&self) -> Vec<(&'static str, u16)>;
}

impl NodePorts for NodeConfig {
    fn ports(&self) -> Vec<(&'static str, u16)> {
        let mut ports = vec![("RPC", self.rpc_port), ("P2P", self.p2p_port)];
        if let Some(port) = self.ws_port {
            ports.push(("WS", port));
        }
        ports
    }
}
