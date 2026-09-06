use super::super::*;
use url::Host;

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

/// Whether an otherwise-valid deployment reference crosses a network boundary
/// without transport encryption.
///
/// Launch-pack endpoints are intentionally protocol-agnostic, so this remains
/// a validation warning instead of being treated as an application signer
/// backend selection. Loopback HTTP is useful for a locally managed sidecar;
/// every non-loopback endpoint should use HTTPS.
pub(in crate::private_network) fn signer_endpoint_is_remote_cleartext(value: &str) -> bool {
    let Ok(parsed) = Url::parse(value.trim()) else {
        return false;
    };
    parsed.scheme() == "http" && !is_loopback_host(&parsed)
}

fn is_loopback_host(url: &Url) -> bool {
    match url.host() {
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => {
            address.is_loopback()
                || address
                    .to_ipv4_mapped()
                    .is_some_and(|mapped| mapped.is_loopback())
        }
        Some(Host::Domain(domain)) => {
            let domain = domain.trim_end_matches('.');
            domain.eq_ignore_ascii_case("localhost")
                || domain
                    .strip_suffix(".localhost")
                    .is_some_and(|prefix| !prefix.is_empty())
        }
        None => false,
    }
}
