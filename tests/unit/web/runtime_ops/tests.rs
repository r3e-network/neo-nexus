//! The guarantees the install flow makes before it touches the network or the
//! disk: platform fit, and that the review says what will actually happen.

use crate::{
    core::{
        node::NodeType,
        runtime::{RuntimeCatalogProfile, RuntimePlatform, RuntimeRelease},
    },
    web::runtime_ops::{review_lines, Staged},
};

fn platform(os: &str, arch: &str) -> RuntimePlatform {
    RuntimePlatform {
        os: os.to_string(),
        arch: arch.to_string(),
    }
}

/// A platform that cannot be this host's, so the refusal path is testable
/// wherever the suite runs.
fn some_other_platform() -> RuntimePlatform {
    let here = RuntimePlatform::current();
    if here.os == "macos" {
        platform("linux", &here.arch)
    } else {
        platform("macos", &here.arch)
    }
}

fn release(platform: RuntimePlatform) -> RuntimeRelease {
    RuntimeRelease {
        id: "rel-1".to_string(),
        label: "neo-go 0.110".to_string(),
        node_type: NodeType::NeoGo,
        version: "0.110.0".to_string(),
        platform,
        url: "https://example.test/neo-go.tar.gz".to_string(),
        file_name: "neo-go.tar.gz".to_string(),
        executable_name: "neo-go".to_string(),
        expected_sha256: "a".repeat(64),
        max_bytes: 64 * 1024 * 1024,
    }
}

fn profile() -> RuntimeCatalogProfile {
    RuntimeCatalogProfile {
        id: "cat-1".to_string(),
        label: "Internal mirror".to_string(),
        source: "https://mirror.example.test/catalog.json".to_string(),
        signature_source: None,
        ed25519_public_key: Some("ed25519-key".to_string()),
        max_bytes: 1024 * 1024,
        enabled: true,
        last_loaded_at_unix: None,
        last_signature_verified: None,
        last_bytes: None,
    }
}

fn staged_for(platform: RuntimePlatform) -> Staged {
    Staged {
        profile: profile(),
        release: release(platform),
    }
}

fn value_of<'a>(lines: &'a [(&'static str, String)], label: &str) -> &'a str {
    lines
        .iter()
        .find(|(row_label, _)| *row_label == label)
        .map(|(_, value)| value.as_str())
        .unwrap_or_else(|| unreachable!("{label} should be part of the review"))
}

#[test]
fn a_release_built_for_this_host_can_proceed() {
    assert!(staged_for(RuntimePlatform::current()).fits_this_host());
}

#[test]
fn a_release_built_for_another_platform_is_refused_before_any_download() {
    assert!(!staged_for(some_other_platform()).fits_this_host());
}

#[test]
fn the_review_shows_the_source_the_package_will_come_from() {
    let lines = review_lines(&staged_for(RuntimePlatform::current()));
    assert_eq!(value_of(&lines, "Catalogue"), "Internal mirror");
    assert_eq!(value_of(&lines, "Source"), profile().source);
}

#[test]
fn the_review_shows_both_platforms_side_by_side() {
    let lines = review_lines(&staged_for(some_other_platform()));
    let host = value_of(&lines, "This host").to_string();
    let package = value_of(&lines, "Package platform").to_string();
    assert_eq!(host, RuntimePlatform::current().to_string());
    assert_ne!(
        host, package,
        "a mismatch must be visible, not hidden behind one column"
    );
    assert!(value_of(&lines, "Expected SHA-256").starts_with('a'));
}

#[test]
fn a_profile_without_a_signer_key_says_so_instead_of_implying_verification() {
    let mut staged = staged_for(RuntimePlatform::current());
    staged.profile.ed25519_public_key = None;
    let lines = review_lines(&staged);
    assert!(
        value_of(&lines, "Signature").contains("no signer key"),
        "the page must not imply a signature check that cannot happen",
    );

    let keyed = review_lines(&staged_for(RuntimePlatform::current()));
    assert!(
        !value_of(&keyed, "Signature").contains("no signer key"),
        "with a key configured the wording must change",
    );
}
