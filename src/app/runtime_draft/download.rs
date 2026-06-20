use anyhow::Result;

use crate::runtime::RuntimeDownloadRequest;

use super::{RuntimePackageDraft, BYTES_PER_MIB};

impl RuntimePackageDraft {
    pub(in crate::app) fn to_download_request(&self) -> Result<RuntimeDownloadRequest> {
        let file_name = if self.download_file_name.trim().is_empty() {
            file_name_from_url(&self.download_url)?
        } else {
            self.download_file_name.trim().to_string()
        };
        Ok(RuntimeDownloadRequest {
            url: self.download_url.trim().to_string(),
            file_name,
            expected_sha256: self.expected_sha256.trim().to_string(),
            max_bytes: self.download_max_mib.saturating_mul(BYTES_PER_MIB),
        })
    }
}

fn file_name_from_url(value: &str) -> Result<String> {
    let url = url::Url::parse(value.trim())?;
    let file_name = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|segment| !segment.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("runtime download file name is required"))?;
    Ok(file_name.to_string())
}
