use anyhow::{Context, Result};
use url::Url;

use super::{NewRemoteServerProfile, RemoteServerProfile};

pub fn normalized_remote_input(input: &NewRemoteServerProfile) -> Result<NewRemoteServerProfile> {
    let name = input.name.trim();
    if name.is_empty() {
        anyhow::bail!("remote server name is required");
    }
    if name.chars().count() > 96 {
        anyhow::bail!("remote server name must be 96 characters or fewer");
    }

    let description = input.description.trim();
    if description.chars().count() > 512 {
        anyhow::bail!("remote server description must be 512 characters or fewer");
    }

    Ok(NewRemoteServerProfile {
        name: name.to_string(),
        base_url: normalize_remote_base_url(&input.base_url)?,
        description: description.to_string(),
        enabled: input.enabled,
    })
}

pub fn validate_remote_server_profile(profile: &RemoteServerProfile) -> Result<()> {
    if profile.id.trim().is_empty() {
        anyhow::bail!("remote server id is required");
    }
    normalized_remote_input(&NewRemoteServerProfile {
        name: profile.name.clone(),
        base_url: profile.base_url.clone(),
        description: profile.description.clone(),
        enabled: profile.enabled,
    })?;
    if profile.created_at_unix == 0 {
        anyhow::bail!("remote server creation timestamp is required");
    }
    if profile.updated_at_unix == 0 {
        anyhow::bail!("remote server update timestamp is required");
    }
    Ok(())
}

pub fn normalize_remote_base_url(raw: &str) -> Result<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw == "https://" || raw == "http://" {
        anyhow::bail!("remote server base URL is required");
    }

    let candidate = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };
    let mut url = Url::parse(&candidate).context("remote server base URL is invalid")?;

    if !matches!(url.scheme(), "http" | "https") {
        anyhow::bail!("remote server base URL must use http or https");
    }
    if url.host_str().is_none() {
        anyhow::bail!("remote server base URL must include a host");
    }
    if !url.username().is_empty() || url.password().is_some() {
        anyhow::bail!("remote server base URL must not include credentials");
    }
    if url.query().is_some() || url.fragment().is_some() {
        anyhow::bail!("remote server base URL must not include query strings or fragments");
    }

    let trimmed_path = url.path().trim_end_matches('/').to_string();
    if trimmed_path.is_empty() {
        url.set_path("/");
    } else {
        url.set_path(&trimmed_path);
    }

    Ok(url.as_str().trim_end_matches('/').to_string())
}
