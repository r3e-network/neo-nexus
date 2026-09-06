//! Shared HTTP and signer-envelope transport.
//!
//! This is the only module that constructs signer URLs or interprets signer
//! response envelopes. Authentication header construction lives beside it in
//! `authentication`, split into configured-admin and public-relay flows.

use std::io::Read;

use anyhow::{anyhow, bail, Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use zeroize::Zeroizing;

use super::{
    authentication::{
        apply_configured_credentials, apply_forwarded_credentials, ForwardedCredentials,
    },
    SignerClient,
};
use crate::signer_client::{CallerToken, Outcome, Refusal};

const MAX_ID_LENGTH: usize = 128;
pub(crate) const MAX_REQUEST_BODY_BYTES: usize = 256 * 1024;
// The largest current endpoint is a bounded audit listing. One MiB leaves
// generous contract headroom without letting a faulty or hostile upstream
// make the synchronous relay retain an arbitrary response. Some successful
// admin responses contain newly issued credentials, so the buffer is zeroized.
pub(super) const MAX_RESPONSE_BODY_BYTES: usize = 1024 * 1024;

impl SignerClient {
    pub(super) fn get<T>(
        &self,
        credentials: &CallerToken<'_>,
        path: &str,
        query: Option<&str>,
    ) -> Result<Outcome<T>>
    where
        T: DeserializeOwned,
    {
        self.request(credentials, "GET", path, query, None)
    }

    pub(super) fn post_json<B, T>(
        &self,
        credentials: &CallerToken<'_>,
        path: &str,
        body: &B,
    ) -> Result<Outcome<T>>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        let body = to_json(body)?;
        self.request(credentials, "POST", path, None, Some(body.as_bytes()))
    }

    pub(super) fn request<T>(
        &self,
        credentials: &CallerToken<'_>,
        method: &str,
        path: &str,
        query: Option<&str>,
        body: Option<&[u8]>,
    ) -> Result<Outcome<T>>
    where
        T: DeserializeOwned,
    {
        self.ensure_transport_allowed()?;
        let url = self.config.url(path, query);
        let route = self.config.route(path, query);
        let request = self.agent.request(method, &url).set(
            "User-Agent",
            concat!("NeoNexus/", env!("CARGO_PKG_VERSION")),
        );
        let request = apply_configured_credentials(
            request,
            credentials,
            method,
            &route,
            self.config.workload_audience(),
            body.unwrap_or_default(),
        )?;
        self.send(request, method, path, query, body, &url)
    }

    /// Forward a public caller's exact body and authentication inputs.
    pub(crate) fn relay_raw<T>(
        &self,
        credentials: &ForwardedCredentials<'_>,
        method: &str,
        path: &str,
        query: Option<&str>,
        body: Option<&[u8]>,
    ) -> Result<Outcome<T>>
    where
        T: DeserializeOwned,
    {
        self.ensure_transport_allowed()?;
        let url = self.config.url(path, query);
        let request = self.agent.request(method, &url).set(
            "User-Agent",
            concat!("NeoNexus/", env!("CARGO_PKG_VERSION")),
        );
        let request = apply_forwarded_credentials(request, credentials);
        self.send(request, method, path, query, body, &url)
    }

    fn ensure_transport_allowed(&self) -> Result<()> {
        if self.config.uses_cleartext() && !self.config.allows_insecure_loopback_http_for_test() {
            bail!(
                "refusing cleartext signer transport; configure authenticated HTTPS (the HTTP loopback seam is test-only)"
            );
        }
        Ok(())
    }

    fn send<T>(
        &self,
        request: ureq::Request,
        method: &str,
        path: &str,
        query: Option<&str>,
        body: Option<&[u8]>,
        url: &str,
    ) -> Result<Outcome<T>>
    where
        T: DeserializeOwned,
    {
        let response = match body {
            Some(bytes) => request
                .set("Content-Type", "application/json")
                .send_bytes(bytes),
            None => request.call(),
        };
        let (status, body) = match response {
            Ok(response) => (response.status(), read_body(response)?),
            Err(ureq::Error::Status(status, response)) => (status, read_body(response)?),
            Err(ureq::Error::Transport(transport)) => {
                return Err(anyhow!(transport)
                    .context(format!("could not reach the signer service at {url}")))
            }
        };
        let parsed = match std::str::from_utf8(&body) {
            Ok(text) => parse_reply(status, text, method, path),
            Err(_) => Err(anyhow!(
                "answered {method} {path} with HTTP {status} and {} bytes that are not UTF-8",
                body.len()
            )),
        };
        parsed.with_context(|| {
            format!(
                "the signer service answered {method} {}",
                path_with_query(path, query)
            )
        })
    }
}

