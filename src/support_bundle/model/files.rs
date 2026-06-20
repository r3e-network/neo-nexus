use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupportBundleFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}
