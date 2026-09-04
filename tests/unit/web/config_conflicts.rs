use super::*;
use crate::{
    config::ConfigExporter,
    repository::Repository,
    types::{Network, NewNode, NodeType},
    web::auth::AuthStore,
};

#[tokio::test]
async fn config_page_exposes_review_and_stale_safe_resolution_without_secret_values() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "configuration review".into(),
            node_type: NodeType::NeoRs,
            network: Network::Mainnet,
            binary_path: "neo-rs".into(),
            args: Vec::new(),
            runtime_version: "1.0".into(),
            storage_engine: NodeType::NeoRs.default_storage_engine(),
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();
    let state = WebState::new(
        repository,
        directory.path().into(),
        AuthStore::from_token("test"),
    );
    let path =
        ConfigExporter::managed_target_path(directory.path().join("nodes").join(&node.id), &node);
    ConfigExporter::write_node_config_to_path(&path, &node, &[]).unwrap();
    let local = format!(
        "{}\npassword = \"local-secret\"",
        std::fs::read_to_string(&path).unwrap()
    );
    std::fs::write(&path, &local).unwrap();
    let response = review(State(state.clone()), Path(node.id.clone())).await;
    assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
    let conflict = config_conflict(&path).unwrap().unwrap();
    let page =
        super::super::config::config(State(state.clone()), axum::extract::RawQuery(None)).await;
    let body = String::from_utf8(
        axum::body::to_bytes(page.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("Review config"));
    assert!(body.contains("Keep local"));
    assert!(body.contains("Back up local and use generated"));
    assert!(!body.contains("local-secret"));
    let response = resolve(
        State(state.clone()),
        Path(node.id.clone()),
        Form(Resolution {
            file: 0,
            token: "stale-token".into(),
            choice: "generated".into(),
        }),
    )
    .await;
    assert!(response.headers()["location"]
        .to_str()
        .unwrap()
        .contains("expired"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), local);
    resolve(
        State(state.clone()),
        Path(node.id),
        Form(Resolution {
            file: 0,
            token: conflict.token,
            choice: "local".into(),
        }),
    )
    .await;
    assert!(config_conflict(&path).unwrap().is_none());
    ConfigExporter::write_node_config_to_path(&path, &node_from_state(&state), &[]).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), local);
}

fn node_from_state(state: &WebState) -> NodeConfig {
    state.repository.list_nodes().unwrap().remove(0)
}
