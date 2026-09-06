//! Signer endpoint and browser-origin validation.
//!
//! URL validation is part of the custody boundary: cleartext is loopback-only,
//! redirects are disabled by the client, credentials never live in URLs, and
//! the contract prefix remains owned by NeoNexus rather than configuration.

use anyhow::{bail, Context, Result};
use url::{Host, Url};

use super::API_PREFIX;

/// Split a compatibility constructor URL into an origin and reverse-proxy path.
pub(super) fn validated_compatible_base_url(value: &str) -> Result<(String, String, bool, bool)> {
    let parsed = Url::parse(value.trim()).context("is not a valid URL")?;
    let cleartext = match parsed.scheme() {
        "https" => false,
        "http" => true,
        other => bail!("uses {other}://, and the signer contract is served over http or https"),
    };
    if parsed.host_str().is_none() {
        bail!("has no host");
    }
    let loopback = is_loopback_host(&parsed);
    if cleartext && !loopback {
        bail!(
            "uses cleartext HTTP with a non-loopback host; use HTTPS unless the signer is on \
             loopback"
        );
    }
    if has_userinfo(value) {
        bail!("carries URL credentials — a caller token belongs in the Authorization header");
    }
    if parsed.fragment().is_some() {
        bail!("carries a fragment, which never reaches the service");
    }
    if parsed.query().is_some() {
        bail!("carries a query string, which every request would drop");
    }
    let mount_path = parsed.path().trim_end_matches('/');
    if mount_path.contains(API_PREFIX) {
        bail!("already contains the endpoint path; the client appends {API_PREFIX} itself");
    }

    // Preserve IPv6 brackets and URL-parser normalization by removing only the
    // already-validated path from the parser's own serialization.
    let serialized = parsed.as_str();
    let origin = serialized
        .strip_suffix(parsed.path())
        .unwrap_or(serialized)
        .trim_end_matches('/')
        .to_string();
    Ok((origin, mount_path.to_string(), cleartext, loopback))
}

/// Validate the native signer endpoint used by production configuration.
///
/// The Rust service registers `/signer/api/v1/*` at its HTTP root, and workload
/// signatures commit to the exact path-and-query received there. Accepting a
/// configurable mount path would make a reverse proxy rewrite part of the
/// signed message without a protocol for telling this client what it became.
pub(super) fn validated_service_url(value: &str) -> Result<(String, bool, bool)> {
    let parsed = Url::parse(value.trim()).context("is not a valid URL")?;
    let (origin, mount_path, cleartext, loopback) = validated_compatible_base_url(value)?;
    if parsed.path() != "/" || !mount_path.is_empty() || has_non_root_raw_path(value) {
        bail!(
            "carries a path, but the native signer contract is rooted at {API_PREFIX}; \
             configure a plain origin"
        );
    }
    Ok((origin, cleartext, loopback))
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

/// Validate and normalize an exact browser `Origin` value.
pub(super) fn validated_consumer_origin(value: &str) -> Result<String> {
    let parsed = Url::parse(value.trim()).context("is not a valid URL")?;
    match parsed.scheme() {
        "http" | "https" => {}
        other => bail!("uses {other}://, and an Origin must use http or https"),
    }
    if parsed.host_str().is_none() {
        bail!("has no host");
    }
    if has_userinfo(value) {
        bail!("carries URL credentials");
    }
    if parsed.query().is_some() {
        bail!("carries a query string");
    }
    if parsed.fragment().is_some() {
        bail!("carries a fragment");
    }
    if parsed.path() != "/" {
        bail!("carries a path, which is not part of an Origin");
    }
    Ok(parsed
        .as_str()
        .strip_suffix('/')
        .unwrap_or(parsed.as_str())
        .to_string())
}

fn has_userinfo(value: &str) -> bool {
    value
        .trim()
        .split_once("://")
        .map(|(_, remainder)| {
            remainder
                .split(['/', '?', '#'])
                .next()
                .is_some_and(|authority| authority.contains('@'))
        })
        .unwrap_or(false)
}

fn has_non_root_raw_path(value: &str) -> bool {
    value
        .trim()
        .split_once("://")
        .and_then(|(_, remainder)| remainder.find('/').map(|index| &remainder[index..]))
        .map(|path_and_more| {
            let path = path_and_more
                .split(['?', '#'])
                .next()
                .unwrap_or(path_and_more);
            !matches!(path, "" | "/")
        })
        .unwrap_or(false)
}

/// Apply the same caller-id and subject constraints the signer verifies before
/// accepting a workload assertion.
pub(super) fn validated_workload_identity(
    caller_id: String,
    subject: Option<String>,
) -> Result<(String, Option<String>)> {
    let caller_id = caller_id.trim().to_string();
    if caller_id.is_empty()
        || caller_id.len() > 128
        || !caller_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        bail!("workload caller id must be 1..=128 ASCII alphanumeric or `-` characters");
    }
    let subject = subject
        .map(|subject| subject.trim().to_string())
        .filter(|subject| !subject.is_empty());
    if subject
        .as_deref()
        .is_some_and(|subject| subject.len() > 200 || subject.chars().any(char::is_control))
    {
        bail!("workload subject must be at most 200 printable characters");
    }
    Ok((caller_id, subject))
}
