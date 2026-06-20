use anyhow::{Context, Result};
use url::Url;

use super::validate_https_redirect;

pub(super) fn fetch_https_response(url: Url) -> Result<ureq::Response> {
    const MAX_REDIRECTS: usize = 5;

    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let mut current = url;
    for _ in 0..=MAX_REDIRECTS {
        match agent.get(current.as_str()).call() {
            Ok(response) if (200..300).contains(&response.status()) => return Ok(response),
            Ok(response) if (300..400).contains(&response.status()) => {
                current = next_https_redirect(&current, &response)?;
            }
            Ok(response) => {
                anyhow::bail!(
                    "runtime download failed with HTTP status {} from {}",
                    response.status(),
                    current
                );
            }
            Err(ureq::Error::Status(status, response)) if (300..400).contains(&status) => {
                current = next_https_redirect(&current, &response)?;
            }
            Err(ureq::Error::Status(status, _)) => {
                anyhow::bail!("runtime download failed with HTTP status {status} from {current}");
            }
            Err(error) => {
                anyhow::bail!("failed to download runtime package from {current}: {error}");
            }
        }
    }

    anyhow::bail!("runtime download exceeded {MAX_REDIRECTS} HTTPS redirects");
}

fn next_https_redirect(current: &Url, response: &ureq::Response) -> Result<Url> {
    let location = response
        .header("location")
        .context("runtime download redirect missing location header")?;
    validate_https_redirect(current, location)
}
