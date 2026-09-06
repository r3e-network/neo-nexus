//! Browser response hardening shared by public and operator routes.

use std::sync::OnceLock;

use axum::{
    extract::{Request, State},
    http::{header, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sha2::{Digest, Sha256};

use super::{assets, WebState};

static CONTENT_SECURITY_POLICY: OnceLock<String> = OnceLock::new();

pub async fn apply(State(state): State<WebState>, request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    insert_static(headers, header::CACHE_CONTROL, "no-store");
    insert_static(headers, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    insert_static(headers, header::X_FRAME_OPTIONS, "DENY");
    // Chromium serializes a same-origin form POST's `Origin` as `null` under
    // `no-referrer`, which makes the exact-origin CSRF boundary reject the
    // workbench's own forms. `same-origin` preserves provenance only inside
    // this origin and still sends no Referer to another site.
    insert_static(headers, header::REFERRER_POLICY, "same-origin");
    insert_static(
        headers,
        HeaderName::from_static("cross-origin-opener-policy"),
        "same-origin",
    );
    insert_static(
        headers,
        HeaderName::from_static("cross-origin-resource-policy"),
        "same-origin",
    );
    insert_static(
        headers,
        HeaderName::from_static("permissions-policy"),
        "accelerometer=(), camera=(), geolocation=(), microphone=(), payment=(), usb=()",
    );
    if let Ok(value) = HeaderValue::from_str(content_security_policy()) {
        headers.insert(header::CONTENT_SECURITY_POLICY, value);
    }
    if state.web_security().secure_cookies() {
        insert_static(
            headers,
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000",
        );
    }
    response
}

fn insert_static(headers: &mut axum::http::HeaderMap, name: HeaderName, value: &'static str) {
    headers.insert(name, HeaderValue::from_static(value));
}

fn content_security_policy() -> &'static str {
    CONTENT_SECURITY_POLICY
        .get_or_init(|| {
            format!(
                "default-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'; connect-src 'self'; img-src 'self' data:; style-src 'sha256-{}'; script-src 'sha256-{}'",
                sha256_base64(assets::CSS),
                sha256_base64(assets::SCRIPT),
            )
        })
        .as_str()
}

fn sha256_base64(value: &str) -> String {
    STANDARD.encode(Sha256::digest(value.as_bytes()))
}

#[cfg(test)]
#[path = "../../tests/unit/web/security_headers/tests.rs"]
mod tests;
