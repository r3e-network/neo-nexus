use anyhow::{Context, Result};
use serde::Deserialize;

use super::{
    super::{RuntimePackageManager, RuntimePlatform},
    RuntimeRelease,
};

#[derive(Deserialize)]
pub(super) struct RuntimeReleaseCatalogDto {
    pub(super) schema_version: u32,
    pub(super) generated_at_unix: Option<u64>,
    pub(super) releases: Vec<RuntimeReleaseDto>,
}

#[derive(Deserialize)]
pub(super) struct RuntimeReleaseDto {
    id: String,
    label: String,
    node_type: String,
    version: String,
    platform: RuntimePlatformDto,
    #[serde(alias = "download_url")]
    url: String,
    #[serde(alias = "download_file_name")]
    file_name: String,
    executable_name: String,
    expected_sha256: String,
    #[serde(default)]
    max_bytes: Option<u64>,
}

#[derive(Deserialize)]
struct RuntimePlatformDto {
    os: String,
    arch: String,
}

impl TryFrom<RuntimeReleaseDto> for RuntimeRelease {
    type Error = anyhow::Error;

    fn try_from(value: RuntimeReleaseDto) -> Result<Self> {
        let release_id = value.id.trim().to_string();
        let node_type = value
            .node_type
            .trim()
            .parse()
            .with_context(|| format!("runtime release {release_id} has invalid node type"))?;

        Ok(Self {
            id: release_id,
            label: value.label.trim().to_string(),
            node_type,
            version: value.version.trim().to_string(),
            platform: RuntimePlatform {
                os: value.platform.os.trim().to_string(),
                arch: value.platform.arch.trim().to_string(),
            },
            url: value.url.trim().to_string(),
            file_name: value.file_name.trim().to_string(),
            executable_name: value.executable_name.trim().to_string(),
            expected_sha256: value.expected_sha256.trim().to_string(),
            max_bytes: value
                .max_bytes
                .unwrap_or(RuntimePackageManager::DEFAULT_DOWNLOAD_MAX_BYTES),
        })
    }
}
