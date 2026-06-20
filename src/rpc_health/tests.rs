use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

use anyhow::Result;

use super::{probe_rpc_endpoint, RpcHealthStatus};

#[test]
fn rpc_health_reports_healthy_node() -> Result<()> {
    let endpoint = spawn_rpc_server(ServerMode::Healthy)?;

    let report = probe_rpc_endpoint(&endpoint, Duration::from_secs(1));

    assert_eq!(report.status, RpcHealthStatus::Healthy);
    assert_eq!(report.block_count, Some(42));
    assert!(matches!(report.version.as_deref(), Some("neo-rs-test")));
    Ok(())
}

#[test]
fn rpc_health_reports_degraded_node() -> Result<()> {
    let endpoint = spawn_rpc_server(ServerMode::BlockCountError)?;

    let report = probe_rpc_endpoint(&endpoint, Duration::from_secs(1));

    assert_eq!(report.status, RpcHealthStatus::Degraded);
    assert!(report.version.is_some());
    assert!(report.block_count.is_none());
    Ok(())
}

#[test]
fn rpc_health_reports_unreachable_endpoint() {
    let report = probe_rpc_endpoint("127.0.0.1:1", Duration::from_millis(80));

    assert_eq!(report.status, RpcHealthStatus::Unreachable);
    assert_eq!(report.endpoint, "http://127.0.0.1:1");
    assert!(report.methods.iter().all(|method| !method.ok));
}

#[derive(Clone, Copy)]
enum ServerMode {
    Healthy,
    BlockCountError,
}

fn spawn_rpc_server(mode: ServerMode) -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    thread::spawn(move || {
        for _ in 0..2 {
            let Ok((mut stream, _peer)) = listener.accept() else {
                return;
            };
            let Ok(request) = read_http_request(&mut stream) else {
                return;
            };
            let body = if request.contains("getversion") {
                r#"{"jsonrpc":"2.0","id":"neonexus-health","result":{"useragent":"neo-rs-test"}}"#
                    .to_string()
            } else if matches!(mode, ServerMode::BlockCountError) {
                r#"{"jsonrpc":"2.0","id":"neonexus-health","error":{"code":-1,"message":"not ready"}}"#
                    .to_string()
            } else {
                r#"{"jsonrpc":"2.0","id":"neonexus-health","result":42}"#.to_string()
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    Ok(format!("http://{address}"))
}

fn read_http_request(stream: &mut std::net::TcpStream) -> Result<String> {
    let mut reader = BufReader::new(stream);
    let mut request = String::new();
    let mut content_length = 0_usize;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if let Some(length) = line
            .strip_prefix("Content-Length:")
            .or_else(|| line.strip_prefix("content-length:"))
            .and_then(|value| value.trim().parse::<usize>().ok())
        {
            content_length = length;
        }
        request.push_str(&line);
        if line == "\r\n" {
            break;
        }
    }

    if content_length > 0 {
        let mut body = vec![0_u8; content_length];
        std::io::Read::read_exact(&mut reader, &mut body)?;
        request.push_str(&String::from_utf8_lossy(&body));
    }

    Ok(request)
}
