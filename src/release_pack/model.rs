use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleasePackage {
    pub archive_path: PathBuf,
    pub checksum_path: PathBuf,
    pub manifest_path: PathBuf,
    pub archive_sha256: String,
    pub archive_bytes: u64,
    pub binary_sha256: String,
    pub binary_bytes: u64,
    pub package_id: String,
}

impl ReleasePackage {
    pub fn to_cli_text(&self) -> String {
        format!(
            "release-package: ok\npackage: {}\narchive: {}\narchive-sha256: {}\narchive-bytes: {}\nmanifest: {}\nchecksum: {}\nbinary-sha256: {}\nbinary-bytes: {}\n",
            self.package_id,
            self.archive_path.display(),
            self.archive_sha256,
            self.archive_bytes,
            self.manifest_path.display(),
            self.checksum_path.display(),
            self.binary_sha256,
            self.binary_bytes
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleasePackageVerification {
    pub archive_path: PathBuf,
    pub checksum_path: PathBuf,
    pub manifest_path: PathBuf,
    pub package_id: String,
    pub archive_sha256: String,
    pub archive_bytes: u64,
    pub binary_name: String,
    pub binary_sha256: String,
    pub binary_bytes: u64,
}

impl ReleasePackageVerification {
    pub fn to_cli_text(&self) -> String {
        format!(
            "release-package-verification: ok\npackage: {}\narchive: {}\narchive-sha256: {}\narchive-bytes: {}\nmanifest: {}\nchecksum: {}\nbinary: {}\nbinary-sha256: {}\nbinary-bytes: {}\n",
            self.package_id,
            self.archive_path.display(),
            self.archive_sha256,
            self.archive_bytes,
            self.manifest_path.display(),
            self.checksum_path.display(),
            self.binary_name,
            self.binary_sha256,
            self.binary_bytes
        )
    }
}
