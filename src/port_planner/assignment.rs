#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortAssignment {
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
}

impl PortAssignment {
    pub fn ports(self) -> Vec<(&'static str, u16)> {
        let mut ports = vec![("RPC", self.rpc_port), ("P2P", self.p2p_port)];
        if let Some(port) = self.ws_port {
            ports.push(("WS", port));
        }
        ports
    }

    pub fn summary(self) -> String {
        match self.ws_port {
            Some(ws_port) => format!(
                "RPC {}, P2P {}, WS {}",
                self.rpc_port, self.p2p_port, ws_port
            ),
            None => format!("RPC {}, P2P {}", self.rpc_port, self.p2p_port),
        }
    }
}
