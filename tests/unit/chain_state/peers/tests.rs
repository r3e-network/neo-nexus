use super::*;
use crate::types::ChainFamily;

#[test]
fn peer_connectivity_labels() {
    assert_eq!(PeerConnectivity::Healthy.label(), "healthy");
    assert_eq!(PeerConnectivity::Sparse.label(), "sparse");
    assert_eq!(PeerConnectivity::Isolated.label(), "isolated");
}

#[test]
fn classify_connectivity_thresholds() {
    assert_eq!(classify_connectivity(0), PeerConnectivity::Isolated);
    assert_eq!(classify_connectivity(1), PeerConnectivity::Sparse);
    assert_eq!(classify_connectivity(2), PeerConnectivity::Sparse);
    assert_eq!(classify_connectivity(3), PeerConnectivity::Healthy);
    assert_eq!(classify_connectivity(50), PeerConnectivity::Healthy);
}

#[test]
fn parse_hex_quantities() {
    assert_eq!(parse_hex_u64("0x0"), Some(0));
    assert_eq!(parse_hex_u64("0x10"), Some(16));
    assert_eq!(parse_hex_u64("0x1f"), Some(31));
    assert_eq!(parse_hex_u64("ff"), Some(255));
    assert_eq!(parse_hex_u64("invalid"), None);
}

#[test]
fn peer_telemetry_cli_text_formatting() {
    let telemetry = PeerTelemetry {
        endpoint: "http://127.0.0.1:10332".to_string(),
        family: ChainFamily::NeoN3,
        connected_count: 5,
        unconnected_count: Some(10),
        bad_count: Some(1),
        connectivity: PeerConnectivity::Healthy,
        sample_peers: vec![
            PeerEndpoint {
                address: "10.0.0.1".to_string(),
                port: Some(10333),
            },
            PeerEndpoint {
                address: "10.0.0.2".to_string(),
                port: None,
            },
        ],
    };

    let text = telemetry.to_cli_text();
    assert!(text.contains("peer-telemetry: healthy"));
    assert!(text.contains("endpoint: http://127.0.0.1:10332"));
    assert!(text.contains("connected-peers: 5"));
    assert!(text.contains("unconnected-peers: 10"));
    assert!(text.contains("bad-peers: 1"));
    assert!(text.contains("peer: 10.0.0.1:10333"));
    assert!(text.contains("peer: 10.0.0.2"));
}

#[test]
fn isolated_peer_telemetry_cli_text() {
    let telemetry = PeerTelemetry {
        endpoint: "http://127.0.0.1:8545".to_string(),
        family: ChainFamily::NeoX,
        connected_count: 0,
        unconnected_count: None,
        bad_count: None,
        connectivity: PeerConnectivity::Isolated,
        sample_peers: Vec::new(),
    };

    let text = telemetry.to_cli_text();
    assert!(text.contains("peer-telemetry: isolated"));
    assert!(text.contains("connected-peers: 0"));
}
