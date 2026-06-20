use anyhow::{Context, Result};
use url::Url;

pub(in crate::alerts) struct OpsgenieTarget {
    pub(in crate::alerts) endpoint_url: String,
    pub(in crate::alerts) api_key: String,
}

pub(in crate::alerts) fn opsgenie_target(raw: &str) -> Result<OpsgenieTarget> {
    let mut url = Url::parse(raw).context("Opsgenie Alerts API URL is invalid")?;
    if url.host_str() != Some("api.opsgenie.com") || url.path() != "/v2/alerts" {
        anyhow::bail!("Opsgenie alert target must use https://api.opsgenie.com/v2/alerts");
    }
    let api_key = url
        .query_pairs()
        .find(|(key, _)| key == "api_key")
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .context("Opsgenie alert target must include an api_key query parameter")?;
    url.set_query(None);
    Ok(OpsgenieTarget {
        endpoint_url: url.to_string(),
        api_key,
    })
}
