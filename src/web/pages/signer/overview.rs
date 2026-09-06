use crate::{
    signer_client::{AuditRow, Caller, Grant, KeyPublic},
    signing::{LocalWalletSigner, SignerBackendKind},
    types::Network,
};

use super::super::super::{html, time, Admin, Custody};
use super::{tabs, SignerTab};

const AUDIT_ROWS: usize = 50;

pub(super) struct Inventory {
    keys: Vec<KeyPublic>,
    callers: Vec<Caller>,
    audit: Vec<AuditRow>,
}

pub(super) fn inventory(admin: &Admin) -> anyhow::Result<Inventory> {
    let credentials = admin.credentials()?;
    let client = admin.client();
    Ok(Inventory {
        keys: client.list_keys(&credentials)?.into_parts()?,
        callers: client.list_callers(&credentials)?.into_parts()?,
        audit: client
            .list_audit(&credentials, None, Some(AUDIT_ROWS))?
            .into_parts()?,
    })
}

pub(super) fn closed_page(banner: &str, error: &anyhow::Error, tab: SignerTab) -> String {
    format!(
        "{}{}{}{}",
        page_head(),
        tabs(tab),
        banner,
        html::empty_state(
            "Custody inventory unavailable",
            &format!("No inventory is shown because none could be read: {error}"),
            "",
        )
    )
}

pub(super) fn render_body(
    banner: &str,
    inventory: &Inventory,
    secret: Option<&str>,
    tab: SignerTab,
) -> String {
    let Inventory {
        keys,
        callers,
        audit,
    } = inventory;
    let content = match tab {
        SignerTab::Overview => overview_panel(keys, callers, audit),
        SignerTab::Keys => format!(
            r#"<div class="section-head"><h2>Custody keys</h2><span class="muted">Private material never enters NeoNexus.</span></div>{}{}"#,
            key_table(keys),
            new_key_form()
        ),
        SignerTab::Callers => format!(
            r#"<div class="section-head"><h2>Caller access</h2><span class="muted">Least-privilege identities allowed to ask custody.</span></div>{}{}{}{}"#,
            caller_table(callers),
            new_caller_form(keys),
            new_workload_caller_form(keys),
            api_reference()
        ),
        SignerTab::Audit => format!(
            r#"<div class="section-head"><h2>Custody audit</h2><span class="muted">Latest {AUDIT_ROWS} service decisions.</span></div>{}"#,
            audit_table(audit, keys, callers)
        ),
    };
    let secret = secret.map_or_else(String::new, |token| {
        format!(
            r#"<div class="notice warn" role="status"><strong>New caller token — shown once, not recoverable afterwards.</strong><code class="secret-value">{}</code></div>"#,
            html::escape(token)
        )
    });
    format!(
        "{}{}{}{}{}",
        page_head(),
        tabs(tab),
        banner,
        secret,
        content
    )
}

fn page_head() -> String {
    html::page_head(
        "Signer",
        "Remote custody identities, policy boundaries and accountable access.",
        "",
    )
}

