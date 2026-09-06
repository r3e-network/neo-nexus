//! Authentication for configured control-plane calls and public relay calls.
//!
//! These paths are deliberately separate: configured credentials may create a
//! fresh workload assertion, while relayed credentials are a strict header
//! allowlist and can never fall back to NeoNexus's admin identity.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use crate::signer_client::CallerToken;

const WORKLOAD_PROTOCOL_V2: &str = "neoos-workload-v2";

/// Borrowed authentication headers accepted from a public signer caller.
///
/// There is intentionally no generic header map here: cookies, proxy headers,
/// web sessions, and NeoNexus's admin identity cannot cross this boundary.
pub(crate) struct ForwardedCredentials<'a> {
    pub authorization: Option<&'a str>,
    pub origin: Option<&'a str>,
    pub referer: Option<&'a str>,
    pub workload_protocol: Option<&'a str>,
    pub workload_audience: Option<&'a str>,
    pub workload_caller: Option<&'a str>,
    pub workload_timestamp: Option<&'a str>,
    pub workload_nonce: Option<&'a str>,
    pub workload_signature: Option<&'a str>,
}

pub(super) fn apply_configured_credentials(
    mut request: ureq::Request,
    credentials: &CallerToken<'_>,
    method: &str,
    route: &str,
    audience: &str,
    body: &[u8],
) -> Result<ureq::Request> {
    if let Some(token) = credentials.bearer_value() {
        // Preserve an empty bearer. The signer owns the `missing-token`
        // refusal and its audit semantics.
        request = request.set("Authorization", &format!("Bearer {token}"));
    } else if let Some(workload) = credentials.workload_value() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("the system clock is before the Unix epoch")?
            .as_secs();
        let nonce = uuid::Uuid::new_v4().simple().to_string();
        let digest = Sha256::digest(body);
        let origin = credentials.origin().unwrap_or_default();
        let message = format!(
            "{WORKLOAD_PROTOCOL_V2}\naudience:{audience}\ncaller:{}\nsubject:{}\ntimestamp:{timestamp}\nnonce:{nonce}\nmethod:{}\nroute:{route}\nbody-sha256:{}\norigin:{origin}",
            workload.caller_id(),
            workload.subject().unwrap_or_default(),
            method.to_ascii_uppercase(),
            lowercase_hex(&digest),
        );
        let signature = workload.sign(message.as_bytes());
        request = request
            .set("X-NeoOS-Workload-Protocol", WORKLOAD_PROTOCOL_V2)
            .set("X-NeoOS-Audience", audience)
            .set("X-NeoOS-Caller", workload.caller_id())
            .set("X-NeoOS-Timestamp", &timestamp.to_string())
            .set("X-NeoOS-Nonce", &nonce)
            .set("X-NeoOS-Signature", &lowercase_hex(&signature));
    } else {
        bail!("the signer request has no bearer or workload credential");
    }
    if let Some(origin) = credentials.origin() {
        request = request.set("Origin", origin);
    }
    if let Some(referer) = credentials.referer() {
        request = request.set("Referer", referer);
    }
    Ok(request)
}

pub(super) fn apply_forwarded_credentials(
    mut request: ureq::Request,
    credentials: &ForwardedCredentials<'_>,
) -> ureq::Request {
    for (name, value) in [
        ("Authorization", credentials.authorization),
        ("Origin", credentials.origin),
        ("Referer", credentials.referer),
        ("X-NeoOS-Workload-Protocol", credentials.workload_protocol),
        ("X-NeoOS-Audience", credentials.workload_audience),
        ("X-NeoOS-Caller", credentials.workload_caller),
        ("X-NeoOS-Timestamp", credentials.workload_timestamp),
        ("X-NeoOS-Nonce", credentials.workload_nonce),
        ("X-NeoOS-Signature", credentials.workload_signature),
    ] {
        if let Some(value) = value {
            request = request.set(name, value);
        }
    }
    request
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
