//! Login: one admin token unlocks a browser session. The token comes from
//! a protected token file or the bootstrap value printed only to an interactive
//! terminal.

use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, RawQuery, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;

use super::super::auth::LoginDecision;
use super::super::html;

#[derive(Deserialize)]
pub struct LoginInput {
    #[serde(default)]
    token: String,
}

pub async fn login_page(RawQuery(query): RawQuery) -> Response {
    let flash = super::super::html::query_value(query.as_deref(), "error")
        .map(|_| "That token was not accepted.".to_string())
        .unwrap_or_default();
    Html(login_html(&flash)).into_response()
}

pub async fn login_submit(
    State(state): State<super::super::WebState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Form(input): Form<LoginInput>,
) -> Response {
    match state.auth.authenticate(peer.ip(), input.token.trim()) {
        LoginDecision::Rejected => return Redirect::to("/login?error=1").into_response(),
        LoginDecision::Throttled {
            retry_after_seconds,
        } => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, retry_after_seconds.to_string())],
                Html(login_html(
                    "Too many sign-in attempts. Wait before trying again.",
                )),
            )
                .into_response()
        }
        LoginDecision::Accepted => {}
    }
    let session = state.auth.create_session();
    (
        [
            (header::SET_COOKIE, state.session_cookie(&session)),
            (header::LOCATION, "/".to_string()),
        ],
        axum::http::StatusCode::SEE_OTHER,
    )
        .into_response()
}

fn login_html(error: &str) -> String {
    let accessibility = if error.is_empty() {
        r#" aria-describedby="login-help""#
    } else {
        r#" aria-invalid="true" aria-describedby="login-help login-error""#
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Sign in · NeoNexus</title>
<style>{css}</style>
</head>
<body>
<div class="login-wrap">
<form class="login-card" method="post" action="/login">
<h1>NeoNexus</h1>
<p class="muted" id="login-help">Operator sign in. Use the workspace web token.</p>
{error}
<label class="field" for="web-token"><span>Web token</span>
<input id="web-token" type="password" name="token" placeholder="Paste the workspace web token" autofocus autocomplete="current-password" autocapitalize="none" spellcheck="false"{accessibility}></label>
<button class="primary" type="submit">Sign in</button>
</form>
</div>
</body>
</html>"#,
        css = super::super::assets::CSS,
        error = if error.is_empty() {
            String::new()
        } else {
            format!(
                r#"<p class="err" id="login-error" role="alert" aria-live="assertive">{}</p>"#,
                html::escape(error)
            )
        },
        accessibility = accessibility,
    )
}
