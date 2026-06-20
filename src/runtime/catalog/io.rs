use std::{fs::File, io::Read, path::PathBuf};

use anyhow::{Context, Result};
use url::Url;

use super::super::fetch_https_response;

pub(super) enum RuntimeCatalogSource {
    Https(Url),
    File(PathBuf),
}

impl RuntimeCatalogSource {
    pub(super) fn parse(value: &str, label: &str) -> Result<Self> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            anyhow::bail!("{label} is required");
        }
        if is_https_source(trimmed) {
            let url = Url::parse(trimmed).with_context(|| format!("{label} URL is invalid"))?;
            if url.host_str().is_none() {
                anyhow::bail!("{label} URL must include a host");
            }
            return Ok(Self::Https(url));
        }
        if trimmed.contains("://") {
            anyhow::bail!("{label} must use HTTPS or a local file path");
        }

        Ok(Self::File(PathBuf::from(trimmed)))
    }
}

pub(super) fn read_catalog_source_bytes(
    source: RuntimeCatalogSource,
    max_bytes: u64,
    label: &str,
) -> Result<Vec<u8>> {
    match source {
        RuntimeCatalogSource::Https(url) => {
            let response = fetch_https_response(url)?;
            if let Some(content_length) = response.header("content-length") {
                let length = content_length
                    .parse::<u64>()
                    .with_context(|| format!("invalid content-length header from {label}"))?;
                if length > max_bytes {
                    anyhow::bail!("{label} is too large: {length} bytes exceeds limit {max_bytes}");
                }
            }
            read_limited_bytes(response.into_reader(), max_bytes, label)
        }
        RuntimeCatalogSource::File(path) => {
            let file = File::open(&path)
                .with_context(|| format!("failed to open {label} {}", path.display()))?;
            read_limited_bytes(file, max_bytes, label)
        }
    }
}

pub(super) fn is_https_source(value: &str) -> bool {
    value.trim().starts_with("https://")
}

fn read_limited_bytes(mut reader: impl Read, max_bytes: u64, label: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 16 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .with_context(|| format!("failed to read {label}"))?;
        if read == 0 {
            break;
        }
        if bytes.len().saturating_add(read) as u64 > max_bytes {
            anyhow::bail!("{label} exceeded size limit {max_bytes} bytes");
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    Ok(bytes)
}
