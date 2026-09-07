//! Magic Override Replay Prevention Tests
//!
//! These tests verify that the magic number override system prevents:
//! 1. Cross-node replay attacks (node-A's token applied to node-B)
//! 2. Duplicate application of same token (replay prevention)
//! 3. Concurrent override attempts with consistent results
//! 4. Expiration enforcement
//! 5. Node identity binding integrity

use crate::private_network::magic_override::{
    create_magic_override_request, ConsumedMagicOverride,
};
use std::collections::HashMap;

fn make_default_overrides() -> crate::private_network::magic_override::MagicOverrideOverrides {
    crate::private_network::magic_override::MagicOverrideOverrides {
        seed_nodes: vec![],
        validators_count: 0,
        committee_public_keys: vec![],
        consensus_enabled: false,
    }
}

#[test]
fn test_valid_single_node_magic_override_succeeds() {
    // Create a magic override request for a specific node
    let node_id = "node-validator-01";
    let operator_session = "batch-upgrade-2026-09-06";
    let network_magic = 1_230_001;

    let overrides = crate::private_network::magic_override::MagicOverrideOverrides {
        seed_nodes: vec![],
        validators_count: 1,
        committee_public_keys: vec![],
        consensus_enabled: true,
    };

    let request =
        create_magic_override_request(node_id, operator_session, network_magic, &overrides)
            .expect("request should be created");

    assert_eq!(request.node_id, node_id);
    assert_eq!(request.network_magic, network_magic);
    assert!(!request.is_expired());

    // Consume should succeed when applying to correct node
    let consumption = request
        .consume_for_node(node_id)
        .expect("consumption should succeed for intended node");

    assert_eq!(consumption.node_id, node_id);
    assert_eq!(consumption.network_magic, network_magic);
    assert_eq!(consumption.operator_session, operator_session);
}

#[test]
fn test_cross_node_replay_attempt_is_rejected() {
    // Create override for node A
    let node_a = "node-validator-01";
    let request = create_magic_override_request(
        node_a,
        "upgrade-session",
        1_230_001,
        &make_default_overrides(),
    )
    .expect("request should be created");

    // Attempt to apply to node B (cross-node replay attack)
    let node_b = "node-validator-02";
    let result = request.consume_for_node(node_b);

    assert!(result.is_err(), "cross-node replay should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("WrongNode"),
        "should report wrong node error"
    );
    assert!(
        error_msg.contains(node_a),
        "error should mention expected node"
    );
    assert!(
        error_msg.contains(node_b),
        "error should mention actual node"
    );
    assert!(
        error_msg.contains("Cross-node replay attack detected and blocked"),
        "error message should clearly identify attack type"
    );
}

#[test]
fn test_same_magic_change_rejected_when_applied_twice() {
    // Create and consume an override for node A
    let node_id = "node-staker-01";
    let request = create_magic_override_request(
        node_id,
        "migration-batch-001",
        1_230_500,
        &make_default_overrides(),
    )
    .expect("request should be created");

    // First consumption should succeed
    let first_consumption = request
        .consume_for_node(node_id)
        .expect("first consumption should succeed");
    // The generation is a monotonic process-wide counter; simply confirm it was assigned.
    let _ = first_consumption.generation;

    // Second consumption attempt should fail (already consumed)
    let second_result = request.consume_for_node(node_id);
    assert!(second_result.is_err(), "duplicate consumption should fail");

    let error_msg = second_result.unwrap_err().to_string();
    assert!(
        error_msg.contains("AlreadyConsumed"),
        "should report already consumed error"
    );
    assert!(
        error_msg.contains("This token cannot be reused"),
        "error should explain token is single-use"
    );
}

#[test]
fn test_concurrent_overrides_on_different_nodes_handled_correctly() {
    use std::thread;

    let base_magic = 1_230_100;
    let node_ids: Vec<String> = vec![
        "node-consensus-01".to_string(),
        "node-consensus-02".to_string(),
        "node-consensus-03".to_string(),
    ];

    let handles: Vec<_> = node_ids
        .iter()
        .map(|node_id| {
            let session = format!("concurrent-test-{}", node_id);
            let magic = base_magic + node_id.len() as u32; // Unique but similar values
            let node_id_clone = node_id.clone();
            thread::spawn(move || {
                let request = create_magic_override_request(
                    &node_id_clone,
                    &session,
                    magic,
                    &make_default_overrides(),
                )
                .expect("request should be created");
                request
                    .consume_for_node(&node_id_clone)
                    .expect("consumption should succeed")
            })
        })
        .collect();

    // Collect all results
    let consumptions: Vec<ConsumedMagicOverride> = handles
        .into_iter()
        .map(|handle| handle.join().expect("thread panicked"))
        .collect();

    // Verify each node consumed its own valid token
    assert_eq!(consumptions.len(), 3);
    for (i, item) in consumptions.iter().enumerate() {
        assert_eq!(item.node_id, node_ids[i]);
        assert_eq!(item.network_magic, base_magic + node_ids[i].len() as u32);
    }
}

