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
mod tests {
    use super::validate_node_ports;

    #[test]
    fn node_port_validation_rejects_zero_and_duplicate_ports() {
        assert!(validate_node_ports(0, 10333, None).is_err());
        assert!(validate_node_ports(10332, 0, None).is_err());
        assert!(validate_node_ports(10332, 10333, Some(0)).is_err());
        assert!(validate_node_ports(10332, 10332, None).is_err());
        assert!(validate_node_ports(10332, 10333, Some(10332)).is_err());
        assert!(validate_node_ports(10332, 10333, Some(10333)).is_err());
    }

    #[test]
    fn node_port_validation_accepts_distinct_nonzero_ports() {
        assert!(validate_node_ports(10332, 10333, None).is_ok());
        assert!(validate_node_ports(10332, 10333, Some(10334)).is_ok());
    }
}
