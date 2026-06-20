use anyhow::{Context, Result};
use url::Url;

const MAX_WEBHOOK_URL_LEN: usize = 2_048;

pub fn normalized_webhook_url(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Alert webhook URL is required");
    }
    if trimmed.len() > MAX_WEBHOOK_URL_LEN {
        anyhow::bail!("Alert webhook URL is too long");
    }
    let url = Url::parse(trimmed).context("Alert webhook URL is invalid")?;
    let Some(host) = url.host_str() else {
        anyhow::bail!("Alert webhook URL must include a host");
    };
    if !url.username().is_empty() || url.password().is_some() {
        anyhow::bail!("Alert webhook URL must not include credentials");
    }
    if url.fragment().is_some() {
        anyhow::bail!("Alert webhook URL must not include a fragment");
    }
    match url.scheme() {
        "https" => {}
        "http" if is_loopback_host(host) => {}
        _ => anyhow::bail!("Alert webhook URL must use HTTPS unless targeting localhost"),
    }
    Ok(url.to_string().trim_end_matches('/').to_string())
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}
