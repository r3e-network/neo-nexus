use crate::{
    signer_client::{AuditRow, Caller, KeyPublic},
    types::Network,
    web::html,
};

use super::{
    audit::audit_table,
    callers::{caller_identity, caller_table, new_caller_form, new_workload_caller_form},
    model::AUDIT_ROWS,
};

pub fn page_head() -> String {
    let breadcrumb = html::breadcrumb(&[("KMS", "/signer"), ("Customer managed keys", "")]);
    let head = html::page_head(
        "KMS & Custody Signer",
        "Hardware-isolated cryptographic keys, least-privilege IAM caller policies, and immutable signing audit logs.",
        r#"<a class="btn" href="/settings/api-tokens">🔑 IAM API Tokens</a>"#,
    );
    format!("{breadcrumb}\n{head}")
}

pub fn overview_panel(keys: &[KeyPublic], callers: &[Caller], audit: &[AuditRow]) -> String {
    let refusal_count = audit.iter().filter(|row| row.outcome != "allowed").count();
    let key_rows = keys
        .iter()
        .take(4)
        .map(|key| {
            format!(
                r#"<li><a href="/signer/keys/{id}">{label}</a><span>{network} · {state}</span></li>"#,
                id = html::urlencoding_lite(&key.key_id),
                label = html::escape(&key.label),
                network = html::escape(&key.network),
                state = if key.signing_enabled { "enabled" } else { "disabled" },
            )
        })
        .collect::<String>();
    let key_rows = if key_rows.is_empty() {
        "<li><span>No keys in custody</span><span>held closed</span></li>".to_string()
    } else {
        key_rows
    };
    let caller_rows = callers
        .iter()
        .take(4)
        .map(|caller| {
            format!(
                "<li><strong>{}</strong><span>{}</span></li>",
                html::escape(&caller.label),
                html::escape(&caller_identity(caller))
            )
        })
        .collect::<String>();
    let caller_rows = if caller_rows.is_empty() {
        "<li><span>No callers registered</span><span>no signing access</span></li>".to_string()
    } else {
        caller_rows
    };
    format!(
        r#"<div class="stat-grid" aria-label="Custody summary">
<div class="stat"><div class="stat-value">{keys}</div><div class="stat-label">Keys</div><div class="stat-detail">sealed by the service</div></div>
<div class="stat positive"><div class="stat-value">{enabled}</div><div class="stat-label">Able to sign</div><div class="stat-detail">policy still applies</div></div>
<div class="stat info"><div class="stat-value">{callers}</div><div class="stat-label">Callers</div><div class="stat-detail">registered identities</div></div>
<div class="stat{refusal_tone}"><div class="stat-value">{refusals}</div><div class="stat-label">Recent refusals</div><div class="stat-detail">last {AUDIT_ROWS} decisions</div></div>
</div>
<div class="signer-grid">
<section class="surface"><div class="section-head"><h2>Key identities</h2><a href="/signer?tab=keys">Manage keys</a></div><ul class="summary-list">{key_rows}</ul></section>
<section class="surface"><div class="section-head"><h2>Caller access</h2><a href="/signer?tab=callers">Manage callers</a></div><ul class="summary-list">{caller_rows}</ul></section>
</div>
<div class="section-head"><h2>Recent audit</h2><a href="/signer?tab=audit">Open audit</a></div>
{audit}
<details class="panel"><summary>Provision and manage custody identities</summary>
<p class="muted">The dedicated Keys and Callers tabs arrange these controls for daily use. They remain available here so the original <code>/signer</code> surface and no-script bookmarks keep working.</p>
{legacy_keys}{new_key}{legacy_callers}{new_caller}{new_workload}
</details>"#,
        keys = keys.len(),
        enabled = keys.iter().filter(|key| key.signing_enabled).count(),
        callers = callers.len(),
        refusal_tone = if refusal_count > 0 {
            " warning"
        } else {
            " positive"
        },
        refusals = refusal_count,
        audit = audit_table(&audit[..audit.len().min(5)], keys, callers),
        legacy_keys = key_table(keys),
        new_key = new_key_form(),
        legacy_callers = caller_table(callers),
        new_caller = new_caller_form(keys),
        new_workload = new_workload_caller_form(keys),
    )
}

