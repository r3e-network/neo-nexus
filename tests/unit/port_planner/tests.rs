use std::{
    collections::BTreeSet,
    io::ErrorKind,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener},
    path::PathBuf,
};

use crate::types::{Network, NodeConfig, NodeStatus, NodeType, StorageEngine};

use super::*;

#[test]
fn port_planner_prefers_requested_block_when_free() -> anyhow::Result<()> {
    let assignment = plan_available_node_ports_with(&[], None, 10_332, true, |_| true)?;

    assert_eq!(
        assignment,
        PortAssignment {
            rpc_port: 10_332,
            p2p_port: 10_333,
            ws_port: Some(10_334)
        }
    );
    Ok(())
}

#[test]
fn port_planner_skips_reserved_and_locally_occupied_ports() -> anyhow::Result<()> {
    let existing = vec![node("alpha", 10_332, 10_333, Some(10_334))];
    let blocked = BTreeSet::from([10_342]);
    let assignment = plan_available_node_ports_with(&existing, None, 10_332, true, |port| {
        !blocked.contains(&port)
    })?;

    assert_eq!(
        assignment,
        PortAssignment {
            rpc_port: 10_352,
            p2p_port: 10_353,
            ws_port: Some(10_354)
        }
    );
    Ok(())
}

#[test]
fn port_planner_ignores_current_node_when_replanning() -> anyhow::Result<()> {
    let existing = vec![
        node("current", 20_332, 20_333, Some(20_334)),
        node("other", 10_332, 10_333, Some(10_334)),
    ];
    let assignment =
        plan_available_node_ports_with(&existing, Some("current"), 20_332, true, |_| true)?;

    assert_eq!(
        assignment,
        PortAssignment {
            rpc_port: 20_332,
            p2p_port: 20_333,
            ws_port: Some(20_334)
        }
    );
    Ok(())
}

#[test]
fn port_planner_can_allocate_without_websocket() -> anyhow::Result<()> {
    let assignment = plan_available_node_ports_with(&[], None, 30_332, false, |_| true)?;

    assert_eq!(
        assignment,
        PortAssignment {
            rpc_port: 30_332,
            p2p_port: 30_333,
            ws_port: None
        }
    );
    Ok(())
}

#[test]
fn localhost_port_probe_detects_ipv4_loopback_listener() -> anyhow::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
    let port = listener.local_addr()?.port();

    assert!(!is_localhost_tcp_port_available(port));
    Ok(())
}

#[test]
fn localhost_port_probe_detects_ipv6_loopback_listener_when_supported() -> anyhow::Result<()> {
    let listener = match TcpListener::bind(SocketAddr::from((Ipv6Addr::LOCALHOST, 0))) {
        Ok(listener) => listener,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::AddrNotAvailable | ErrorKind::Unsupported
            ) =>
        {
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    let port = listener.local_addr()?.port();

    assert!(!is_localhost_tcp_port_available(port));
    Ok(())
}

fn node(id: &str, rpc_port: u16, p2p_port: u16, ws_port: Option<u16>) -> NodeConfig {
    NodeConfig {
        id: id.to_string(),
        name: id.to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("neo-node"),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port,
        p2p_port,
        ws_port,
        status: NodeStatus::Stopped,
        pid: None,
    }
}
