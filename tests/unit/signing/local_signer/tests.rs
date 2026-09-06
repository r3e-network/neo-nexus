use super::*;

const PUBLIC_KEY: &str = "031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e";

#[test]
fn accepts_only_local_native_signer_origins() {
    assert!(LocalSignerConfig::new("http://127.0.0.1:9991", PUBLIC_KEY, 42).is_ok());
    assert!(LocalSignerConfig::new("https://[::1]:9991", PUBLIC_KEY, 42).is_ok());
    assert!(LocalSignerConfig::new("vsock://2345:9991", PUBLIC_KEY, 42).is_ok());
    assert!(LocalSignerConfig::new("http://signer.example:9991", PUBLIC_KEY, 42).is_err());
    assert!(LocalSignerConfig::new("http://localhost:9991", PUBLIC_KEY, 42).is_err());
    assert!(LocalSignerConfig::new("http://127.0.0.1:9991/path", PUBLIC_KEY, 42).is_err());
}

#[test]
fn pins_a_canonical_compressed_p256_identity_and_network() {
    let config = LocalSignerConfig::new(
        "http://127.0.0.1:9991",
        PUBLIC_KEY.to_ascii_uppercase(),
        860_833_102,
    )
    .unwrap();
    assert_eq!(config.public_key(), PUBLIC_KEY);
    assert_eq!(config.network_magic(), 860_833_102);
    assert!(LocalSignerConfig::new("http://127.0.0.1:9991", "02", 42).is_err());
    assert!(LocalSignerConfig::new("http://127.0.0.1:9991", PUBLIC_KEY, 0).is_err());
}
