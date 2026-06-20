use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct SnapshotCatalogDto {
    pub schema_version: u32,
    pub generated_at_unix: Option<u64>,
    pub snapshots: Vec<SnapshotCatalogEntryDto>,
}

#[derive(Deserialize)]
pub(super) struct SnapshotCatalogEntryDto {
    pub id: String,
    pub label: String,
    pub network: String,
    pub node_type: String,
    #[serde(alias = "source_url")]
    pub url: String,
    #[serde(alias = "download_file_name")]
    pub file_name: Option<String>,
    pub expected_sha256: String,
    #[serde(default)]
    pub max_bytes: Option<u64>,
}
