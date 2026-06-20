use super::super::*;

pub(in crate::private_network) fn validate_signer_endpoint(value: &str) -> Result<String> {
    let endpoint = value.trim();
    let parsed = Url::parse(endpoint).context("signer endpoint must be a valid URL")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("signer endpoint must use http or https");
    }
    if parsed.host_str().is_none() {
        anyhow::bail!("signer endpoint must include a host");
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        anyhow::bail!("signer endpoint must not include credentials");
    }
    if parsed.fragment().is_some() {
        anyhow::bail!("signer endpoint must not include a fragment");
    }
    Ok(endpoint.to_string())
}
