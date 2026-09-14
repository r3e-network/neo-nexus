//! Capability-aware signer surface for local wallets and Rust signer services.

mod caller;
mod key;
mod overview;

use axum::{
    extract::{Path, RawQuery, State},
    http::{header, HeaderValue},
    response::{Html, IntoResponse, Response},
};

use super::super::{html, WebState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SignerTab {
    Overview,
    Keys,
    Callers,
    Audit,
}

impl SignerTab {
    fn from_query(query: Option<&str>) -> Self {
        match html::query_value(query, "tab").as_deref() {
            Some("keys") => Self::Keys,
            Some("callers") => Self::Callers,
            Some("audit") => Self::Audit,
            _ => Self::Overview,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Keys => "keys",
            Self::Callers => "callers",
            Self::Audit => "audit",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Keys => "Keys",
            Self::Callers => "Callers",
            Self::Audit => "Audit",
        }
    }
}

pub async fn signer(State(state): State<WebState>, RawQuery(query): RawQuery) -> Response {
    let flash = html::flash(query.as_deref());
    render_tab(
        &state,
        &flash,
        None,
        SignerTab::from_query(query.as_deref()),
    )
    .await
}

pub async fn key_detail(
    State(state): State<WebState>,
    Path(id): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    key::render(&state, &id, &html::flash(query.as_deref()), None).await
}

/// Render the caller-management surface after minting or rotating a token.
/// The signature is retained for `signer_control`; direct control responses
/// land on Callers, where the one-time value belongs.
pub(crate) async fn render(state: &WebState, flash: &str, secret: Option<&str>) -> Response {
    render_tab(state, flash, secret, SignerTab::Callers).await
}

async fn render_tab(
    state: &WebState,
    flash: &str,
    secret: Option<&str>,
    tab: SignerTab,
) -> Response {
    let banner = overview::custody_notice(state.custody());
    let body = match state.custody().local_wallet_signer() {
        Some(signer) => overview::render_local_wallet(&banner, signer, tab),
        None => match state.custody().ask(overview::inventory).await {
            Ok(inventory) => overview::render_body(&banner, &inventory, secret, tab),
            Err(error) => overview::closed_page(&banner, &error, tab),
        },
    };
    let mut response = Html(html::layout("Signing keys", "signer", flash, &body)).into_response();
    if secret.is_some() {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}

pub async fn delete_form(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    key::delete_form(&state, &id).await
}

pub async fn caller_rotate_form(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    caller::confirmation(&state, &id, caller::CallerAction::Rotate).await
}

pub async fn caller_delete_form(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    caller::confirmation(&state, &id, caller::CallerAction::Delete).await
}

pub(super) fn tabs(active: SignerTab) -> String {
    let items = [
        SignerTab::Overview,
        SignerTab::Keys,
        SignerTab::Callers,
        SignerTab::Audit,
    ]
    .into_iter()
    .map(|tab| {
        let aria = if tab == active {
            r#" class="current" aria-current="page""#
        } else {
            ""
        };
        format!(
            r#"<a href="/signer?tab={}"{}>{}</a>"#,
            tab.slug(),
            aria,
            tab.label()
        )
    })
    .collect::<Vec<_>>()
    .join("");
    format!(r#"<nav class="subnav" aria-label="Signer sections">{items}</nav>"#)
}

#[cfg(test)]
#[path = "../../../tests/unit/web/signer/tests.rs"]
mod tests;
