//! The caller-facing signing API — the only surface in the workbench that is
//! *not* authenticated by the operator session.
//!
//! Routes live outside the session `route_layer` on purpose. A caller is a
//! program holding either a bearer credential or an Ed25519 workload identity;
//! it has no operator cookie, and wrapping these paths in the session middleware
//! would produce a `401` from a layer that knows nothing about the caller, and no
//! audit row.
//!
//! This module is a relay. It authenticates nobody and decides nothing: the three
//! checks a signing request has to pass — the token digest identifies an enabled
//! caller, `Origin` matches the way that caller was declared, the caller is
//! granted the key it names and the key's boundary decides the bytes — all belong
//! to the service, which runs them in the order §5 fixes and writes the audit row
//! for each. What this file adds to that is a transport and nothing else, so:
//!
//! * The caller's own credential is forwarded, not the workbench's. The console's
//!   admin token would be refused on a sign route anyway, and if the routes ever
//!   blurred enough for it to work, every signature in the audit trail would
//!   belong to whoever ran the browser.
//! * A refusal keeps the status the service chose. The old code had a table here
//!   mapping codes to statuses; §5 owns that mapping, it is longer than it looks,
//!   and a second copy could only drift — the drift showing up as a caller
//!   retrying a request that will never succeed.
//! * The request body is bounded but never parsed or reserialized here. A
//!   workload proof commits to SHA-256 of the exact received bytes; changing JSON
//!   whitespace would invalidate it before policy evaluation. Schema and hex
//!   validation therefore remain the service's responsibility and are recorded
//!   in its audit trail.
//!
//! What reaches the caller and what stays in the vault are different texts, by
//! construction: [`RefusalBody`] carries the code and the sentence, never the
//! bounded `detail` that names the amounts and hashes behind a refusal. "Recipient
//! `0x…` is on this key's blacklist" is worth nothing to a caller that is not
//! already the operator, and a great deal to someone probing which accounts are
//! protected. The client this module uses cannot even represent a `detail` outside
//! an audit row — see [`crate::signer_client::Refusal`].

