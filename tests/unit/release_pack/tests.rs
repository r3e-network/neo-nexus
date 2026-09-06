use std::io::Write;

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};

use super::archive::validate_archive_entries;

use super::{ReleasePackagePlatform, ReleasePackageVerifier, ReleasePackager};

#[test]
fn release_packager_writes_zip_manifest_and_checksum() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary_path = temp_dir.path().join("neo-nexus-test");
    std::fs::write(&binary_path, b"native binary bytes")?;
    let output_dir = temp_dir.path().join("dist");

    let package = ReleasePackager::package_binary(
        &binary_path,
        &output_dir,
        "9.8.7",
        ReleasePackagePlatform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        },
    )?;

    assert_eq!(package.package_id, "neo-nexus-9.8.7-linux-x86_64");
    assert!(package.archive_path.is_file());
    assert!(package.checksum_path.is_file());
    assert!(package.manifest_path.is_file());
    assert_eq!(package.binary_bytes, "native binary bytes".len() as u64);
    assert!(std::fs::read_to_string(&package.checksum_path)?.contains(&package.archive_sha256));

    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&package.manifest_path)?)?;
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(
        manifest["package_id"].as_str(),
        Some(package.package_id.as_str())
    );
    assert_eq!(
        manifest["archive_sha256"].as_str(),
        Some(package.archive_sha256.as_str())
    );
    assert_eq!(
        manifest["binary_sha256"].as_str(),
        Some(package.binary_sha256.as_str())
    );

    let archive_file = std::fs::File::open(&package.archive_path)?;
    let mut archive = zip::ZipArchive::new(archive_file)?;
    let mut names = Vec::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .with_context(|| format!("missing zip entry {index}"))?;
        names.push(entry.name().to_string());
    }
    assert!(names.iter().any(|name| name == "neo-nexus"));
    assert!(names.iter().any(|name| name == "release-manifest.json"));
    Ok(())
}

#[test]
fn release_package_verifier_accepts_dist_manifest_and_archive_inputs() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary_path = temp_dir.path().join("neo-nexus-test");
    std::fs::write(&binary_path, b"native binary bytes")?;
    let output_dir = temp_dir.path().join("dist");

    let package = ReleasePackager::package_binary(
        &binary_path,
        &output_dir,
        "9.8.7",
        ReleasePackagePlatform {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
        },
    )?;

    for input in [&output_dir, &package.manifest_path, &package.archive_path] {
        let verification = ReleasePackageVerifier::verify_integrity(input)?;
        assert_eq!(verification.package_id, package.package_id);
        assert_eq!(verification.archive_sha256, package.archive_sha256);
        assert_eq!(verification.binary_sha256, package.binary_sha256);
        assert_eq!(verification.binary_name, "neo-nexus");
        assert!(verification
            .to_cli_text()
            .contains("release-package-integrity: ok"));
        assert!(!verification.publisher_authenticated);
    }
    Ok(())
}

#[test]
fn release_package_verifier_rejects_tampered_checksum() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary_path = temp_dir.path().join("neo-nexus-test");
    std::fs::write(&binary_path, b"native binary bytes")?;
    let output_dir = temp_dir.path().join("dist");

    let package = ReleasePackager::package_binary(
        &binary_path,
        &output_dir,
        "9.8.7",
        ReleasePackagePlatform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        },
    )?;
    std::fs::write(
        &package.checksum_path,
        format!(
            "0000000000000000000000000000000000000000000000000000000000000000  {}\n",
            package
                .archive_path
                .file_name()
                .context("missing archive name")?
                .to_string_lossy()
        ),
    )?;

    let error = ReleasePackageVerifier::verify_integrity(&package.manifest_path)
        .expect_err("tampered checksum should fail verification");
    assert!(error.to_string().contains("checksum SHA-256 mismatch"));
    Ok(())
}

#[test]
fn authenticated_verifier_requires_and_checks_explicit_ed25519_trust_anchor() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary_path = temp_dir.path().join("neo-nexus-test");
    std::fs::write(&binary_path, b"native binary bytes")?;
    let package = ReleasePackager::package_binary(
        &binary_path,
        temp_dir.path().join("dist"),
        "9.8.7",
        ReleasePackagePlatform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        },
    )?;
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let signature = signing_key.sign(&std::fs::read(&package.manifest_path)?);
    let signature_path = temp_dir.path().join("release-manifest.sig");
    std::fs::write(&signature_path, STANDARD.encode(signature.to_bytes()))?;
    let trusted_key = STANDARD.encode(signing_key.verifying_key().to_bytes());

    let verified = ReleasePackageVerifier::verify_authenticated(
        &package.manifest_path,
        &trusted_key,
        &signature_path,
    )?;
    assert!(verified.publisher_authenticated);
    assert!(verified
        .to_cli_text()
        .contains("publisher-authenticated: yes"));

    let wrong_key = STANDARD.encode(
        SigningKey::from_bytes(&[8u8; 32])
            .verifying_key()
            .to_bytes(),
    );
    assert!(ReleasePackageVerifier::verify_authenticated(
        &package.manifest_path,
        &wrong_key,
        &signature_path,
    )
    .is_err());
    assert!(ReleasePackageVerifier::verify_authenticated(
        &package.manifest_path,
        "",
        &signature_path,
    )
    .is_err());
    assert!(ReleasePackageVerifier::verify_authenticated(
        &package.manifest_path,
        &trusted_key,
        temp_dir.path().join("missing.sig"),
    )
    .is_err());
    Ok(())
}

