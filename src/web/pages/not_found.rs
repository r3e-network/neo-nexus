//! The page an operator gets when an address does not exist.
//!
//! Without it, axum answered a mistyped URL with a bare `404 Not Found` in plain
//! text: no navigation, no way back, and a look that belongs to no part of this
//! console. An operator who followed a stale bookmark had to edit the address bar
//! to recover, which is a poor answer for a product whose whole job is being
//! reachable.

use axum::{
    extract::Request,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::web::html;

/// Whether an address that does not exist should answer in JSON.
///
/// A program calling `/api/...` parses the body. Handing it an HTML error page
/// turns "wrong endpoint" into "unparseable response", which reads as a broken
/// service rather than a wrong address — and the two need different fixes. The
/// signing API is included because its callers are programs as well.
fn wants_json(path: &str) -> bool {
    path.starts_with("/api/") || path.starts_with("/signer/api/")
}

pub async fn not_found(request: Request) -> Response {
    let path = request.uri().path().to_string();
    if wants_json(&path) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "not_found", "path": path })),
        )
            .into_response();
    }

    // The address is echoed because a typo is the common case, and seeing it is
    // how one finds the mistake. `empty_state` escapes its body, which matters
    // here: a path is whatever the caller sent, not a string this console chose.
    let body = format!(
        r#"{head}{state}"#,
        head = html::page_head(
            "Page not found",
            "Nothing in this workspace answers this address.",
            "",
        ),
        state = html::empty_state(
            "No page at this address",
            &format!(
                "The console has no page at {path}. If a link brought you here, that link is out of \
                 date — the address it points at has probably been renamed, or it belonged to a \
                 feature this workspace does not have."
            ),
            &shortcuts(),
        ),
    );

    (
        StatusCode::NOT_FOUND,
        Html(html::layout(
            "Page not found",
            // No section is current on a page that belongs to none.
            "",
            html::Flash::default(),
            &body,
        )),
    )
        .into_response()
}

/// The ways out, in the order an operator is most likely to want them.
///
/// A 404 that only says "not found" leaves the operator to guess the address
/// bar; these are the three destinations that cover most recoveries.
fn shortcuts() -> String {
    [
        ("/", "Console home"),
        ("/nodes", "Nodes"),
        ("/events", "Events"),
    ]
    .iter()
    .map(|(href, label)| format!(r#"<a class="btn" href="{href}">{label}</a>"#))
    .collect::<Vec<_>>()
    .join(" ")
}
