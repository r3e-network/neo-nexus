use anyhow::{Context, Result};
use url::Url;

pub(in crate::alerts) struct DatadogTarget {
    pub(in crate::alerts) endpoint_url: String,
    pub(in crate::alerts) api_key: String,
}

const DATADOG_INTAKE_HOSTS: &[&str] = &[
    "event-management-intake.datadoghq.com",
    "event-management-intake.us3.datadoghq.com",
    "event-management-intake.us5.datadoghq.com",
    "event-management-intake.datadoghq.eu",
    "event-management-intake.ap1.datadoghq.com",
    "event-management-intake.ap2.datadoghq.com",
    "event-management-intake.ddog-gov.com",
];

pub(in crate::alerts) fn datadog_target(raw: &str) -> Result<DatadogTarget> {
    let mut url = Url::parse(raw).context("Datadog Events intake URL is invalid")?;
    let host = url.host_str().unwrap_or_default();
    if !DATADOG_INTAKE_HOSTS.contains(&host) || url.path() != "/api/v2/events" {
        anyhow::bail!(
            "Datadog alert target must use an event-management-intake Datadog site /api/v2/events URL"
        );
    }
    let api_key = url
        .query_pairs()
        .find(|(key, _)| key == "api_key")
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .context("Datadog alert target must include an api_key query parameter")?;
    url.set_query(None);
    Ok(DatadogTarget {
        endpoint_url: url.to_string(),
        api_key,
    })
}
