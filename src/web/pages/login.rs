//! Login: one admin token unlocks a browser session. The token comes from
//! `--web-token`, the `NEONEXUS_WEB_TOKEN` environment variable, or the
//! generated value printed at startup.

use axum::{
    extract::{RawQuery, State},
    http::header,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;

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
    Form(input): Form<LoginInput>,
) -> Response {
    if !state.auth.token_matches(input.token.trim()) {
        return Redirect::to("/login?error=1").into_response();
    }
    let session = state.auth.create_session();
    (
        [
            (header::SET_COOKIE, state.auth.session_cookie(&session)),
            (header::LOCATION, "/".to_string()),
        ],
        axum::http::StatusCode::SEE_OTHER,
    )
        .into_response()
}

fn login_html(error: &str) -> String {
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
<p class="muted">Operator sign in. Use the workspace web token.</p>
{error}
<input type="password" name="token" placeholder="web token" autofocus autocomplete="off">
<button class="primary" type="submit">Sign in</button>
</form>
</div>
</body>
</html>"#,
        css = super::super::assets::CSS,
        error = if error.is_empty() {
            String::new()
        } else {
            format!(r#"<p class="err">{}</p>"#, html::escape(error))
        },
    )
}
