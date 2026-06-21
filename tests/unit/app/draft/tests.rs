use super::*;

#[test]
fn draft_converts_ports_and_args() {
    let draft = NodeDraft {
        name: "node".to_string(),
        binary_path: "/usr/local/bin/neo-go".to_string(),
        args: "--config-file \"/Users/me/Neo Config/protocol.yml\" --mainnet".to_string(),
        runtime_version: "v0.110.0".to_string(),
        rpc_port: "20332".to_string(),
        p2p_port: "20333".to_string(),
        ws_port: String::new(),
        ..Default::default()
    };

    let result = draft.to_new_node();
    assert!(result.is_ok(), "draft should convert valid ports and args");
    let Ok(node) = result else {
        return;
    };

    assert_eq!(
        node.args,
        [
            "--config-file",
            "/Users/me/Neo Config/protocol.yml",
            "--mainnet"
        ]
    );
    assert_eq!(node.runtime_version, "v0.110.0");
    assert_eq!(node.rpc_port, 20332);
    assert_eq!(node.p2p_port, 20333);
    assert_eq!(node.ws_port, None);
}

#[test]
fn draft_uses_safe_port_fallbacks() {
    let draft = NodeDraft {
        rpc_port: "abc".to_string(),
        p2p_port: String::new(),
        ws_port: "not-a-port".to_string(),
        ..Default::default()
    };

    let result = draft.to_new_node();

    assert!(result.is_err());
}

#[test]
fn draft_rejects_duplicate_ports_within_node() {
    let draft = NodeDraft {
        rpc_port: "20332".to_string(),
        p2p_port: "20332".to_string(),
        ws_port: "20333".to_string(),
        ..Default::default()
    };

    let result = draft.to_new_node();

    assert!(result.is_err_and(|error| error.to_string().contains("RPC and P2P")));
}

#[test]
fn draft_rejects_unclosed_quoted_args() {
    let draft = NodeDraft {
        args: "--config \"missing-end".to_string(),
        ..Default::default()
    };

    let result = draft.to_new_node();

    assert!(result.is_err_and(|error| error.to_string().contains("unterminated quote")));
}

#[test]
fn draft_formats_loaded_args_with_quotes_when_needed() {
    let mut draft = NodeDraft::default();
    let node = NodeConfig {
        id: "node-1".to_string(),
        name: "node".to_string(),
        node_type: NodeType::NeoGo,
        network: Network::Testnet,
        binary_path: PathBuf::from("/usr/local/bin/neo-go"),
        args: vec![
            "--config-file".to_string(),
            "/Users/me/Neo Config/protocol.yml".to_string(),
            "--label".to_string(),
            "validator \"one\"".to_string(),
        ],
        runtime_version: "latest".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
        status: crate::types::NodeStatus::Stopped,
        pid: None,
    };

    draft.load_from_node(&node);

    assert_eq!(
        draft.args,
        "--config-file \"/Users/me/Neo Config/protocol.yml\" --label \"validator \\\"one\\\"\""
    );
}
