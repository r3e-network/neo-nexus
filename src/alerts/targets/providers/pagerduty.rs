use anyhow::{Context, Result};
use url::Url;

pub(in crate::alerts) struct PagerDutyTarget {
    pub(in crate::alerts) routing_key: String,
}

pub(in crate::alerts) fn pagerduty_target(raw: &str) -> Result<PagerDutyTarget> {
    let url = Url::parse(raw).context("PagerDuty Events API URL is invalid")?;
    if url.host_str() != Some("events.pagerduty.com") || url.path() != "/v2/enqueue" {
        anyhow::bail!("PagerDuty alert target must use https://events.pagerduty.com/v2/enqueue");
    }
    let routing_key = url
        .query_pairs()
        .find(|(key, _)| key == "routing_key")
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .context("PagerDuty alert target must include a routing_key query parameter")?;
    Ok(PagerDutyTarget { routing_key })
}
