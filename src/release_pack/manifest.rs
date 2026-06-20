use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(super) struct ReleaseArchiveManifest<'a> {
    pub(super) schema_version: u32,
    pub(super) package_id: &'a str,
    pub(super) application: &'a str,
    pub(super) version: &'a str,
    pub(super) os: &'a str,
    pub(super) arch: &'a str,
    pub(super) binary_name: &'a str,
    pub(super) binary_sha256: &'a str,
    pub(super) binary_bytes: u64,
}

#[derive(Debug, Deserialize)]
pub(super) struct ReleaseArchiveManifestOwned {
    pub(super) schema_version: u32,
    pub(super) package_id: String,
    pub(super) application: String,
    pub(super) version: String,
    pub(super) os: String,
    pub(super) arch: String,
    pub(super) binary_name: String,
    pub(super) binary_sha256: String,
    pub(super) binary_bytes: u64,
}

#[derive(Debug, Serialize)]
pub(super) struct ReleaseSidecarManifest<'a> {
    pub(super) schema_version: u32,
    pub(super) package_id: &'a str,
    pub(super) application: &'a str,
    pub(super) version: &'a str,
    pub(super) os: &'a str,
    pub(super) arch: &'a str,
    pub(super) archive_file: &'a str,
    pub(super) archive_sha256: &'a str,
    pub(super) archive_bytes: u64,
    pub(super) binary_name: &'a str,
    pub(super) binary_sha256: &'a str,
    pub(super) binary_bytes: u64,
}

#[derive(Debug, Deserialize)]
pub(super) struct ReleaseSidecarManifestOwned {
    pub(super) schema_version: u32,
    pub(super) package_id: String,
    pub(super) application: String,
    pub(super) version: String,
    pub(super) os: String,
    pub(super) arch: String,
    pub(super) archive_file: String,
    pub(super) archive_sha256: String,
    pub(super) archive_bytes: u64,
    pub(super) binary_name: String,
    pub(super) binary_sha256: String,
    pub(super) binary_bytes: u64,
}
