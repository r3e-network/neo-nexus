use super::*;
use crate::{rpc_health::probe_rpc_endpoint_for, types::ChainFamily};
use serde_json::{json, Value};

fn evm_endpoint(sync: Value, height: Value, id: Value) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_http_request(&mut stream).unwrap();
            let result = if request.contains("web3_clientVersion") {
                json!("NeoX/test")
            } else if request.contains("eth_blockNumber") {
                height.clone()
            } else {
                sync.clone()
            };
            let body = json!({"jsonrpc":"2.0","id":id,"result":result}).to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        }
    });
    format!("http://{address}")
}

#[test]
fn evm_false_sync_is_healthy_and_catch_up_or_invalid_sync_is_degraded() {
    for (sync, expected) in [
        (json!(false), RpcHealthStatus::Healthy),
        (
            json!({"startingBlock":"0x0","currentBlock":"0x9","highestBlock":"0xa"}),
            RpcHealthStatus::Degraded,
        ),
        (json!(null), RpcHealthStatus::Degraded),
        (json!({}), RpcHealthStatus::Degraded),
    ] {
        let endpoint = evm_endpoint(sync, json!("0x9"), json!("neonexus-health"));
        let report = probe_rpc_endpoint_for(ChainFamily::NeoX, &endpoint, Duration::from_secs(1));
        assert_eq!(report.status, expected);
        assert_eq!(report.block_count, Some(10));
        assert_eq!(report.methods.len(), 3);
    }
}

#[test]
fn malformed_result_and_wrong_request_id_never_report_healthy() {
    let endpoint = evm_endpoint(json!(false), json!(null), json!("neonexus-health"));
    assert_eq!(
        probe_rpc_endpoint_for(ChainFamily::NeoX, &endpoint, Duration::from_secs(1)).status,
        RpcHealthStatus::Degraded
    );
    let endpoint = evm_endpoint(json!(false), json!("0x9"), json!("another-request"));
    assert_eq!(
        probe_rpc_endpoint_for(ChainFamily::NeoX, &endpoint, Duration::from_secs(1)).status,
        RpcHealthStatus::Unreachable
    );
}
