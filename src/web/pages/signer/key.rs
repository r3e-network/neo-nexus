use axum::response::{Html, IntoResponse, Response};

use crate::signer_client::{AuditRow, Caller, KeyBoundary};

use super::super::super::{
    form_state::{self, FieldErrors, FormDraft},
    html,
    signer_form::{self, PolicyForm},
    WebState,
};
use super::{overview, tabs, SignerTab};

const AUDIT_ROWS: usize = 50;

struct KeyPage {
    boundary: KeyBoundary,
    audit: Vec<AuditRow>,
    callers: Vec<Caller>,
}

/// What the policy form shows: the text to draw and what to complain about.
///
/// A plain page load draws the boundary the service stored and complains about
/// nothing. A refused save draws the text the operator posted and puts each
/// message beside the control that caused it. The form itself is the same
/// either way, which is what makes the second case a *redraw* rather than a
/// second, parallel form.
struct PolicyDraft<'a> {
    form: &'a PolicyForm,
    errors: &'a FieldErrors,
}

pub(super) async fn render(
    state: &WebState,
    id: &str,
    flash: &html::Flash,
    notice: Option<&str>,
) -> Response {
    let banner = overview::custody_notice(state.custody());
    let asked = id.to_string();
    let body = match state.custody().ask(move |admin| load(admin, &asked)).await {
        Ok(page) => {
            let stored = PolicyForm::from_policy(&page.boundary.policy);
            let errors = FieldErrors::new();
            body(
                &banner,
                &page,
                notice,
                &PolicyDraft {
                    form: &stored,
                    errors: &errors,
                },
            )
        }
        Err(error) => unreadable(&banner, &error),
    };
    Html(html::layout("Signer key", "signer", flash, &body)).into_response()
}

/// Draw the key page with the boundary the operator typed still in the form.
///
/// This is the same page [`render`] produces, with two differences: the form is
/// fed the posted text rather than the stored boundary, and every field the
/// console could not accept carries its message. Nothing was saved, so the page
/// must not look like it was: the cards above the form keep describing what the
/// service is enforcing, and the form says which of its values it would not take.
///
/// Failure to read the key is returned rather than rendered, because the caller
/// is the only one who still has the operator's text and can decide what to say
/// when even this page cannot be drawn.
pub(super) async fn render_with_policy_draft(
    state: &WebState,
    id: &str,
    draft: &PolicyForm,
    errors: &FieldErrors,
) -> anyhow::Result<Response> {
    let banner = overview::custody_notice(state.custody());
    let asked = id.to_string();
    let page = state
        .custody()
        .ask(move |admin| load(admin, &asked))
        .await?;
    let body = body(
        &banner,
        &page,
        None,
        &PolicyDraft {
            form: draft,
            errors,
        },
    );
    Ok(Html(html::layout(
        "Signer key",
        "signer",
        html::Flash::default(),
        &body,
    ))
    .into_response())
}

/// The page with no key on it, and the reason.
fn unreadable(banner: &str, error: &anyhow::Error) -> String {
    format!(
        "{}{}{}{}",
        html::page_head("Signer key", "Custody policy boundary", ""),
        tabs(SignerTab::Keys),
        banner,
        html::note(&format!(
            "No custody key is shown because none could be read: {error}"
        ))
    )
}

fn load(admin: &crate::web::Admin, id: &str) -> anyhow::Result<KeyPage> {
    let credentials = admin.credentials()?;
    let client = admin.client();
    Ok(KeyPage {
        boundary: client.key_boundary(&credentials, id)?.into_parts()?,
        audit: client
            .list_audit(&credentials, Some(id), Some(AUDIT_ROWS))?
            .into_parts()?,
        callers: client.list_callers(&credentials)?.into_parts()?,
    })
}