fn overview_panel(keys: &[KeyPublic], callers: &[Caller], audit: &[AuditRow]) -> String {
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

pub(super) fn custody_notice(custody: &Custody) -> String {
    let registry = registry_notice(custody);
    if let Some(signer) = custody.local_wallet_signer() {
        let key = signer.key_info();
        return format!(
            "{}{}",
            registry,
            custody_band(
                "",
                "Local encrypted wallet ready",
                &format!(
                    "Account {} · Neo N3 transaction signing only · encrypted wallet fingerprint {}. No public relay or remote-admin surface is enabled.",
                    key.address,
                    fingerprint(signer.wallet_sha256())
                ),
            )
        );
    }
    let client = match custody.client() {
        Ok(client) => client,
        Err(problem) => {
            return format!(
                "{}{}",
                registry,
                custody_band("danger", "Custody is not configured", &problem.to_string())
            )
        }
    };
    let config = client.config();
    let kind = custody.kind().unwrap_or(SignerBackendKind::NeoOsService);
    let title = match kind {
        SignerBackendKind::LocalSigner => "Local signer connected",
        SignerBackendKind::NeoOsService => "Remote custody connected · NeoOS signer service",
        SignerBackendKind::LocalWallet => "Local encrypted wallet ready",
    };
    let location = match kind {
        SignerBackendKind::LocalSigner => "This machine · loopback",
        SignerBackendKind::NeoOsService => "Remote service",
        SignerBackendKind::LocalWallet => "This machine",
    };
    let mut notice = match config.admin() {
        Some(_) => custody_band(
            "",
            title,
            &format!(
                "{location} · service {} · NeoNexus holds only its admin credential.",
                config.base_url()
            ),
        ),
        None => custody_band(
            "danger",
            "Custody credential missing",
            &format!(
                "Service {} is configured, but this workbench has no credential to read it.",
                config.base_url()
            ),
        ),
    };
    if config.uses_cleartext() {
        notice.push_str(&html::notice(
            "warn",
            "This loopback service uses plain HTTP. Put an authenticated TLS proxy in front of custody before moving it to another host.",
        ));
    }
    format!("{registry}{notice}")
}

fn registry_notice(custody: &Custody) -> String {
    let profiles = custody.profiles().collect::<Vec<_>>();
    if profiles.is_empty() {
        return String::new();
    }
    let registry = custody.registry();
    let rows = profiles
        .iter()
        .map(|profile| {
            let mut roles = Vec::new();
            if registry.console_backend_id() == Some(profile.id.as_str()) {
                roles.push("console");
            }
            if registry.relay_backend_id() == Some(profile.id.as_str()) {
                roles.push("public relay");
            }
            let roles = if roles.is_empty() {
                "available for an explicit node binding".to_string()
            } else {
                roles.join(" + ")
            };
            format!(
                "<li><strong>{}</strong><span>{} · {}</span></li>",
                html::escape(&profile.label),
                html::escape(profile.kind.slug()),
                html::escape(&roles)
            )
        })
        .collect::<String>();
    format!(
        r#"<section class="surface"><div class="section-head"><h2>Signer profiles</h2><span class="muted">{} loaded; every route is explicit and has no fallback.</span></div><ul class="summary-list">{rows}</ul></section>"#,
        profiles.len()
    )
}

pub(super) fn render_local_wallet(
    banner: &str,
    signer: &LocalWalletSigner,
    requested_tab: SignerTab,
) -> String {
    let key = signer.key_info();
    let capabilities = signer.capabilities();
    let unsupported = (requested_tab != SignerTab::Overview).then(|| {
        html::notice(
            "warn",
            "This backend has no remote Keys, Callers, Policy, or Audit control plane. The local wallet overview is shown instead.",
        )
    });
    let rows = [
        ("Backend type", "Local encrypted wallet".to_string()),
        ("Account", key.address.clone()),
        ("Public key", key.public_key.clone()),
        ("Network", key.network.clone()),
        (
            "Network magic",
            key.network_magic
                .map_or_else(|| "—".to_string(), |magic| magic.to_string()),
        ),
        ("Wallet SHA-256", signer.wallet_sha256().to_string()),
        (
            "Transaction signing",
            capability_word(capabilities.neo_n3_transaction).to_string(),
        ),
        (
            "Consensus signing",
            capability_word(capabilities.neo_n3_consensus).to_string(),
        ),
        (
            "Raw signing",
            capability_word(capabilities.neo_n3_raw).to_string(),
        ),
        ("NeoX", "unsupported by NEP-6 / P-256".to_string()),
        ("Public relay", "never exposed".to_string()),
    ]
    .into_iter()
    .map(|(label, value)| html::row(&[html::cell(label), html::cell(&value)]))
    .collect::<Vec<_>>();
    format!(
        "{}{}{}{}{}",
        page_head_local(),
        banner,
        unsupported.unwrap_or_default(),
        html::notice(
            "",
            "The password is read once from a protected local file and is not retained. One zeroizing private-key allocation remains in this process until the profile is dropped; it is never stored in neonexus.db, rendered in HTML, or accepted from a browser. The wallet is hash-pinned and rechecked before each operation."
        ),
        html::table(&["Property", "Value"], &rows),
    )
}

fn page_head_local() -> String {
    html::page_head(
        "Signing backend",
        "Explicit local-wallet backend with a narrow Neo N3 capability set.",
        "",
    )
}

fn capability_word(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

fn fingerprint(value: &str) -> String {
    let prefix = value.chars().take(12).collect::<String>();
    format!("{prefix}…")
}

fn custody_band(kind: &str, title: &str, detail: &str) -> String {
    format!(
        r#"<div class="custody-band {kind}"><span class="custody-mark" aria-hidden="true">S</span><div><strong>{title}</strong><p>{detail}</p></div></div>"#,
        kind = html::escape(kind),
        title = html::escape(title),
        detail = html::escape(detail),
    )
}

fn key_table(keys: &[KeyPublic]) -> String {
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

pub(super) fn key_chain_identity(key: &KeyPublic) -> String {
    match (key.chain_family.as_deref(), key.chain_id) {
        (Some(family), Some(chain_id)) => format!("{family} · {chain_id}"),
        (Some(family), None) => family.to_string(),
        (None, Some(chain_id)) => format!("chain {chain_id}"),
        (None, None) => "not stated by the service".to_string(),
    }
}

fn state_badge(enabled: bool) -> String {
    let (class, label) = if enabled {
        ("running", "Enabled")
    } else {
        ("stopped", "Disabled")
    };
    format!(r#"<span class="badge {class}">{label}</span>"#)
}

pub(super) fn key_actions(key: &KeyPublic) -> String {
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

fn new_key_form() -> String {
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

fn field_with_id(id: &str, label: &str, name: &str, value: &str, help: Option<&str>) -> String {
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

fn caller_table(callers: &[Caller]) -> String {
    if callers.is_empty() {
        return html::empty_state(
            "No callers registered",
            "A key with no caller is held closed. Add only the workload identities that need to ask custody.",
            "",
        );
    }
    let rows = callers
        .iter()
        .map(|caller| {
            let path = html::urlencoding_lite(&caller.id);
            let rotate = match caller.auth_mode.as_deref() {
                None | Some("bearer") => format!(
                    r#"<a class="btn small danger" href="/signer/callers/{path}/rotate">Rotate token</a>"#
                ),
                Some(mode) => format!(
                    r#"<span class="muted">{} identity: no bearer token to rotate</span>"#,
                    html::escape(mode)
                ),
            };
            html::row(&[
                html::cell(&caller.label),
                html::cell(&caller_identity(caller)),
                html::cell(&caller.capabilities.join(" + ")),
                html::cell(&grant_summary(&caller.key_grant)),
                html::cell(&origins_summary(&caller.allowed_origins)),
                html::raw_cell(&state_badge(!caller.disabled)),
                html::raw_cell(&format!(
                    r#"<div class="row-actions">{}{}<a class="btn small danger" href="/signer/callers/{path}/delete">Delete</a></div>"#,
                    html::control_form(
                        &format!("/signer/callers/{path}/state"),
                        &[("disabled", if caller.disabled { "false" } else { "true" })],
                        if caller.disabled { "Enable" } else { "Disable" }
                    ),
                    rotate,
                )),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Label", "Identity", "May", "Keys", "Origins", "Status", "Actions",
        ],
        &rows,
    )
}

fn caller_identity(caller: &Caller) -> String {
    match caller.auth_mode.as_deref() {
        Some("workload-ed25519") => caller.workload_subject.as_deref().map_or_else(
            || "workload-ed25519".to_string(),
            |subject| format!("workload-ed25519 · {subject}"),
        ),
        Some(mode) => mode.to_string(),
        None => "bearer (legacy signer)".to_string(),
    }
}

fn grant_summary(grant: &Grant) -> String {
    if grant.is_any() {
        "any key".to_string()
    } else {
        match grant.key_ids.len() {
            0 => "no keys".to_string(),
            count => format!("{count} keys"),
        }
    }
}

fn origins_summary(origins: &[String]) -> String {
    if origins.is_empty() {
        "server-to-server (no browser origin accepted)".to_string()
    } else {
        origins.join(", ")
    }
}

fn key_options(keys: &[KeyPublic]) -> String {
    keys.iter()
        .map(|key| {
            format!(
                r#"<option value="{}">{}</option>"#,
                html::escape(&key.key_id),
                html::escape(&format!("{} — {}", key.label, key.address))
            )
        })
        .collect()
}

pub(super) fn new_caller_form(keys: &[KeyPublic]) -> String {
    format!(
        r#"<div class="panel"><h3>Add a bearer caller</h3><p class="muted">The token is shown once and is never stored in recoverable form.</p>
<form method="post" action="/signer/callers"><div class="grid">{label}
<label class="field"><span>May</span><select name="capability"><option value="sign">Sign — ask a granted key</option><option value="admin">Admin — configure custody</option></select></label>
<label class="field"><span>Keys</span><select name="grant"><option value="any">Any key</option><option value="only">Only chosen keys</option></select></label>
<label class="field span-all"><span>Granted keys</span><select name="keys" multiple size="4">{options}</select><span class="help">Choosing none under Only chosen keys grants none.</span></label>
<label class="field span-all"><span>Allowed origins</span><input name="origins" class="mono" placeholder="https://relayer.example.com"><span class="help">Comma separated; blank is server-to-server only.</span></label></div>
<button type="submit">Create caller</button></form></div>"#,
        label = field_with_id("bearer-caller-label", "Label", "label", "", None),
        options = key_options(keys),
    )
}

pub(super) fn new_workload_caller_form(keys: &[KeyPublic]) -> String {
    format!(
        r#"<div class="panel"><h3>Add a workload identity</h3><p class="muted">Only its Ed25519 public key enters NeoNexus; the private key remains in the workload.</p>
<form method="post" action="/signer/callers/workload"><div class="grid">{label}
<label class="field"><span>May</span><select name="capability"><option value="sign">Sign parsed transactions</option><option value="raw_sign">Sign raw bytes</option><option value="admin">Administer custody</option></select></label>
<label class="field"><span>Keys</span><select name="grant"><option value="only">Only chosen keys</option><option value="any">Any key</option></select></label>
<label class="field span-all"><span>Granted keys</span><select name="keys" multiple size="4">{options}</select></label>
{public_key}{subject}
<label class="field span-all"><span>Allowed origins</span><input name="origins" class="mono" placeholder="leave blank for a non-browser workload"></label></div>
<button type="submit">Register workload</button></form></div>"#,
        label = field_with_id("workload-caller-label", "Label", "label", "", None),
        options = key_options(keys),
        public_key = field(
            "Ed25519 public key",
            "workload_public_key",
            "",
            Some("Exactly 32 bytes as 64 hexadecimal characters.")
        ),
        subject = field(
            "Workload subject",
            "workload_subject",
            "",
            Some("Optional stable deployment identity.")
        ),
    )
}

fn api_reference() -> String {
    r#"<details class="panel"><summary>Caller API reference</summary><p class="muted">Caller credentials, origins, grants and policy are checked and audited by the custody service.</p><pre>POST /signer/api/v1/sign/transaction
POST /signer/api/v1/sign/consensus
POST /signer/api/v1/sign/eip191-fulfillment
POST /signer/api/v1/sign/raw
GET  /signer/api/v1/keys/{key_id}

Authorization: Bearer &lt;caller token&gt;</pre></details>"#
        .to_string()
}

pub(super) fn audit_table(rows: &[AuditRow], keys: &[KeyPublic], callers: &[Caller]) -> String {
    if rows.is_empty() {
        return html::note("Nothing has been asked of this service yet.");
    }
    let body = rows
        .iter()
        .map(|row| {
            html::row(&[
                html::raw_cell(&time::time_cell(Some(row.recorded_at_unix))),
                html::cell(&row.action),
                html::raw_cell(&outcome_badge(&row.outcome)),
                html::cell(&name(keys, callers, row.key_id.as_deref(), true)),
                html::cell(&name(keys, callers, row.caller_id.as_deref(), false)),
                html::cell(row.origin.as_deref().unwrap_or("—")),
                html::cell(row.reason.as_deref().unwrap_or("")),
                html::cell(row.detail.as_deref().unwrap_or("")),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Time", "Event", "Result", "Key", "Caller", "Origin", "Code", "Detail",
        ],
        &body,
    )
}

fn name(keys: &[KeyPublic], callers: &[Caller], id: Option<&str>, is_key: bool) -> String {
    let Some(id) = id else {
        return "—".to_string();
    };
    let label = if is_key {
        keys.iter()
            .find(|key| key.key_id == id)
            .map(|key| key.label.clone())
    } else {
        callers
            .iter()
            .find(|caller| caller.id == id)
            .map(|caller| caller.label.clone())
    };
    label.map_or_else(|| id.to_string(), |label| format!("{label} ({id})"))
}

fn outcome_badge(outcome: &str) -> String {
    let class = match outcome {
        "allowed" => "running",
        "failed" => "error",
        _ => "stopped",
    };
    format!(
        r#"<span class="badge {class}">{}</span>"#,
        html::escape(outcome)
    )
}
