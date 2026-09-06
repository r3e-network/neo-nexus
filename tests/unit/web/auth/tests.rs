use std::{net::IpAddr, time::Duration};

use axum::http::{header, HeaderMap, HeaderValue};

use super::{
    AuthStore, LoginDecision, WebSecurity, LOGIN_BACKOFF_BASE, MAX_LOGIN_PEERS,
    MIN_OPERATOR_TOKEN_BYTES,
};

const TOKEN: &str = "9f1c2b3a4d5e6f708192a3b4c5d6e7f89f1c2b3a4d5e6f708192a3b4c5d6e7f8";

#[test]
fn operator_tokens_are_strong_and_cookie_transport_tracks_deployment() {
    assert!(AuthStore::from_token(&"x".repeat(MIN_OPERATOR_TOKEN_BYTES - 1)).is_err());
    let auth = AuthStore::from_token(TOKEN).expect("strong token");

    let local = auth.session_cookie("session", false);
    assert!(local.contains("HttpOnly"));
    assert!(local.contains("SameSite=Strict"));
    assert!(!local.contains("Secure"));

    let production = auth.session_cookie("session", true);
    assert!(production.contains("; Secure"));
    assert!(auth.clear_cookie(true).contains("; Secure"));
}

#[test]
fn public_origin_is_mandatory_off_loopback_and_controls_browser_provenance() {
    assert!(WebSecurity::resolve("0.0.0.0".parse().unwrap(), None).is_err());
    assert!(
        WebSecurity::resolve("0.0.0.0".parse().unwrap(), Some("http://nexus.example")).is_err()
    );
    let security = WebSecurity::resolve(
        "127.0.0.1".parse().unwrap(),
        Some("https://NEXUS.example:443"),
    )
    .expect("TLS public origin");
    assert_eq!(security.public_origin(), Some("https://nexus.example"));
    assert!(security.secure_cookies());

    let mut headers = HeaderMap::new();
    headers.insert(
        header::ORIGIN,
        HeaderValue::from_static("https://nexus.example"),
    );
    assert!(security.allows_unsafe_request(&headers));

    headers.insert(
        header::ORIGIN,
        HeaderValue::from_static("https://sibling.example"),
    );
    headers.insert(
        header::REFERER,
        HeaderValue::from_static("https://nexus.example/settings"),
    );
    assert!(!security.allows_unsafe_request(&headers));

    headers.insert(header::ORIGIN, HeaderValue::from_bytes(b"\xff").unwrap());
    assert!(
        !security.allows_unsafe_request(&headers),
        "an unreadable Origin must not fall back to a good Referer"
    );

    headers.remove(header::ORIGIN);
    headers.append(
        header::ORIGIN,
        HeaderValue::from_static("https://nexus.example"),
    );
    headers.append(
        header::ORIGIN,
        HeaderValue::from_static("https://nexus.example"),
    );
    assert!(
        !security.allows_unsafe_request(&headers),
        "duplicate Origin fields are ambiguous and must fail closed"
    );

    headers.remove(header::ORIGIN);
    assert!(security.allows_unsafe_request(&headers));
}

#[test]
fn loopback_development_accepts_only_its_exact_dynamic_origin() {
    let security = WebSecurity::loopback_http();
    let mut headers = HeaderMap::new();
    headers.insert(header::HOST, HeaderValue::from_static("127.0.0.1:8123"));
    headers.insert(
        header::ORIGIN,
        HeaderValue::from_static("http://127.0.0.1:8123"),
    );
    assert!(security.allows_unsafe_request(&headers));
    headers.insert(
        header::ORIGIN,
        HeaderValue::from_static("http://127.0.0.1:9999"),
    );
    assert!(!security.allows_unsafe_request(&headers));
}

#[test]
fn login_failures_back_off_per_peer_and_success_resets_after_the_delay() {
    let auth = AuthStore::from_token(TOKEN).expect("strong token");
    let peer: IpAddr = "192.0.2.10".parse().unwrap();
    let other: IpAddr = "192.0.2.11".parse().unwrap();
    let started = std::time::Instant::now();

    for offset in 0..4 {
        assert_eq!(
            auth.authenticate_at(peer, "wrong", started + Duration::from_millis(offset)),
            LoginDecision::Rejected
        );
    }
    assert_eq!(
        auth.authenticate_at(peer, "wrong", started + Duration::from_millis(4)),
        LoginDecision::Throttled {
            retry_after_seconds: LOGIN_BACKOFF_BASE.as_secs()
        }
    );
    assert!(matches!(
        auth.authenticate_at(peer, TOKEN, started + Duration::from_secs(1)),
        LoginDecision::Throttled { .. }
    ));
    assert_eq!(
        auth.authenticate_at(other, TOKEN, started + Duration::from_secs(1)),
        LoginDecision::Accepted
    );
    assert_eq!(
        auth.authenticate_at(
            peer,
            TOKEN,
            started + LOGIN_BACKOFF_BASE + Duration::from_secs(1)
        ),
        LoginDecision::Accepted
    );
}

#[test]
fn login_attempt_memory_is_bounded_under_peer_rotation() {
    let auth = AuthStore::from_token(TOKEN).expect("strong token");
    let started = std::time::Instant::now();
    for index in 0..(MAX_LOGIN_PEERS + 100) {
        let third = u8::try_from((index / 250) % 250).unwrap();
        let fourth = u8::try_from(index % 250 + 1).unwrap();
        let peer = IpAddr::from([198, 51, third, fourth]);
        assert_eq!(
            auth.authenticate_at(peer, "wrong", started + Duration::from_millis(index as u64)),
            LoginDecision::Rejected
        );
    }
    assert!(
        auth.login_attempts.lock().unwrap().peers.len() <= MAX_LOGIN_PEERS,
        "peer rotation grew the limiter without bound"
    );
}