fn body(banner: &str, page: &KeyPage, notice: Option<&str>, draft: &PolicyDraft<'_>) -> String {
    let key = &page.boundary.key;
    let policy = &page.boundary.policy;
    let path = html::urlencoding_lite(&key.key_id);
    let magic = key.network_magic.map_or_else(
        || "not stated by the service".to_string(),
        |value| value.to_string(),
    );
    let problems = page
        .boundary
        .problems
        .iter()
        .map(|problem| format!("<li>{}</li>", html::escape(&problem.message)))
        .collect::<String>();
    let warnings = if problems.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="notice warn"><ul>{problems}</ul></div>"#)
    };
    format!(
        r#"{crumb}{head}{tabs}{banner}{stats}{notice}{warnings}
<div class="section-head"><h2>Policy boundary</h2><span class="muted">The signer service enforces every field below.</span></div>
{summary}{policy}
<div class="section-head"><h2>Key audit</h2><a href="/signer?tab=audit">Whole-vault audit</a></div>
{audit}"#,
        crumb = html::breadcrumb(&[("Signer", "/signer?tab=keys"), (&key.label, "")]),
        head = html::page_head(
            &format!("{} · {}", key.label, key.network),
            &format!(
                "{} · chain {} · public key {} · magic {} · signing {}. The private half remains sealed by custody.",
                key.address,
                overview::key_chain_identity(key),
                hex_prefix(&key.public_key),
                magic,
                if key.signing_enabled { "on" } else { "off" }
            ),
            &format!(
                "{}<a class=\"btn\" href=\"/signer?tab=keys\">Back to keys</a>",
                overview::key_actions(key)
            ),
        ),
        tabs = tabs(SignerTab::Keys),
        stats = html::cards(&[
            ("Consensus", on_off(policy.allow_consensus)),
            ("Raw signing", on_off(policy.allow_raw)),
            ("Transfers", on_off(policy.allow_transfer)),
            ("Contract calls", on_off(policy.allow_contract_call)),
            ("Global scope", on_off(policy.allow_global_scope)),
        ]),
        notice = notice.map_or_else(String::new, |text| html::notice("warn", text)),
        summary = policy_summary(draft),
        policy = policy_form(&path, draft),
        audit = overview::audit_table(&page.audit, std::slice::from_ref(key), &page.callers),
    )
}

fn on_off(flag: bool) -> String {
    if flag { "allowed" } else { "closed" }.to_string()
}

fn hex_prefix(encoded: &str) -> String {
    format!("{}…", encoded.chars().take(16).collect::<String>())
}

/// How much of the form to look at, and what did not save.
fn policy_summary(draft: &PolicyDraft<'_>) -> String {
    if draft.errors.is_empty() {
        return String::new();
    }
    // A refused form has to look refused. The cards above it describe the
    // boundary the service is still enforcing while the form holds the text it
    // would not take: both are right, they disagree, and an operator who is not
    // told so concludes the save half-worked.
    format!(
        "{}{}",
        form_state::banner(draft.errors),
        html::note(
            "Nothing was saved. The cards above still describe the boundary the service is \
             enforcing; what is in the form is what you typed and what it would not accept."
        )
    )
}

