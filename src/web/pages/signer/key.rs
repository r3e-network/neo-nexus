use axum::response::{Html, IntoResponse, Response};
use log::error;

use crate::signer_client::{AuditRow, Caller, KeyBoundary, Policy};

use super::super::super::{html, WebState};
use super::{overview, tabs, SignerTab};

const AUDIT_ROWS: usize = 50;

struct KeyPage {
    boundary: KeyBoundary,
    audit: Vec<AuditRow>,
    callers: Vec<Caller>,
}

pub(super) async fn render(
    state: &WebState,
    id: &str,
    flash: &str,
    notice: Option<&str>,
) -> Response {
    let banner = overview::custody_notice(state.custody());
    let asked = id.to_string();
    let body = match state.custody().ask(move |admin| load(admin, &asked)).await {
        Ok(page) => body(&banner, &page, notice),
        Err(error) => format!(
            "{}{}{}{}",
            html::page_head("Signer key", "Custody policy boundary", ""),
            tabs(SignerTab::Keys),
            banner,
            html::note(&format!(
                "No custody key is shown because none could be read: {error}"
            ))
        ),
    };
    Html(html::layout("Signer key", "signer", flash, &body)).into_response()
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

fn body(banner: &str, page: &KeyPage, notice: Option<&str>) -> String {
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
{policy}
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
        policy = policy_form(&path, policy),
        audit = overview::audit_table(&page.audit, std::slice::from_ref(key), &page.callers),
    )
}

fn on_off(flag: bool) -> String {
    if flag { "allowed" } else { "closed" }.to_string()
}

fn hex_prefix(encoded: &str) -> String {
    format!("{}…", encoded.chars().take(16).collect::<String>())
}

fn policy_form(path: &str, policy: &Policy) -> String {
    let enabled = vec!["enabled".to_string(), "disabled".to_string()];
    let families = vec![String::new(), "neo-n3".to_string(), "neox".to_string()];
    let (window_seconds, window_amount) = policy.window_limit.as_ref().map_or_else(
        || (String::new(), String::new()),
        |limit| (limit.seconds.to_string(), limit.max_amount.clone()),
    );
    let (signature_seconds, signature_count) = policy.max_signatures.as_ref().map_or_else(
        || (String::new(), String::new()),
        |limit| (limit.seconds.to_string(), limit.count.to_string()),
    );
    let mut fields = String::new();
    fields.push_str(
        &html::ChoiceField {
            label: "Chain family",
            name: "chain_family",
            options: &families,
            selected: policy.chain_family.as_deref().unwrap_or_default(),
            help: Some("Blank keeps the legacy Neo N3-compatible boundary."),
            ..Default::default()
        }
        .render(),
    );
    for (label, name, value) in [
        (
            "Consensus payloads",
            "allow_consensus",
            policy.allow_consensus,
        ),
        ("Raw signing", "allow_raw", policy.allow_raw),
        ("Transfers", "allow_transfer", policy.allow_transfer),
        (
            "Contract calls",
            "allow_contract_call",
            policy.allow_contract_call,
        ),
        (
            "Global scope",
            "allow_global_scope",
            policy.allow_global_scope,
        ),
    ] {
        fields.push_str(
            &html::ChoiceField {
                label,
                name,
                options: &enabled,
                selected: if value { "enabled" } else { "disabled" },
                ..Default::default()
            }
            .render(),
        );
    }
    for (label, name, values) in [
        (
            "Contract whitelist",
            "contract_whitelist",
            &policy.contract_whitelist,
        ),
        (
            "Contract blacklist",
            "contract_blacklist",
            &policy.contract_blacklist,
        ),
        (
            "Asset whitelist",
            "asset_whitelist",
            &policy.asset_whitelist,
        ),
        (
            "Asset blacklist",
            "asset_blacklist",
            &policy.asset_blacklist,
        ),
        (
            "Transfer-to whitelist",
            "transfer_to_whitelist",
            &policy.transfer_to_whitelist,
        ),
        (
            "Transfer-to blacklist",
            "transfer_to_blacklist",
            &policy.transfer_to_blacklist,
        ),
        (
            "EVM method whitelist",
            "evm_method_whitelist",
            &policy.evm_method_whitelist,
        ),
        (
            "EVM method blacklist",
            "evm_method_blacklist",
            &policy.evm_method_blacklist,
        ),
    ] {
        fields.push_str(&text_field(
            label,
            name,
            &values.join(", "),
            true,
            true,
            None,
        ));
    }
    for (label, name, value, help) in [
        (
            "Contract method whitelist (JSON)",
            "contract_method_whitelist",
            json(&policy.contract_method_whitelist),
            Some("Exact contract/method entries."),
        ),
        (
            "Contract method blacklist (JSON)",
            "contract_method_blacklist",
            json(&policy.contract_method_blacklist),
            Some("Exact contract/method denials."),
        ),
        (
            "Per-asset limits (JSON)",
            "asset_limits",
            json(&policy.asset_limits),
            Some("Per-asset single and window limits."),
        ),
        (
            "Additional signer policy fields (JSON)",
            "additional_fields",
            json(&policy.additional_fields),
            Some("Unknown fields are round-tripped so this console cannot erase newer policy."),
        ),
    ] {
        fields.push_str(&text_field(label, name, &value, true, true, help));
    }
    let numbers = [
        (
            "Maximum single amount",
            "max_single_amount",
            policy.max_single_amount.clone().unwrap_or_default(),
        ),
        ("Window seconds", "window_seconds", window_seconds),
        ("Window maximum amount", "window_max_amount", window_amount),
        (
            "Maximum transaction signers",
            "max_signers",
            optional(policy.max_signers),
        ),
        (
            "Maximum system fee",
            "max_system_fee",
            policy.max_system_fee.clone().unwrap_or_default(),
        ),
        (
            "Maximum network fee",
            "max_network_fee",
            policy.max_network_fee.clone().unwrap_or_default(),
        ),
        (
            "Signature window seconds",
            "signature_window_seconds",
            signature_seconds,
        ),
        (
            "Maximum signatures in window",
            "signature_window_count",
            signature_count,
        ),
        (
            "EVM maximum gas price",
            "evm_max_gas_price",
            policy.evm_max_gas_price.clone().unwrap_or_default(),
        ),
        (
            "EVM maximum gas limit",
            "evm_max_gas_limit",
            optional(policy.evm_max_gas_limit),
        ),
        (
            "EVM chain id",
            "evm_chain_id",
            optional(policy.evm_chain_id),
        ),
    ];
    for (label, name, value) in numbers {
        fields.push_str(&text_field(label, name, &value, false, true, None));
    }
    format!(
        r#"<form method="post" action="/signer/keys/{path}/policy" class="panel"><div class="grid">{fields}</div><div class="form-actions"><button class="primary" type="submit">Save boundary</button></div><p class="muted">An empty list means no restriction, not none permitted. A key with no policy signs nothing.</p></form>"#
    )
}

fn text_field(
    label: &str,
    name: &str,
    value: &str,
    full_width: bool,
    monospace: bool,
    help: Option<&str>,
) -> String {
    html::TextField {
        label,
        name,
        value,
        help,
        full_width,
        monospace,
        ..Default::default()
    }
    .render()
}

fn json(value: &impl serde::Serialize) -> String {
    match serde_json::to_string(value) {
        Ok(serialized) => serialized,
        Err(error) => {
            error!("NeoNexus could not render a signer policy field: {error}");
            String::new()
        }
    }
}

fn optional(value: Option<impl ToString>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
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
