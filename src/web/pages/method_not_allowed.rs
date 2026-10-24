//! The page an operator gets when the address exists but the method does not.
//!
//! Every form in this console posts to an address of its own, and none of those
//! addresses was meant to be a page: `GET /settings/density` asks the console to
//! render something it only ever does by accepting a form. axum's default answer
//! is a `405` with an empty body — nothing to read, no way back, and no hint that
//! the address is real and only the method is wrong. An operator who typed the
//! address from memory or followed a stale form action cannot tell that apart
//! from a broken console.
//!
//! The `Allow` header is deliberately not set here. axum puts it on the response
//! after this handler returns, so a tool still learns which methods would have
//! worked; replacing the body must not cost that header, and a test pins it.

use axum::{
    extract::Request,
    http::{Method, StatusCode},
    response::Response,
};
use serde_json::json;

use super::unserved;

pub async fn method_not_allowed(method: Method, request: Request) -> Response {
    let path = request.uri().path().to_string();
    if unserved::wants_json(&path) {
        return unserved::json(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "method": method.as_str(),
                "path": path,
            }),
        );
    }

    unserved::page(
        StatusCode::METHOD_NOT_ALLOWED,
        "Method not allowed",
        "This address exists. It does not accept this method.",
        &format!("Nothing here answers {method}"),
        "The console serves this address to other methods. If a link, a bookmark \
         or a form action brought you here, it points at something that is \
         submitted from a page rather than opened as one — the control that does \
         it is on the page that shows it. These addresses always open:",
    )
}