use axum::{
    body::Bytes,
    extract::{rejection::BytesRejection, OriginalUri, Request, State},
    http::{header, HeaderMap, HeaderValue, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::signer_client::{
    Eip191FulfillmentSignature, ForwardedCredentials, KeyPublic, Outcome, RawSignature, Refusal,
    Signature, API_PREFIX, MAX_REQUEST_BODY_BYTES,
};

use super::WebState;

/// Admit public relay work before Axum reads an attacker-controlled body.
pub async fn admit(State(state): State<WebState>, request: Request, next: Next) -> Response {
    let Some(_permit) = state.try_signer_relay_permit() else {
        return overloaded();
    };
    next.run(request).await
}

/// An allowed answer: §5's `"allowed": true` beside the fields the payload
/// already carries.
///
/// Flattened rather than retyped field by field, because the payload types in
/// [`crate::signer_client`] are the mirror of the service's reply bodies, and a
/// second struct here would be a list of fields to forget to update when the
/// contract gains one.
#[derive(Serialize)]
struct Reply<T: Serialize> {
    allowed: bool,
    #[serde(flatten)]
    payload: T,
}

#[derive(Serialize)]
struct RefusalBody {
    allowed: bool,
    code: String,
    message: String,
}

/// The caller's credential, owned.
///
/// Owned rather than borrowed from the [`HeaderMap`] because the client is
/// blocking and runs on a [`tokio::task::spawn_blocking`] thread that outlives
/// this function's borrows.
struct Relayed {
    authorization: Option<String>,
    origin: Option<String>,
    referer: Option<String>,
    workload_protocol: Option<String>,
    workload_audience: Option<String>,
    workload_caller: Option<String>,
    workload_timestamp: Option<String>,
    workload_nonce: Option<String>,
    workload_signature: Option<String>,
}

impl Relayed {
    /// Copy only the signer's nine authentication inputs. Web cookies, proxy
    /// headers, and NeoNexus's own operator credential are structurally absent.
    fn from_headers(headers: &HeaderMap) -> Self {
        Relayed {
            authorization: header(headers, &header::AUTHORIZATION).map(str::to_string),
            origin: header(headers, &header::ORIGIN).map(str::to_string),
            referer: header(headers, &header::REFERER).map(str::to_string),
            workload_protocol: named_header(headers, "x-neoos-workload-protocol")
                .map(str::to_string),
            workload_audience: named_header(headers, "x-neoos-audience").map(str::to_string),
            workload_caller: named_header(headers, "x-neoos-caller").map(str::to_string),
            workload_timestamp: named_header(headers, "x-neoos-timestamp").map(str::to_string),
            workload_nonce: named_header(headers, "x-neoos-nonce").map(str::to_string),
            workload_signature: named_header(headers, "x-neoos-signature").map(str::to_string),
        }
    }

    fn credentials(&self) -> ForwardedCredentials<'_> {
        ForwardedCredentials {
            authorization: self.authorization.as_deref(),
            origin: self.origin.as_deref(),
            referer: self.referer.as_deref(),
            workload_protocol: self.workload_protocol.as_deref(),
            workload_audience: self.workload_audience.as_deref(),
            workload_caller: self.workload_caller.as_deref(),
            workload_timestamp: self.workload_timestamp.as_deref(),
            workload_nonce: self.workload_nonce.as_deref(),
            workload_signature: self.workload_signature.as_deref(),
        }
    }
}

pub async fn sign_transaction(
    State(state): State<WebState>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    relay_post::<Signature>(&state, &headers, uri, body).await
}

pub async fn sign_consensus(
    State(state): State<WebState>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    relay_post::<Signature>(&state, &headers, uri, body).await
}

pub async fn sign_raw(
    State(state): State<WebState>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    relay_post::<RawSignature>(&state, &headers, uri, body).await
}

pub async fn sign_eip191_fulfillment(
    State(state): State<WebState>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    relay_post::<Eip191FulfillmentSignature>(&state, &headers, uri, body).await
}

pub async fn key_info(
    State(state): State<WebState>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Response {
    relay::<KeyPublic>(&state, &headers, uri, "GET", None).await
}

// -- the plumbing -----------------------------------------------------------

/// Ask the service, off the request thread, and answer with what it said.
///
/// The blocking call runs on its own thread for the reason §5 gives the service
/// itself: the wait is for *another process*, possibly one inside an Enclave on
/// another host, and it can take the whole client timeout. Holding an axum worker
/// through that would let one stalled custody service stall the console.
async fn relay_post<T>(
    state: &WebState,
    headers: &HeaderMap,
    uri: Uri,
    body: Result<Bytes, BytesRejection>,
) -> Response
where
    T: serde::de::DeserializeOwned + Serialize + Send + 'static,
{
    let body = match body {
        Ok(body) if body.len() <= MAX_REQUEST_BODY_BYTES => body,
        Ok(_) | Err(_) => return too_large(),
    };
    relay::<T>(state, headers, uri, "POST", Some(body)).await
}

async fn relay<T>(
    state: &WebState,
    headers: &HeaderMap,
    uri: Uri,
    method: &'static str,
    body: Option<Bytes>,
) -> Response
where
    T: serde::de::DeserializeOwned + Serialize + Send + 'static,
{
    let client = match state.signer() {
        Ok(client) => client.clone(),
        Err(error) => return unavailable(&error),
    };
    let caller = Relayed::from_headers(headers);
    let (path, query) = match signer_target(&uri) {
        Ok(target) => target,
        Err(error) => return unavailable(&error),
    };
    match tokio::task::spawn_blocking(move || {
        client.relay_raw::<T>(
            &caller.credentials(),
            method,
            &path,
            query.as_deref(),
            body.as_deref(),
        )
    })
    .await
    {
        Ok(Ok(Outcome::Allowed(payload))) => Json(Reply {
            allowed: true,
            payload,
        })
        .into_response(),
        Ok(Ok(Outcome::Refused(refusal))) => refusal_response(refusal),
        Ok(Err(error)) => unavailable(&error),
        Err(joined) => unavailable(&anyhow::anyhow!(
            "the custody request thread did not finish: {joined}"
        )),
    }
}

fn signer_target(uri: &Uri) -> anyhow::Result<(String, Option<String>)> {
    let path = uri
        .path()
        .strip_prefix(API_PREFIX)
        .filter(|path| path.starts_with('/'))
        .ok_or_else(|| anyhow::anyhow!("the public signer route escaped its contract prefix"))?;
    Ok((path.to_string(), uri.query().map(str::to_string)))
}

fn overloaded() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [(header::RETRY_AFTER, "1")],
        Json(serde_json::json!({
            "allowed": false,
            "code": "signer-relay-busy",
            "message": "the signer relay is at its bounded concurrency limit; retry later",
        })),
    )
        .into_response()
}

fn header<'a>(headers: &'a HeaderMap, name: &header::HeaderName) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn named_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// A body larger than the relay allocation boundary never reaches custody. This
/// transport refusal deliberately has no audit row and still uses the same JSON
/// envelope as every service answer.
fn too_large() -> Response {
    (
        StatusCode::PAYLOAD_TOO_LARGE,
        Json(serde_json::json!({
            "allowed": false,
            "code": "signer-request-too-large",
            "message": "the request body exceeds the signer relay limit",
        })),
    )
        .into_response()
}

/// A request that could not reach a signer decision. The public caller gets a
/// fixed sentence: transport internals and the configured custody address
/// belong on the operator page, and neither changes the caller's next action.
fn unavailable(_error: &anyhow::Error) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "allowed": false,
            "code": "signer-service-unavailable",
            "message": "the signer service is unavailable; no local signing fallback exists",
        })),
    )
        .into_response()
}

fn refusal_response(refusal: Refusal) -> Response {
    let status = StatusCode::from_u16(refusal.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let retry_after = refusal.retry_after_seconds();
    let mut response = (
        status,
        Json(RefusalBody {
            allowed: false,
            code: refusal.code,
            message: refusal.message,
        }),
    )
        .into_response();
    if let Some(seconds) = retry_after {
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from(seconds));
    }
    response
}
