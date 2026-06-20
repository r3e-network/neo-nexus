use anyhow::{Context, Result};

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
        let verification = ReleasePackageVerifier::verify(input)?;
        assert_eq!(verification.package_id, package.package_id);
        assert_eq!(verification.archive_sha256, package.archive_sha256);
        assert_eq!(verification.binary_sha256, package.binary_sha256);
        assert_eq!(verification.binary_name, "neo-nexus");
        assert!(verification
            .to_cli_text()
            .contains("release-package-verification: ok"));
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

    let error = ReleasePackageVerifier::verify(&package.manifest_path)
        .expect_err("tampered checksum should fail verification");
    assert!(error.to_string().contains("checksum SHA-256 mismatch"));
    Ok(())
}
