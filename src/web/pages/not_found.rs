//! The page an operator gets when an address does not exist.
//!
//! Without it, axum answered a mistyped URL with a bare `404 Not Found` in plain
//! text: no navigation, no way back, and a look that belongs to no part of this
//! console. An operator who followed a stale bookmark had to edit the address bar
//! to recover, which is a poor answer for a product whose whole job is being
//! reachable.
//!
//! The page itself, and the JSON a program gets instead, come from
//! [`super::unserved`] — the same shape a `405` uses, so the two answers cannot
//! describe the same console in two different ways.

use axum::{extract::Request, http::StatusCode, response::Response};
use serde_json::json;

use super::unserved;

pub async fn not_found(request: Request) -> Response {
    let path = request.uri().path().to_string();
    if unserved::wants_json(&path) {
        return unserved::json(
            StatusCode::NOT_FOUND,
            json!({ "error": "not_found", "path": path }),
        );
    }

    // The address is echoed because a typo is the common case, and seeing it is
    // how one finds the mistake.
    unserved::page(
        StatusCode::NOT_FOUND,
        "Page not found",
        "Nothing in this workspace answers this address.",
        "No page at this address",
        &format!(
            "The console has no page at {path}. If a link brought you here, that link is out of \
             date — the address it points at has probably been renamed, or it belonged to a \
             feature this workspace does not have."
        ),
    )
}