pub fn key_table(keys: &[KeyPublic]) -> String {
    if keys.is_empty() {
        return html::empty_state(
            "No keys in custody",
            "Generate one below. Existing keys are imported only at the signer service's trusted operator boundary.",
            "",
        );
    }
    let rows = keys
        .iter()
        .map(|key| {
            let path = html::urlencoding_lite(&key.key_id);
            html::row(&[
                html::raw_cell(&format!(
                    r#"<a href="/signer/keys/{path}">{}</a>"#,
                    html::escape(&key.label)
                )),
                html::cell(&key.address),
                html::cell(&key.network),
                html::cell(&key_chain_identity(key)),
                html::raw_cell(&state_badge(key.signing_enabled)),
                html::raw_cell(&format!(
                    r#"<div class="row-actions">{}<a class="btn small danger" href="/signer/keys/{path}/delete">Delete</a></div>"#,
                    key_actions(key),
                )),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &["Label", "Address", "Network", "Chain", "Signing", "Actions"],
        &rows,
    )
}

pub fn key_chain_identity(key: &KeyPublic) -> String {
    match (key.chain_family.as_deref(), key.chain_id) {
        (Some(family), Some(chain_id)) => format!("{family} · {chain_id}"),
        (Some(family), None) => family.to_string(),
        (None, Some(chain_id)) => format!("chain {chain_id}"),
        (None, None) => "not stated by the service".to_string(),
    }
}

pub fn state_badge(enabled: bool) -> String {
    let (class, label) = if enabled {
        ("running", "Enabled")
    } else {
        ("stopped", "Disabled")
    };
    format!(r#"<span class="badge {class}">{label}</span>"#)
}

pub fn key_actions(key: &KeyPublic) -> String {
    let path = html::urlencoding_lite(&key.key_id);
    let (label, disabled) = if key.signing_enabled {
        ("Disable", "true")
    } else {
        ("Enable", "false")
    };
    html::control_form(
        &format!("/signer/keys/{path}/state"),
        &[("disabled", disabled)],
        label,
    )
}

pub fn new_key_form() -> String {
    let networks = Network::ALL
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let families = vec!["neo-n3".to_string(), "neox".to_string()];
    format!(
        r#"<div class="panel"><h3>Generate a key in custody</h3>
<form method="post" action="/signer/keys/generate"><div class="grid">{label}{network}{family}{chain_id}{magic}</div><button type="submit">Generate</button></form>
<p class="muted">NeoNexus accepts no WIF, raw private key, NEP-2, or passphrase input. Existing key import stays at custody's trusted operator boundary.</p></div>"#,
        label = field_with_id("key-label", "Label", "label", "", None),
        network = html::ChoiceField {
            label: "Network",
            name: "network",
            options: &networks,
            selected: "testnet",
            help: Some("The key signs for this network only."),
            ..Default::default()
        }
        .render(),
        family = html::ChoiceField {
            label: "Chain family",
            name: "chain_family",
            options: &families,
            selected: "neo-n3",
            help: Some("Permanently binds custody to Neo N3 or NeoX."),
            ..Default::default()
        }
        .render(),
        chain_id = field(
            "Chain ID",
            "chain_id",
            "",
            Some("NeoX only; blank uses the canonical public-network id.")
        ),
        magic = field(
            "Network magic",
            "network_magic",
            "",
            Some("Required only to bind a private Neo network.")
        ),
    )
}

fn field(label: &str, name: &str, value: &str, help: Option<&str>) -> String {
    html::TextField {
        label,
        name,
        value,
        help,
        ..Default::default()
    }
    .render()
}

pub fn field_with_id(id: &str, label: &str, name: &str, value: &str, help: Option<&str>) -> String {
    html::TextField {
        id: Some(id),
        label,
        name,
        value,
        help,
        ..Default::default()
    }
    .render()
}
