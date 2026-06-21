use super::super::*;

mod export;
mod import;
mod validation;

fn neo_rs_backup_node(name: &str, rpc_port: u16, p2p_port: u16, ws_port: u16) -> NewNode {
    NewNode {
        name: name.to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: "/usr/local/bin/neo-node".into(),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port,
        p2p_port,
        ws_port: Some(ws_port),
    }
}
