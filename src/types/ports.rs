use anyhow::Result;

/// The lowest port number a node may be told to listen on.
///
/// Zero means "let the workspace assign one", so it is not a port an
/// operator can type — the range starts at one.
pub const MIN_NODE_PORT: u16 = 1;

/// The highest port number a node may be told to listen on.
pub const MAX_NODE_PORT: u16 = 65_535;

pub fn validate_node_ports(rpc_port: u16, p2p_port: u16, ws_port: Option<u16>) -> Result<()> {
    if rpc_port > 0 {
        validate_node_port(rpc_port, "RPC")?;
    }
    validate_node_port(p2p_port, "P2P")?;
    if let Some(ws_port) = ws_port {
        validate_node_port(ws_port, "WebSocket")?;
    }

    if rpc_port > 0 && rpc_port == p2p_port {
        anyhow::bail!("RPC and P2P ports must be different");
    }
    if rpc_port > 0 && ws_port.is_some_and(|port| port == rpc_port) {
        anyhow::bail!("RPC and WebSocket ports must be different");
    }
    if ws_port.is_some_and(|port| port == p2p_port) {
        anyhow::bail!("P2P and WebSocket ports must be different");
    }
    Ok(())
}

pub fn validate_node_port(port: u16, label: &str) -> Result<()> {
    // Reading the range from the constants keeps the rule enforced here and
    // the `min`/`max` hints the browser is given from ever drifting apart.
    if port < MIN_NODE_PORT {
        anyhow::bail!("{label} port must be greater than 0");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/types/ports/tests.rs"]
mod tests;
