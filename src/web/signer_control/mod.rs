//! Custody controls from the browser: ask the service to generate a key, write
//! its boundary, switch it off, and register the callers that may ask it to
//! sign.

mod callers;
mod forms;
mod keys;

pub use callers::*;
pub use forms::*;
pub use keys::*;

use anyhow::Result;
use axum::response::{IntoResponse, Redirect, Response};

use crate::web::{html, Admin, WebState};

/// Ask the service, off the request thread, and redirect with its answer.
pub(crate) async fn manage<Call>(state: &WebState, path: String, call: Call) -> Response
where
    Call: FnOnce(&Admin) -> Result<String> + Send + 'static,
{
    respond(&path, state.custody().ask(call).await)
}

pub(crate) fn respond(path: &str, outcome: Result<String>) -> Response {
    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("not saved: {error}"),
    };
    Redirect::to(&format!(
        "{path}?flash={}",
        html::urlencoding_lite(&message),
        path = path
    ))
    .into_response()
}

#[cfg(test)]
#[path = "../../../tests/unit/web/signer_control/tests.rs"]
mod tests;
