use anyhow::Result;

pub fn validate_node_ports(rpc_port: u16, p2p_port: u16, ws_port: Option<u16>) -> Result<()> {
    validate_node_port(rpc_port, "RPC")?;
    validate_node_port(p2p_port, "P2P")?;
    if let Some(ws_port) = ws_port {
        validate_node_port(ws_port, "WebSocket")?;
    }

    if rpc_port == p2p_port {
        anyhow::bail!("RPC and P2P ports must be different");
    }
    if ws_port.is_some_and(|port| port == rpc_port) {
        anyhow::bail!("RPC and WebSocket ports must be different");
    }
    if ws_port.is_some_and(|port| port == p2p_port) {
        anyhow::bail!("P2P and WebSocket ports must be different");
    }
    Ok(())
}

pub fn validate_node_port(port: u16, label: &str) -> Result<()> {
    if port == 0 {
        anyhow::bail!("{label} port must be greater than 0");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/types/ports/tests.rs"]
mod tests;
