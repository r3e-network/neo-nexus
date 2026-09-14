//! Confirmation pages for caller changes that can interrupt workloads.

use anyhow::{anyhow, Result};
use axum::response::{Html, IntoResponse, Response};

use crate::signer_client::Caller;

use super::super::super::{html, Admin, WebState};
use super::{overview, tabs, SignerTab};

#[derive(Clone, Copy)]
pub(super) enum CallerAction {
    Rotate,
    Delete,
}

impl CallerAction {
    fn verb(self) -> &'static str {
        match self {
            Self::Rotate => "Rotate",
            Self::Delete => "Delete",
        }
    }

    fn consequence(self) -> &'static str {
        match self {
            Self::Rotate => {
                "The current bearer token stops working immediately. The replacement is shown once, so update the workload before leaving the result page."
            }
            Self::Delete => {
                "This permanently revokes the caller. Workloads using it can no longer ask custody to sign, and the caller cannot be restored."
            }
        }
    }

    fn path(self, caller_id: &str) -> String {
        let id = html::urlencoding_lite(caller_id);
        match self {
            Self::Rotate => format!("/signer/callers/{id}/rotate"),
            Self::Delete => format!("/signer/callers/{id}/delete"),
        }
    }
}

pub(super) async fn confirmation(state: &WebState, id: &str, action: CallerAction) -> Response {
    let banner = overview::custody_notice(state.custody());
    let asked = id.to_string();
    let body = match state
        .custody()
        .ask(move |admin| load_caller(admin, &asked))
        .await
    {
        Ok(caller) => confirmation_body(&banner, &caller, action),
        Err(error) => format!(
            "{}{}{}{}",
            html::page_head("Signer caller", "Custody access identity", ""),
            tabs(SignerTab::Callers),
            banner,
            html::empty_state(
                "Caller unavailable",
                &format!(
                    "No caller change is offered because the identity could not be read: {error}"
                ),
                r#"<a class="btn" href="/signer?tab=callers">Back to callers</a>"#,
            )
        ),
    };
    Html(html::layout(
        &format!("{} caller", action.verb()),
        "signer",
        "",
        &body,
    ))
    .into_response()
}

fn load_caller(admin: &Admin, id: &str) -> Result<Caller> {
    let credentials = admin.credentials()?;
    admin
        .client()
        .list_callers(&credentials)?
        .into_parts()?
        .into_iter()
        .find(|caller| caller.id == id)
        .ok_or_else(|| anyhow!("custody caller {id} was not found"))
}

pub(super) fn confirmation_body(banner: &str, caller: &Caller, action: CallerAction) -> String {
    let title = format!("{} {}?", action.verb(), caller.label);
    let mode = caller.auth_mode.as_deref().unwrap_or("bearer");
    let rotate_blocked = matches!(action, CallerAction::Rotate) && mode != "bearer";
    let controls = if rotate_blocked {
        html::notice(
            "danger",
            "This workload identity has no bearer token to rotate. Return to Callers to change or remove it.",
        )
    } else {
        format!(
            r#"<div class="form-actions">{}<a class="btn" href="/signer?tab=callers">Cancel</a></div>"#,
            html::danger_control_form(
                &action.path(&caller.id),
                &[],
                &format!("{} {}", action.verb(), caller.label),
            )
        )
    };
    let facts = [
        ("Identity", caller.id.as_str()),
        ("Authentication", mode),
        (
            "Status",
            if caller.disabled {
                "disabled"
            } else {
                "enabled"
            },
        ),
    ]
    .iter()
    .map(|(label, value)| html::row(&[html::cell(label), html::cell(value)]))
    .collect::<Vec<_>>();
    format!(
        r#"{crumb}{head}{tabs}{banner}{warning}{facts}{controls}"#,
        crumb = html::breadcrumb(&[
            ("Signing keys", "/signer"),
            ("Callers", "/signer?tab=callers"),
            (action.verb(), "")
        ]),
        head = html::page_head(
            &title,
            "Review the affected custody identity before continuing.",
            ""
        ),
        tabs = tabs(SignerTab::Callers),
        warning = html::notice("danger", action.consequence()),
        facts = html::table(&["Setting", "Value"], &facts),
    )
}
