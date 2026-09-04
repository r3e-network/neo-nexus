use super::*;
use crate::{
    rpc_health::{RpcHealthReport, RpcIdentityKind, RpcNetworkObservation},
    types::{Network, NewNode, NodeStatus, NodeType},
};

#[test]
fn historical_health_is_not_presented_as_current_after_staleness_stop_or_restart() {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("test.db")).unwrap();
    let mut node = repository
        .create_node(NewNode {
            name: "test".into(),
            node_type: NodeType::NeoRs,
            network: Network::Mainnet,
            binary_path: "unused".into(),
            args: vec![],
            runtime_version: "test".into(),
            storage_engine: NodeType::NeoRs.default_storage_engine(),
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();
    node.status = NodeStatus::Running;
    node.pid = Some(123);
    repository
        .record_rpc_health_at(
            &node,
            &RpcHealthReport {
                endpoint: "http://127.0.0.1:10332".into(),
                status: RpcHealthStatus::Healthy,
                version: Some("test".into()),
                block_count: Some(42),
                syncing: None,
                network: RpcNetworkObservation {
                    identity_kind: Some(RpcIdentityKind::N3NetworkMagic),
                    actual_identity: Some(860_833_102),
                    expected_identity: Some(860_833_102),
                    peer_count: Some(2),
                    peers_expected: true,
                },
                methods: vec![],
            },
            100,
        )
        .unwrap();
    let policy = RpcHealthMonitorPolicy::enabled_default();
    assert!(latest_health_label(&repository, &node, policy, 150).starts_with("healthy"));
    assert!(latest_health_label(&repository, &node, policy, 200).starts_with("stale"));
    node.pid = Some(456);
    assert!(latest_health_label(&repository, &node, policy, 150).contains("current process"));
    node.status = NodeStatus::Stopped;
    node.pid = None;
    assert!(latest_health_label(&repository, &node, policy, 150).starts_with("not running"));
}
