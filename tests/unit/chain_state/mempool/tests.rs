use super::*;
use crate::types::ChainFamily;

#[test]
fn mempool_congestion_labels() {
    assert_eq!(MempoolCongestion::Normal.label(), "normal");
    assert_eq!(MempoolCongestion::Elevated.label(), "elevated");
    assert_eq!(MempoolCongestion::Congested.label(), "congested");
}

#[test]
fn classify_congestion_thresholds() {
    assert_eq!(classify_congestion(0), MempoolCongestion::Normal);
    assert_eq!(classify_congestion(499), MempoolCongestion::Normal);
    assert_eq!(classify_congestion(500), MempoolCongestion::Elevated);
    assert_eq!(classify_congestion(2000), MempoolCongestion::Elevated);
    assert_eq!(classify_congestion(2001), MempoolCongestion::Congested);
    assert_eq!(classify_congestion(10000), MempoolCongestion::Congested);
}

#[test]
fn mempool_telemetry_cli_text_formatting() {
    let telemetry = MempoolTelemetry {
        endpoint: "http://127.0.0.1:10332".to_string(),
        family: ChainFamily::NeoN3,
        total_count: 120,
        verified_count: Some(100),
        unverified_count: Some(20),
        congestion: MempoolCongestion::Normal,
    };

    let text = telemetry.to_cli_text();
    assert!(text.contains("mempool-telemetry: normal"));
    assert!(text.contains("endpoint: http://127.0.0.1:10332"));
    assert!(text.contains("total-transactions: 120"));
    assert!(text.contains("verified-transactions: 100"));
    assert!(text.contains("unverified-transactions: 20"));
}

#[test]
fn congested_mempool_telemetry_cli_text() {
    let telemetry = MempoolTelemetry {
        endpoint: "http://127.0.0.1:8545".to_string(),
        family: ChainFamily::NeoX,
        total_count: 3500,
        verified_count: None,
        unverified_count: None,
        congestion: MempoolCongestion::Congested,
    };

    let text = telemetry.to_cli_text();
    assert!(text.contains("mempool-telemetry: congested"));
    assert!(text.contains("total-transactions: 3500"));
}
