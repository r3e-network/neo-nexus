//! Cross-site write protection for the browser workbench.
//!
//! Management forms authenticate with a session cookie, which a browser
//! attaches to any request it makes towards this origin — including requests a
//! malicious page tricked it into sending. SameSite cookies are the first
//! line; this module is the second: a state-changing request that names a
//! different origin in `Origin` or `Referer` is rejected before the session is
//! ever consulted. Requests carrying neither header have no ambient-credential
//! surface to abuse — scripts, the CLI and MCP clients — and pass through.
//!
//! See the OWASP CSRF prevention sheet, "verifying origin".

use axum::{
    extract::Request,
    http::{header, Method},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn verify_origin(request: Request, next: Next) -> Response {
    let safe = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    if safe {
        return next.run(request).await;
    }

    let host = header_text(request.headers(), header::HOST);
    // `Origin: null` is what sandboxed frames and some redirects send; it
    // names no origin and is rejected outright rather than compared.
    if header_text(request.headers(), header::ORIGIN).as_deref() == Some("null") {
        return (
            axum::http::StatusCode::FORBIDDEN,
            "a null origin was rejected",
        )
            .into_response();
    }
    let origin_authority = header_text(request.headers(), header::ORIGIN)
        .map(|value| authority_after_scheme(&value))
        .or_else(|| {
            header_text(request.headers(), header::REFERER)
                .map(|value| authority_after_scheme(&value))
        });

    match origin_authority {
        // A browser always names an origin on a cross-site write; absence
        // means the caller is not a browser page, and there is no ambient
        // cookie channel to forge.
        None => next.run(request).await,
        Some(authority) if authority_matches(&authority, host.as_deref()) => {
            next.run(request).await
        }
        Some(authority) => (
            axum::http::StatusCode::FORBIDDEN,
            format!("cross-origin request from '{authority}' was rejected"),
        )
            .into_response(),
    }
}

fn header_text(headers: &axum::http::HeaderMap, name: header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

/// The `host[:port]` part of an `Origin` (`scheme://host[:port]`) or a
/// `Referer` (`scheme://host[:port]/path`), lower-cased for comparison.
fn authority_after_scheme(value: &str) -> String {
    value
        .split("://")
        .nth(1)
        .unwrap_or(value)
        .split(['/', '?'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// Whether a claimed authority names the host this server answered on.
///
/// Either side may omit the port when it is the scheme's default, so
/// `example.com` and `example.com:443` name the same deployment behind a
/// TLS-terminating proxy. The names must still be equal — a different name is
/// a different origin no matter which port it claims.
fn authority_matches(authority: &str, host_header: Option<&str>) -> bool {
    let Some(host_header) = host_header else {
        return false;
    };
    let host_header = host_header.to_ascii_lowercase();
    let (authority_name, authority_port) = split_authority(authority);
    let (host_name, host_port) = split_authority(&host_header);
    authority_name == host_name
        && match (authority_port, host_port) {
            (Some(left), Some(right)) => left == right,
            _ => true,
        }
}

fn split_authority(authority: &str) -> (&str, Option<&str>) {
    match authority.rsplit_once(':') {
        Some((name, port)) if port.chars().all(|c| c.is_ascii_digit()) && !port.is_empty() => {
            (name, Some(port))
        }
        _ => (authority, None),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/web/csrf/tests.rs"]
mod tests;
