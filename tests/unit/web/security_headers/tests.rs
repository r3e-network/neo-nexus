use super::*;

#[test]
fn content_policy_pins_the_embedded_assets_without_unsafe_inline_script() {
    let policy = content_security_policy();
    assert!(policy.contains("default-src 'none'"));
    assert!(policy.contains("frame-ancestors 'none'"));
    assert!(policy.contains("form-action 'self'"));
    assert!(policy.contains("script-src 'sha256-"));
    assert!(policy.contains("style-src 'sha256-"));
    assert!(!policy.contains("'unsafe-inline'"));
}
