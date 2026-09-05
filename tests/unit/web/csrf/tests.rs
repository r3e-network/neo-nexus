use super::{authority_after_scheme, authority_matches};

#[test]
fn an_origin_authority_is_extracted() {
    assert_eq!(
        authority_after_scheme("https://ops.example.com"),
        "ops.example.com"
    );
    assert_eq!(
        authority_after_scheme("http://127.0.0.1:8080"),
        "127.0.0.1:8080"
    );
}

#[test]
fn a_referer_authority_stops_at_the_path() {
    assert_eq!(
        authority_after_scheme("https://ops.example.com/nodes?flash=x"),
        "ops.example.com"
    );
}

#[test]
fn the_same_authority_matches() {
    assert!(authority_matches(
        "ops.example.com",
        Some("ops.example.com")
    ));
    assert!(authority_matches("127.0.0.1:8080", Some("127.0.0.1:8080")));
}

/// A TLS-terminating proxy makes the browser's Origin carry the default port
/// while the Host header names the upstream without one; both name the same
/// deployment and must match.
#[test]
fn a_default_port_is_not_a_different_origin() {
    assert!(authority_matches(
        "ops.example.com:443",
        Some("ops.example.com")
    ));
    assert!(authority_matches(
        "ops.example.com",
        Some("ops.example.com:443")
    ));
    assert!(authority_matches("127.0.0.1:8080", Some("127.0.0.1")));
}

#[test]
fn a_different_name_or_explicit_port_is_rejected() {
    assert!(!authority_matches(
        "evil.example.com",
        Some("ops.example.com")
    ));
    assert!(!authority_matches(
        "ops.example.com:8443",
        Some("ops.example.com:8080")
    ));
}

/// `Origin: null` is rejected by the middleware itself before comparison; the
/// pure matcher only ever sees real authorities.
#[test]
fn a_null_origin_is_not_a_real_authority() {
    assert_eq!(authority_after_scheme("null"), "null");
}

#[test]
fn no_host_header_means_no_match() {
    assert!(!authority_matches("ops.example.com", None));
}
