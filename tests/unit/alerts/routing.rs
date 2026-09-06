use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use super::*;

#[test]
fn webhook_redirect_is_rejected_without_contacting_location() -> anyhow::Result<()> {
    let redirect_target = TcpListener::bind("127.0.0.1:0")?;
    redirect_target.set_nonblocking(true)?;
    let redirect_url = format!("http://{}/secret", redirect_target.local_addr()?);
    let origin = TcpListener::bind("127.0.0.1:0")?;
    let origin_url = format!("http://{}/hook", origin.local_addr()?);
    let server = thread::spawn(move || -> std::io::Result<()> {
        let (mut stream, _) = origin.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        read_complete_http_request(&mut stream)?;
        write!(
            stream,
            "HTTP/1.1 302 Found\r\nLocation: {redirect_url}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )?;
        stream.flush()
    });

    let policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Info,
        webhook_url: Some(origin_url),
        timeout_seconds: 2,
    };
    let report = deliver_webhook_alert(&policy, &event(EventSeverity::Critical), "test");
    server.join().expect("origin server thread panicked")?;

    assert_eq!(report.status, AlertDeliveryStatus::Failed);
    assert_eq!(report.http_status, Some(302));
    assert!(report.message.contains("redirect rejected"));
    assert!(matches!(
        redirect_target.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
    Ok(())
}

fn read_complete_http_request(stream: &mut TcpStream) -> io::Result<()> {
    const MAX_REQUEST_BYTES: usize = 64 * 1024;

    let mut request = Vec::new();
    let mut expected_length = None;
    while expected_length.is_none_or(|length| request.len() < length) {
        let mut chunk = [0u8; 4096];
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..count]);
        if request.len() > MAX_REQUEST_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "test webhook request exceeded its bound",
            ));
        }
        if expected_length.is_none() {
            expected_length = expected_http_request_length(&request);
        }
    }
    Ok(())
}

fn expected_http_request_length(request: &[u8]) -> Option<usize> {
    let header_end = request.windows(4).position(|bytes| bytes == b"\r\n\r\n")? + 4;
    let headers = String::from_utf8_lossy(&request[..header_end]);
    let content_length = headers
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    header_end.checked_add(content_length)
}