#[test]
fn integrity_verifier_rejects_noncanonical_or_oversized_manifests() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary_path = temp_dir.path().join("neo-nexus-test");
    std::fs::write(&binary_path, b"native binary bytes")?;
    let package = ReleasePackager::package_binary(
        &binary_path,
        temp_dir.path().join("dist"),
        "9.8.7",
        ReleasePackagePlatform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        },
    )?;

    let original = std::fs::read_to_string(&package.manifest_path)?;
    std::fs::write(&package.manifest_path, format!("{original}\n"))?;
    let error = ReleasePackageVerifier::verify_integrity(&package.manifest_path)
        .expect_err("noncanonical manifest must fail");
    assert!(error.to_string().contains("not in the canonical"));

    std::fs::write(&package.manifest_path, vec![b' '; 64 * 1024 + 1])?;
    let error = ReleasePackageVerifier::verify_integrity(&package.manifest_path)
        .expect_err("oversized manifest must fail before parsing");
    assert!(error.to_string().contains("verification limit"));
    Ok(())
}

#[test]
fn integrity_verifier_rejects_extreme_zip_compression_ratio() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let binary_path = temp_dir.path().join("neo-nexus-test");
    std::fs::write(&binary_path, vec![0u8; 1024 * 1024])?;
    let package = ReleasePackager::package_binary(
        &binary_path,
        temp_dir.path().join("dist"),
        "9.8.7",
        ReleasePackagePlatform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        },
    )?;

    let error = ReleasePackageVerifier::verify_integrity(&package.manifest_path)
        .expect_err("extreme compression ratio must fail");
    assert!(error.to_string().contains("compression ratio"));
    Ok(())
}

#[test]
fn archive_inspection_enforces_count_names_sizes_and_exact_entries() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let too_many = temp_dir.path().join("too-many.zip");
    write_test_zip(
        &too_many,
        &[
            ("release-manifest.json", b"{}"),
            ("neo-nexus", b"a"),
            ("extra", b"b"),
        ],
    )?;
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&too_many)?)?;
    assert!(validate_archive_entries(&mut archive, None)
        .expect_err("extra entry must fail")
        .to_string()
        .contains("exactly 2"));

    let unsafe_name = temp_dir.path().join("unsafe-name.zip");
    write_test_zip(
        &unsafe_name,
        &[("release-manifest.json", b"{}"), ("../neo-nexus", b"a")],
    )?;
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&unsafe_name)?)?;
    assert!(validate_archive_entries(&mut archive, None).is_err());

    let duplicate = temp_dir.path().join("duplicate.zip");
    write_test_zip(
        &duplicate,
        &[
            ("release-manifest.json", b"{}"),
            ("duplicate--entry.json", b"duplicate"),
        ],
    )?;
    rewrite_zip_entry_name(&duplicate, "duplicate--entry.json", "release-manifest.json")?;
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&duplicate)?)?;
    assert!(
        validate_archive_entries(&mut archive, None).is_err(),
        "duplicate names must fail even when the ZIP reader collapses them"
    );

    let wrong_binary = temp_dir.path().join("wrong-binary.zip");
    write_test_zip(
        &wrong_binary,
        &[("release-manifest.json", b"{}"), ("other", b"a")],
    )?;
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&wrong_binary)?)?;
    assert!(validate_archive_entries(&mut archive, Some("neo-nexus"))
        .expect_err("unexpected binary entry must fail")
        .to_string()
        .contains("exactly match"));

    let oversized_manifest = temp_dir.path().join("oversized-manifest.zip");
    let manifest = vec![b'x'; 64 * 1024 + 1];
    write_test_zip(
        &oversized_manifest,
        &[
            ("release-manifest.json", manifest.as_slice()),
            ("neo-nexus", b"a"),
        ],
    )?;
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&oversized_manifest)?)?;
    assert!(validate_archive_entries(&mut archive, None)
        .expect_err("oversized archive manifest must fail")
        .to_string()
        .contains("exceeds 65536 bytes"));
    Ok(())
}

fn write_test_zip(path: &std::path::Path, entries: &[(&str, &[u8])]) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, bytes) in entries {
        zip.start_file(*name, options)?;
        zip.write_all(bytes)?;
    }
    zip.finish()?;
    Ok(())
}

/// `zip` correctly refuses duplicate names while writing. Mutate the two
/// equal-length filename records afterward so the verifier still exercises a
/// hostile archive produced by a less strict tool.
fn rewrite_zip_entry_name(path: &std::path::Path, from: &str, to: &str) -> Result<()> {
    anyhow::ensure!(
        from.len() == to.len(),
        "test ZIP names must have equal length"
    );
    let mut bytes = std::fs::read(path)?;
    let mut replacements = 0;
    let mut offset = 0;
    while offset + from.len() <= bytes.len() {
        if &bytes[offset..offset + from.len()] == from.as_bytes() {
            bytes[offset..offset + to.len()].copy_from_slice(to.as_bytes());
            replacements += 1;
            offset += from.len();
        } else {
            offset += 1;
        }
    }
    anyhow::ensure!(
        replacements >= 2,
        "test ZIP did not contain local and central filename records"
    );
    std::fs::write(path, bytes)?;
    Ok(())
}
