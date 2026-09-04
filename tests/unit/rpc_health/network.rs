use super::*;
use crate::{
    rpc_health::{probe_node_rpc, RpcIdentityStatus},
    types::{ChainFamily, Network, NodeConfig, NodeStatus, NodeType},
};
use serde_json::{json, Value};

fn node_with_rpc(
    node_type: NodeType,
    network: Network,
    identity: Value,
    peers: Value,
) -> NodeConfig {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let requests = if node_type.family() == ChainFamily::NeoN3 {
            3
        } else {
            5
        };
        for _ in 0..requests {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_http_request(&mut stream).unwrap();
            let request: Value =
                serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
            let result = match request["method"].as_str().unwrap() {
                "getversion" => json!({"useragent":"N3/test", "protocol":{"network":identity}}),
                "getblockcount" => json!(42),
                "getconnectioncount" | "net_peerCount" => peers.clone(),
                "web3_clientVersion" => json!("NeoX/test"),
                "eth_blockNumber" => json!("0x29"),
                "eth_syncing" => json!(false),
                "eth_chainId" => identity.clone(),
                _ => Value::Null,
            };
            let body = json!({"jsonrpc":"2.0", "id":request["id"], "result":result}).to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        }
    });
    NodeConfig {
        id: "rpc-test".into(),
        name: "test".into(),
        node_type,
        network,
        binary_path: "unused".into(),
        args: vec![],
        runtime_version: "test".into(),
        storage_engine: node_type.default_storage_engine(),
        rpc_port: port,
        p2p_port: 30333,
        ws_port: None,
        status: NodeStatus::Running,
        pid: Some(123),
    }
}

#[test]
fn all_five_clients_verify_both_public_networks_and_store_the_peer_count() {
    for node_type in NodeType::ALL {
        for network in [Network::Mainnet, Network::Testnet] {
            let identity = match (node_type.family(), network) {
                (ChainFamily::NeoN3, Network::Mainnet) => 860_833_102,
                (ChainFamily::NeoN3, _) => 894_710_606,
                (ChainFamily::NeoX, Network::Mainnet) => 47_763,
                (ChainFamily::NeoX, _) => 12_227_332,
            };
            let (wire_identity, peers) = if node_type.family() == ChainFamily::NeoN3 {
                (json!(identity), json!(4))
            } else {
                (json!(format!("0x{identity:x}")), json!("0x4"))
            };
            let node = node_with_rpc(node_type, network, wire_identity, peers);
            let report = probe_node_rpc(&node, Duration::from_secs(1));
            assert_eq!(
                report.status,
                RpcHealthStatus::Healthy,
                "{node_type}: {}",
                report.message()
            );
            assert_eq!(report.network.identity_status(), RpcIdentityStatus::Matched);
            assert_eq!(report.network.actual_identity, Some(identity));
            assert_eq!(report.network.expected_identity, Some(identity));
            assert_eq!(report.network.peer_count, Some(4));
            assert!(report.to_cli_text().contains("4 peers"));
        }
    }
}

#[test]
fn wrong_or_unreadable_network_identity_never_reports_healthy() {
    for node_type in [NodeType::NeoCli, NodeType::NeoXReth] {
        let evm = node_type.family() == ChainFamily::NeoX;
        for identity in [if evm { json!("0x1") } else { json!(123) }, json!(null)] {
            let missing = identity.is_null();
            let node = node_with_rpc(
                node_type,
                Network::Mainnet,
                identity,
                if evm { json!("0x2") } else { json!(2) },
            );
            let report = probe_node_rpc(&node, Duration::from_secs(1));
            assert_eq!(report.status, RpcHealthStatus::Degraded);
            assert_eq!(
                report.network.identity_status(),
                if missing {
                    RpcIdentityStatus::Unknown
                } else {
                    RpcIdentityStatus::Mismatch
                }
            );
            assert!(report.message().contains(if missing {
                "unavailable"
            } else {
                "wrong network"
            }));
        }
    }
}

#[test]
fn zero_peers_warns_on_public_networks_but_preserves_isolated_private_nodes() {
    for node_type in [NodeType::NeoGo, NodeType::NeoXGeth] {
        let evm = node_type.family() == ChainFamily::NeoX;
        let identity = if evm {
            json!("0xba93")
        } else {
            json!(860_833_102)
        };
        let peers = if evm { json!("0x0") } else { json!(0) };
        let node = node_with_rpc(node_type, Network::Mainnet, identity.clone(), peers.clone());
        let report = probe_node_rpc(&node, Duration::from_secs(1));
        assert_eq!(report.status, RpcHealthStatus::Degraded);
        assert!(report.message().contains("no connected peers"));
        let node = node_with_rpc(node_type, Network::Private, identity, peers);
        let report = probe_node_rpc(&node, Duration::from_secs(1));
        assert_eq!(report.status, RpcHealthStatus::Healthy);
        assert_eq!(
            report.network.identity_status(),
            RpcIdentityStatus::Unverified
        );
        assert_eq!(report.network.expected_identity, None);
        assert!(report.message().contains("isolated"));
    }
}

#[test]
fn malformed_peer_counts_are_unknown_instead_of_zero() {
    for node_type in [NodeType::NeoRs, NodeType::NeoXReth] {
        let identity = if node_type.family() == ChainFamily::NeoX {
            json!("0xba93")
        } else {
            json!(860_833_102)
        };
        let node = node_with_rpc(node_type, Network::Mainnet, identity, json!("invalid"));
        let report = probe_node_rpc(&node, Duration::from_secs(1));
        assert_eq!(report.status, RpcHealthStatus::Degraded);
        assert_eq!(report.network.peer_count, None);
        assert!(report.message().contains("invalid peer count"));
    }
}

#[test]
fn one_deadline_bounds_the_complete_probe_instead_of_each_extra_method() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_http_request(&mut stream);
        thread::sleep(Duration::from_millis(300));
    });
    let report = probe_rpc_endpoint(&endpoint, Duration::from_millis(50));
    assert_eq!(report.status, RpcHealthStatus::Unreachable);
    assert!(report.methods[1..]
        .iter()
        .all(|method| method.detail.contains("deadline exceeded")));
}
