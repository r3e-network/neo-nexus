use std::time::Duration;

use super::{Custody, LocalSignerConfig, SignerBackendKind, SignerConfig};

const PUBLIC_KEY: &str = "031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e";

#[test]
fn explicit_backend_kinds_enforce_different_protocols_and_locations() {
    let local_config = || {
        LocalSignerConfig::new("http://127.0.0.1:9991", PUBLIC_KEY, 860_833_102)
            .expect("native local signer config")
    };
    let remote = || {
        SignerConfig::new("https://signer.example", None, Duration::from_secs(1))
            .expect("remote signer config")
    };

    let local = Custody::resolve(
        Some(SignerBackendKind::LocalSigner),
        None,
        Some(local_config()),
        None,
    )
    .expect("local signer accepts loopback");
    assert_eq!(local.kind(), Some(SignerBackendKind::LocalSigner));

    let neo_os = Custody::resolve(
        Some(SignerBackendKind::NeoOsService),
        None,
        None,
        Some(remote()),
    )
    .expect("NeoOS signer accepts HTTPS");
    assert_eq!(neo_os.kind(), Some(SignerBackendKind::NeoOsService));

    assert!(Custody::resolve(
        Some(SignerBackendKind::LocalSigner),
        None,
        None,
        Some(remote()),
    )
    .is_err());
    assert!(Custody::resolve(
        Some(SignerBackendKind::NeoOsService),
        None,
        Some(local_config()),
        None,
    )
    .is_err());
}

#[test]
fn legacy_service_configuration_stays_compatible_without_guessing_local() {
    let config = SignerConfig::new("https://signer.example", None, Duration::from_secs(1)).unwrap();
    let custody = Custody::resolve(None, None, None, Some(config)).expect("legacy service config");
    assert_eq!(custody.kind(), Some(SignerBackendKind::NeoOsService));

    let loopback =
        SignerConfig::new("http://127.0.0.1:8081", None, Duration::from_secs(1)).unwrap();
    assert!(Custody::resolve(None, None, None, Some(loopback)).is_err());
}

#[test]
fn backend_selection_never_falls_back() {
    assert!(Custody::resolve(Some(SignerBackendKind::LocalWallet), None, None, None).is_err());
    assert!(Custody::resolve(Some(SignerBackendKind::LocalSigner), None, None, None).is_err());
    assert!(Custody::resolve(Some(SignerBackendKind::NeoOsService), None, None, None).is_err());
}
