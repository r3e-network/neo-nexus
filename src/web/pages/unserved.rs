//! The shape of the two answers the console gives a request it will not serve.
//!
//! An address it does not have is a `404`; an address it does have, but not for
//! the method the caller used, is a `405`. Both used to fall through to axum's
//! defaults, which are correct HTTP and useless to a person: a bare line of
//! text for the first, and an empty body for the second. A program cannot do
//! anything with an HTML error page either — the failure reads as "the service
//! is broken" instead of "the request is wrong", and those need different fixes.
//!
//! The shape lives in one place rather than being copied into the two handlers,
//! because the answers have to agree: the same JSON keys from both, and the
//! same ways out on both pages. Two copies of "which addresses answer in JSON"
//! would drift the first time one of them learned a new prefix.

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};

use crate::web::html;

/// Whether an address that cannot be served should answer in JSON.
///
/// A program calling `/api/...` parses the body. Handing it an HTML error page
/// turns "wrong endpoint" into "unparseable response". The signing API is
/// included because its callers are programs as well.
pub(super) fn wants_json(path: &str) -> bool {
    path.starts_with("/api/") || path.starts_with("/signer/api/")
}

/// A JSON refusal. The status is the caller's to choose; the content type is
/// not, which is the whole reason this is a function and not three lines at
/// each call site.
pub(super) fn json(status: StatusCode, body: serde_json::Value) -> Response {
    (status, Json(body)).into_response()
}

/// A page for a person, ending in the same ways out as every other dead end.
///
/// `message` is passed through `empty_state`, which escapes its body: a path or
/// a method is whatever the caller sent, not a string this console chose.
pub(super) fn page(
    status: StatusCode,
    title: &str,
    subtitle: &str,
    heading: &str,
    message: &str,
) -> Response {
    let body = format!(
        r#"{head}{state}"#,
        head = html::page_head(title, subtitle, ""),
        state = html::empty_state(heading, message, &ways_out()),
    );

    (
        status,
        Html(html::layout(
            title,
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
/// A refusal that only says "not found" leaves the operator to guess at the
/// address bar; these are the three destinations that cover most recoveries.
fn ways_out() -> String {
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