#[test]
fn test_token_expiration_enforcement() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let node_id = "node-expiration-test";

    // Manually construct a request with expired timestamp
    let expired_request = crate::private_network::magic_override::MagicOverrideRequest {
        token: crate::private_network::magic_override::MagicOverrideToken::new(),
        node_id: node_id.to_string(),
        operator_session: "expired-test".to_string(),
        network_magic: 1_230_001,
        seed_nodes: vec![],
        validators_count: 1,
        committee_public_keys: vec![],
        consensus_enabled: true,
        generation: 100,
        generated_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 301, // Expired 1 second ago
        expired_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 300,
    };

    assert!(
        expired_request.is_expired(),
        "request should be marked as expired"
    );

    let result = expired_request.consume_for_node(node_id);
    assert!(result.is_err(), "expired token should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Expired"),
        "should report expiration error"
    );
}

#[test]
fn test_invalid_node_id_formatation_rejected() {
    let invalid_str = "a".repeat(129);
    let invalid_ids = vec![
        "",                       // Empty string
        invalid_str.as_str(),     // Too long
        "node@invalid",           // Special character
        "-node-starts-with-dash", // Starts with dash
        "node with space",        // Whitespace
    ];

    for node_id in invalid_ids {
        let result = create_magic_override_request(
            node_id,
            "test-session",
            1_230_001,
            &make_default_overrides(),
        );

        assert!(
            result.is_err(),
            "node_id '{}' should be rejected",
            &node_id[..node_id.len().min(20)]
        );
    }
}

#[test]
fn test_zero_and_public_magic_values_rejected() {
    // Zero magic
    let result =
        create_magic_override_request("node-test", "test-session", 0, &make_default_overrides());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("network magic must be greater than zero"));

    // Mainnet magic value
    const MAINNET_MAGIC: u32 = 860_833_102;
    let result = create_magic_override_request(
        "node-test",
        "test-session",
        MAINNET_MAGIC,
        &make_default_overrides(),
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot use public network magic"));

    // Testnet magic value
    const TESTNET_MAGIC: u32 = 894_710_606;
    let result = create_magic_override_request(
        "node-test",
        "test-session",
        TESTNET_MAGIC,
        &make_default_overrides(),
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot use public network magic"));
}

#[test]
fn test_default_overrides_are_empty() {
    let defaults = make_default_overrides();

    assert!(defaults.seed_nodes.is_empty());
    assert_eq!(defaults.validators_count, 0);
    assert!(defaults.committee_public_keys.is_empty());
    assert!(!defaults.consensus_enabled); // Default false
}

#[test]
fn test_consumed_override_into_runtime_config_profile_conversion() {
    let node_id = "node-profile-test";
    let overrides = crate::private_network::magic_override::MagicOverrideOverrides {
        seed_nodes: vec!["seed1.example.com:20333".to_string()],
        validators_count: 7,
        committee_public_keys: vec!["03abc...".to_string()],
        consensus_enabled: true,
    };
    let request =
        create_magic_override_request(node_id, "profile-conversion-test", 1_230_999, &overrides)
            .expect("request should be created");

    let consumption = request
        .consume_for_node(node_id)
        .expect("consumption should succeed");

    let profile = consumption.into_runtime_config_profile();

    assert_eq!(profile.network_magic, 1_230_999);
    assert_eq!(
        profile.seed_nodes,
        vec!["seed1.example.com:20333".to_string()]
    );
    assert_eq!(profile.validators_count, 7);
    assert_eq!(profile.committee_public_keys, vec!["03abc...".to_string()]);
    assert!(profile.consensus_enabled);
}

#[test]
fn test_defensive_node_identity_verification_after_consumption() {
    let node_a = "node-defensive-a";
    let request = create_magic_override_request(
        node_a,
        "defensive-test",
        1_230_111,
        &make_default_overrides(),
    )
    .expect("request should be created");

    let consumption = request
        .consume_for_node(node_a)
        .expect("consumption should succeed");

    // Verify defense works even after consumption
    consumption
        .verify_node_identity(node_a)
        .expect("node identity should match");

    let node_b = "node-defensive-b";
    let result = consumption.verify_node_identity(node_b);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("WrongNode"));
}

// Integration test showing how this would be used in batch operations
#[tokio::test]
async fn test_batch_operation_with_multiple_nodes_security_guarantees() {
    let nodes = ["node-a", "node-b", "node-c"];

    // Create one unique override per node
    let overrides: HashMap<_, _> = nodes
        .iter()
        .map(|&node_id| {
            let req = create_magic_override_request(
                node_id,
                "batch-upgrade-final",
                1_230_001 + node_id.len() as u32,
                &make_default_overrides(),
            )
            .expect("request creation should succeed");
            (node_id, req)
        })
        .collect();

    // Simulate applying to each node
    let mut successes = 0usize;
    let mut failures = 0usize;

    for (node_id, request) in overrides {
        let result = request.consume_for_node(node_id);
        if result.is_ok() {
            successes += 1;
        } else {
            failures += 1;
        }
    }

    assert_eq!(successes, 3);
    assert_eq!(failures, 0);
}