pub(super) fn parse_reply<T>(
    status: u16,
    text: &str,
    method: &str,
    path: &str,
) -> Result<Outcome<T>>
where
    T: DeserializeOwned,
{
    let mut value: serde_json::Value = serde_json::from_str(text).map_err(|_| {
        anyhow!(
            "answered {method} {path} with HTTP {status} and {} bytes that are not JSON",
            text.len()
        )
    })?;
    if value.get("allowed").and_then(serde_json::Value::as_bool) == Some(false) {
        let code = value.get("code").and_then(serde_json::Value::as_str);
        let message = value.get("message").and_then(serde_json::Value::as_str);
        return match (code, message) {
            (Some(code), Some(message)) => Ok(Outcome::Refused(Refusal {
                code: code.to_string(),
                message: message.to_string(),
                status,
            })),
            _ => Err(anyhow!(
                "answered HTTP {status} with a refusal that has no code"
            )),
        };
    }
    if !(200..=299).contains(&status) {
        return Err(anyhow!(
            "answered HTTP {status} with an allowed answer, which §5 reserves for 2xx"
        ));
    }
    if let Some(object) = value.as_object_mut() {
        object.remove("allowed");
    }
    serde_json::from_value::<T>(value)
        .map(Outcome::Allowed)
        .map_err(|error| anyhow!("its allowed answer did not match the contract: {error}"))
}

pub(super) fn to_json<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    serde_json::to_string(value)
        .context("this client built a request body its own types cannot encode")
}

fn read_body(response: ureq::Response) -> Result<Zeroizing<Vec<u8>>> {
    let declared_length = response
        .header("Content-Length")
        .and_then(|value| value.trim().parse::<u64>().ok());
    read_limited_body(response.into_reader(), declared_length)
}

pub(super) fn read_limited_body(
    reader: impl Read,
    declared_length: Option<u64>,
) -> Result<Zeroizing<Vec<u8>>> {
    let limit = MAX_RESPONSE_BODY_BYTES as u64;
    if declared_length.is_some_and(|length| length > limit) {
        bail!("the signer response declared more than the {MAX_RESPONSE_BODY_BYTES}-byte limit");
    }

    let capacity = declared_length.unwrap_or(8 * 1024).min(limit) as usize;
    let mut body = Zeroizing::new(Vec::with_capacity(capacity));
    reader
        .take(limit + 1)
        .read_to_end(&mut body)
        .context("the signer response body could not be read")?;
    if body.len() > MAX_RESPONSE_BODY_BYTES {
        bail!("the signer response exceeded the {MAX_RESPONSE_BODY_BYTES}-byte limit");
    }
    Ok(body)
}

pub(super) fn map_outcome<T, U>(outcome: Outcome<T>, f: impl FnOnce(T) -> U) -> Outcome<U> {
    match outcome {
        Outcome::Allowed(payload) => Outcome::Allowed(f(payload)),
        Outcome::Refused(refusal) => Outcome::Refused(refusal),
    }
}

fn path_with_query(path: &str, query: Option<&str>) -> String {
    match query {
        Some(query) => format!("{path}?{query}"),
        None => path.to_string(),
    }
}

pub(super) fn checked_id(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("an empty id cannot name a key or a caller");
    }
    if value.len() > MAX_ID_LENGTH {
        bail!("an id over {MAX_ID_LENGTH} characters is not one this service issues");
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        bail!("an id must be alphanumeric, `-` or `_`")
    }
    Ok(value.to_string())
}
