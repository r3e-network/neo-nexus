use super::*;

#[test]
fn remote_federation_urls_normalize_and_reject_unsafe_shapes() {
    assert_eq!(
        normalize_remote_base_url("nexus.example.com/ops/").unwrap(),
        "https://nexus.example.com/ops"
    );
    assert_eq!(
        normalize_remote_base_url("http://localhost:9090/").unwrap(),
        "http://localhost:9090"
    );
    assert!(normalize_remote_base_url("ftp://nexus.example.com").is_err());
    assert!(normalize_remote_base_url("https://user:pass@nexus.example.com").is_err());
    assert!(normalize_remote_base_url("https://nexus.example.com?token=secret").is_err());
    assert!(normalize_remote_base_url("https://nexus.example.com/#status").is_err());
}

#[test]
fn remote_public_status_parser_accepts_nested_and_string_counts() {
    let parsed = parse_public_status(&serde_json::json!({
        "status": {
            "totalNodes": "7",
            "runningNodes": 5,
            "syncingNodes": 1,
            "errorNodes": 1,
            "totalBlocks": "123456",
            "totalPeers": 32
        }
    }));

    assert_eq!(parsed.total_nodes, Some(7));
    assert_eq!(parsed.running_nodes, Some(5));
    assert_eq!(parsed.syncing_nodes, Some(1));
    assert_eq!(parsed.error_nodes, Some(1));
    assert_eq!(parsed.total_blocks, Some(123456));
    assert_eq!(parsed.total_peers, Some(32));
}
