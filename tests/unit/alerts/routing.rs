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
        kinds: Vec::new(),
        node_ids: Vec::new(),
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

/// A severity floor was the whole of this decision: `event.kind` and
/// `event.node_id` were never read, so one webhook received everything above a
/// global threshold and "page me when a node stalls, not when a plugin is
/// installed" was unsayable.
#[test]
fn a_route_can_be_narrowed_to_the_kinds_worth_waking_someone_for() {
    let policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("https://hooks.example.com/abc".to_string()),
        timeout_seconds: 5,
        kinds: vec![EventKind::NodeHealthChanged],
        node_ids: Vec::new(),
    }
    .normalized();

    let stall = scoped_event(EventKind::NodeHealthChanged, EventSeverity::Critical, None);
    let plugin = scoped_event(EventKind::PluginInstalled, EventSeverity::Critical, None);
    assert!(should_route_alert(&policy, &stall));
    assert!(!should_route_alert(&policy, &plugin));
}

/// "Page on the validator, warn on the observers" is the most ordinary routing
/// rule a node operator has, and it could not be expressed at all.
#[test]
fn a_route_can_be_narrowed_to_particular_nodes() {
    let policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("https://hooks.example.com/abc".to_string()),
        timeout_seconds: 5,
        kinds: Vec::new(),
        node_ids: vec!["node-validator".to_string()],
    }
    .normalized();

    let validator = scoped_event(
        EventKind::NodeExited,
        EventSeverity::Critical,
        Some("node-validator"),
    );
    let observer = scoped_event(
        EventKind::NodeExited,
        EventSeverity::Critical,
        Some("node-observer"),
    );
    assert!(should_route_alert(&policy, &validator));
    assert!(!should_route_alert(&policy, &observer));
}

/// A workspace-wide event has no node. Once a route names specific nodes, it is
/// out of scope: an operator who narrowed to their validator did not thereby
/// ask to hear about backup exports.
#[test]
fn a_node_scoped_route_excludes_workspace_wide_events() {
    let policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Info,
        webhook_url: Some("https://hooks.example.com/abc".to_string()),
        timeout_seconds: 5,
        kinds: Vec::new(),
        node_ids: vec!["node-validator".to_string()],
    }
    .normalized();

    let workspace_wide = scoped_event(EventKind::BackupExported, EventSeverity::Critical, None);
    assert!(!should_route_alert(&policy, &workspace_wide));
}

/// **An empty scope means everything, never nothing.**
///
/// A policy that narrowed to nothing by default would be an alert route that
/// silently delivers no alerts — the exact failure mode alerting exists to
/// avoid, and one nobody notices until an incident.
#[test]
fn an_unscoped_route_still_delivers_everything_above_its_floor() {
    let policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Warning,
        webhook_url: Some("https://hooks.example.com/abc".to_string()),
        timeout_seconds: 5,
        kinds: Vec::new(),
        node_ids: Vec::new(),
    }
    .normalized();

    for kind in [
        EventKind::NodeHealthChanged,
        EventKind::PluginInstalled,
        EventKind::BackupExported,
    ] {
        assert!(should_route_alert(
            &policy,
            &scoped_event(kind, EventSeverity::Critical, Some("node-1"))
        ));
        assert!(should_route_alert(
            &policy,
            &scoped_event(kind, EventSeverity::Critical, None)
        ));
    }
}

/// The scopes compose, and the severity floor still applies underneath them.
#[test]
fn the_scopes_compose_with_each_other_and_with_the_severity_floor() {
    let policy = AlertRoutingPolicy {
        enabled: true,
        provider: AlertProvider::Generic,
        min_severity: EventSeverity::Critical,
        webhook_url: Some("https://hooks.example.com/abc".to_string()),
        timeout_seconds: 5,
        kinds: vec![EventKind::NodeHealthChanged],
        node_ids: vec!["node-validator".to_string()],
    }
    .normalized();

    let wanted = scoped_event(
        EventKind::NodeHealthChanged,
        EventSeverity::Critical,
        Some("node-validator"),
    );
    assert!(should_route_alert(&policy, &wanted));

    // Right kind and node, below the floor.
    let degraded = scoped_event(
        EventKind::NodeHealthChanged,
        EventSeverity::Warning,
        Some("node-validator"),
    );
    assert!(!should_route_alert(&policy, &degraded));
}

/// The policy says what it narrowed to. A route that quietly stopped covering
/// what it was written for is worse than one that was never configured.
#[test]
fn the_policy_describes_its_own_scope() {
    let unscoped = AlertRoutingPolicy::default();
    assert!(!unscoped.is_scoped());
    assert!(unscoped.scope_description().contains("every kind"));
    assert!(unscoped.scope_description().contains("any node"));

    let scoped = AlertRoutingPolicy {
        kinds: vec![EventKind::NodeHealthChanged],
        node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        ..AlertRoutingPolicy::default()
    }
    .normalized();
    assert!(scoped.is_scoped());
    assert!(scoped.scope_description().contains("node-health-changed"));
    assert!(scoped.scope_description().contains("2 selected node(s)"));
}

/// An event of a given kind, severity and node scope, for the routing
/// decision. Named apart from the module's `event(severity)` helper, which
/// predates the scopes and is still what the delivery tests want.
fn scoped_event(kind: EventKind, severity: EventSeverity, node_id: Option<&str>) -> RuntimeEvent {
    RuntimeEvent {
        id: 1,
        occurred_at_unix: 1_770_000_000,
        node_id: node_id.map(str::to_string),
        node_name: node_id.map(str::to_string),
        kind,
        severity,
        message: "something happened".to_string(),
    }
}