fn policy_form(path: &str, draft: &PolicyDraft<'_>) -> String {
    let form = draft.form;
    let enabled = vec!["enabled".to_string(), "disabled".to_string()];
    let families = vec![String::new(), "neo-n3".to_string(), "neox".to_string()];
    let mut fields = String::new();
    fields.push_str(
        &html::ChoiceField {
            label: "Chain family",
            name: signer_form::CHAIN_FAMILY,
            options: &families,
            selected: form.value(signer_form::CHAIN_FAMILY),
            help: Some("Blank keeps the legacy Neo N3-compatible boundary."),
            error: form_state::error_for(draft.errors, signer_form::CHAIN_FAMILY),
            ..Default::default()
        }
        .render(),
    );
    for (label, name) in [
        ("Consensus payloads", signer_form::ALLOW_CONSENSUS),
        ("Raw signing", signer_form::ALLOW_RAW),
        ("Transfers", signer_form::ALLOW_TRANSFER),
        ("Contract calls", signer_form::ALLOW_CONTRACT_CALL),
        ("Global scope", signer_form::ALLOW_GLOBAL_SCOPE),
    ] {
        fields.push_str(
            &html::ChoiceField {
                label,
                name,
                options: &enabled,
                selected: form.value(name),
                error: form_state::error_for(draft.errors, name),
                ..Default::default()
            }
            .render(),
        );
    }
    for (label, name) in [
        ("Contract whitelist", signer_form::CONTRACT_WHITELIST),
        ("Contract blacklist", signer_form::CONTRACT_BLACKLIST),
        ("Asset whitelist", signer_form::ASSET_WHITELIST),
        ("Asset blacklist", signer_form::ASSET_BLACKLIST),
        ("Transfer-to whitelist", signer_form::TRANSFER_TO_WHITELIST),
        ("Transfer-to blacklist", signer_form::TRANSFER_TO_BLACKLIST),
        ("EVM method whitelist", signer_form::EVM_METHOD_WHITELIST),
        ("EVM method blacklist", signer_form::EVM_METHOD_BLACKLIST),
    ] {
        fields.push_str(&text_field(label, name, form, draft, true, true, None));
    }
    for (label, name, help) in [
        (
            "Contract method whitelist (JSON)",
            signer_form::CONTRACT_METHOD_WHITELIST,
            Some("Exact contract/method entries."),
        ),
        (
            "Contract method blacklist (JSON)",
            signer_form::CONTRACT_METHOD_BLACKLIST,
            Some("Exact contract/method denials."),
        ),
        (
            "Per-asset limits (JSON)",
            signer_form::ASSET_LIMITS,
            Some("Per-asset single and window limits."),
        ),
        (
            "Additional signer policy fields (JSON)",
            signer_form::ADDITIONAL_FIELDS,
            Some("Unknown fields are round-tripped so this console cannot erase newer policy."),
        ),
    ] {
        fields.push_str(&text_field(label, name, form, draft, true, true, help));
    }
    // The amounts are decimal *text* on purpose: the custody service decides
    // what a ceiling may look like, and it must be the one to say "1e6" is not
    // a number. A numeric input would quietly reject that before the request
    // left, taking the domain's word away from it.
    for (label, name) in [
        ("Maximum single amount", signer_form::MAX_SINGLE_AMOUNT),
        ("Window maximum amount", signer_form::WINDOW_MAX_AMOUNT),
        ("Maximum system fee", signer_form::MAX_SYSTEM_FEE),
        ("Maximum network fee", signer_form::MAX_NETWORK_FEE),
        ("EVM maximum gas price", signer_form::EVM_MAX_GAS_PRICE),
    ] {
        fields.push_str(&text_field(label, name, form, draft, false, true, None));
    }
    // These are whole numbers the domain parses, so the browser can say so
    // before the request is sent. The upper bound is per field: `max_signers`
    // is a u16, the rest are u64 and have no ceiling the domain enforces.
    for (label, name, max) in [
        ("Window seconds", signer_form::WINDOW_SECONDS, None),
        (
            "Maximum transaction signers",
            signer_form::MAX_SIGNERS,
            Some("65535"),
        ),
        (
            "Signature window seconds",
            signer_form::SIGNATURE_WINDOW_SECONDS,
            None,
        ),
        (
            "Maximum signatures in window",
            signer_form::SIGNATURE_WINDOW_COUNT,
            None,
        ),
        (
            "EVM maximum gas limit",
            signer_form::EVM_MAX_GAS_LIMIT,
            None,
        ),
        ("EVM chain id", signer_form::EVM_CHAIN_ID, None),
    ] {
        fields.push_str(
            &html::TextField {
                label,
                name,
                value: form.value(name),
                error: form_state::error_for(draft.errors, name),
                full_width: false,
                monospace: true,
                input_type: html::TextInputType::Number,
                min: Some("0"),
                max,
                step: Some("1"),
                ..Default::default()
            }
            .render(),
        );
    }
    format!(
        r#"<form method="post" action="/signer/keys/{path}/policy" class="panel"><div class="grid">{fields}</div><div class="form-actions"><button class="primary" type="submit">Save boundary</button></div><p class="muted">An empty list means no restriction, not none permitted. A key with no policy signs nothing.</p></form>"#
    )
}

fn text_field(
    label: &str,
    name: &'static str,
    form: &PolicyForm,
    draft: &PolicyDraft<'_>,
    full_width: bool,
    monospace: bool,
    help: Option<&str>,
) -> String {
    html::TextField {
        label,
        name,
        value: form.value(name),
        help,
        error: form_state::error_for(draft.errors, name),
        full_width,
        monospace,
        ..Default::default()
    }
    .render()
}

pub(super) async fn delete_form(state: &WebState, id: &str) -> Response {
    let banner = overview::custody_notice(state.custody());
    let asked = id.to_string();
    let body = match state
        .custody()
        .ask(move |admin| {
            Ok(admin
                .client()
                .key_boundary(&admin.credentials()?, &asked)?
                .into_parts()?
                .key)
        })
        .await
    {
        Ok(key) => {
            let path = html::urlencoding_lite(&key.key_id);
            let form = format!(
                r#"<div class="form-actions">{}<a class="btn" href="/signer/keys/{}">Cancel</a></div>"#,
                html::danger_control_form(
                    &format!("/signer/keys/{path}/delete"),
                    &[],
                    "Delete this key",
                ),
                path,
            );
            format!(
                r#"{crumb}{tabs}{banner}<h1>Delete {label}?</h1>{notice}<p><span class="mono">{address}</span> controls whatever this key signs. Its sealed private key will be removed from custody and cannot be recovered there; audit history remains.</p>{form}"#,
                crumb = html::breadcrumb(&[("Signer", "/signer?tab=keys"), (&key.label, ""), ("Delete", "")]),
                tabs = tabs(SignerTab::Keys),
                label = html::escape(&key.label),
                notice = html::notice("danger", "There is no export from custody. Back up the key elsewhere first if it may be needed."),
                address = html::escape(&key.address),
            )
        }
        Err(error) => format!(
            "{banner}{}",
            html::note(&format!("custody key {id} could not be read: {error}"))
        ),
    };
    Html(html::layout("Delete key", "signer", "", &body)).into_response()
}
