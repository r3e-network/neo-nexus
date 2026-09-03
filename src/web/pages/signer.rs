//! NeoOS signer workbench.
//!
//! This page renders only metadata and non-secret management controls. It has
//! no route or form that accepts a Neo private key, passphrase, or bytes to
//! sign.

use axum::{
    extract::{Path, RawQuery, State},
    response::{Html, IntoResponse, Response},
};

use crate::signer_client::{
    AuditEntry, AuditFilter, KeyPolicy, SignerCaller, SignerClient, SignerKey, SignerOutcome,
    SignerPolicy,
};

use super::super::{html, time, WebState};

pub async fn signer(State(state): State<WebState>, RawQuery(query): RawQuery) -> Response {
    let body = match state.signer_client() {
        Ok(Some(client)) => overview(client).await,
        Ok(None) => unconfigured(),
        Err(error) => html::notice(
            "danger",
            &format!("Signer configuration is invalid: {error}"),
        ),
    };
    Html(html::layout(
        "NeoOS signer",
        "signer",
        &html::flash(query.as_deref()),
        &body,
    ))
    .into_response()
}

pub async fn key_detail(
    State(state): State<WebState>,
    Path(key_id): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let body = match state.signer_client() {
        Ok(Some(client)) => {
            let loaded = tokio::task::spawn_blocking(move || client.key_policy(&key_id)).await;
            match loaded {
                Ok(Ok(SignerOutcome::Allowed(detail))) => policy_page(&detail),
                Ok(Ok(SignerOutcome::Refused(refusal))) => refusal_notice(&refusal),
                Ok(Err(error)) => {
                    html::notice("danger", &format!("Signer request failed: {error}"))
                }
                Err(_) => html::notice("danger", "Signer request task did not finish."),
            }
        }
        Ok(None) => unconfigured(),
        Err(error) => html::notice(
            "danger",
            &format!("Signer configuration is invalid: {error}"),
        ),
    };
    Html(html::layout(
        "Signer key boundary",
        "signer",
        &html::flash(query.as_deref()),
        &body,
    ))
    .into_response()
}

pub async fn delete_key_page(Path(key_id): Path<String>) -> Response {
    Html(html::layout(
        "Delete signer key",
        "signer",
        "",
        &delete_form(&key_id),
    ))
    .into_response()
}

async fn overview(client: SignerClient) -> String {
    let endpoint = client.endpoint().to_string();
    let loaded = tokio::task::spawn_blocking(move || Overview::load(&client)).await;
    match loaded {
        Ok(snapshot) => render_overview(&endpoint, snapshot),
        Err(_) => html::notice("danger", "Signer request task did not finish."),
    }
}

struct Overview {
    health: Result<String, String>,
    keys: Result<Vec<SignerKey>, String>,
    callers: Result<Vec<SignerCaller>, String>,
    audit: Result<Vec<AuditEntry>, String>,
}

impl Overview {
    fn load(client: &SignerClient) -> Self {
        Self {
            health: client
                .health()
                .map(|health| health.status)
                .map_err(|error| error.to_string()),
            keys: allowed(client.list_keys()),
            callers: allowed(client.list_callers()),
            audit: allowed(client.audit(&AuditFilter {
                key_id: None,
                limit: Some(50),
            })),
        }
    }
}

fn allowed<T>(
    result: Result<SignerOutcome<T>, crate::signer_client::SignerClientError>,
) -> Result<T, String> {
    match result {
        Ok(SignerOutcome::Allowed(value)) => Ok(value),
        Ok(SignerOutcome::Refused(refusal)) => Err(format!(
            "{} ({}; HTTP {})",
            refusal.message, refusal.code, refusal.status
        )),
        Err(error) => Err(error.to_string()),
    }
}

fn render_overview(endpoint: &str, snapshot: Overview) -> String {
    let health = match &snapshot.health {
        Ok(status) => html::notice(
            if status == "ok" { "ok" } else { "warn" },
            &format!("Signer {status} at {endpoint}"),
        ),
        Err(error) => html::notice(
            "danger",
            &format!("Signer health request failed at {endpoint}: {error}"),
        ),
    };
    let key_count = snapshot.keys.as_ref().map_or(0, Vec::len);
    let caller_count = snapshot.callers.as_ref().map_or(0, Vec::len);
    format!(
        r#"<h1>NeoOS signer</h1>
{boundary}
{health}
{cards}
<h2>Generate an enclave-born key</h2>
{generate}
<h2>Custody keys</h2>
{keys}
<h2>Provision a caller</h2>
{caller_forms}
<h2>Callers</h2>
{callers}
<h2>Recent signer audit</h2>
{audit}"#,
        boundary = html::note(
            "NeoNexus is a client only. Custody, policy decisions, limits, and audit storage stay in the separately deployed Rust signer.",
        ),
        cards = html::cards(&[
            ("Keys", key_count.to_string()),
            ("Callers", caller_count.to_string()),
            (
                "Vault",
                snapshot
                    .health
                    .as_deref()
                    .unwrap_or("unreachable")
                    .to_string(),
            ),
        ]),
        generate = generate_form(),
        keys = result_section(snapshot.keys, key_table),
        caller_forms = caller_forms(),
        callers = result_section(snapshot.callers, caller_table),
        audit = result_section(snapshot.audit, audit_table),
    )
}

fn unconfigured() -> String {
    format!(
        r#"<h1>NeoOS signer</h1>
{notice}
<h2>Configuration</h2>
<p>Set <code>NEONEXUS_SIGNER_URL</code> and exactly one
<code>NEONEXUS_SIGNER_ADMIN_TOKEN_FILE</code> or workload identity profile,
then restart NeoNexus. Cleartext endpoints are accepted only on loopback.</p>
<p>See <code>docs/signer-service.md</code> for the complete contract.</p>"#,
        notice = html::notice(
            "warn",
            "Signer integration is not configured. No local key or signing fallback is available.",
        )
    )
}

fn generate_form() -> String {
    let networks = ["testnet", "mainnet", "private"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    format!(
        r#"<form class="filters" method="post" action="/signer/keys">
{label}
{network}
{magic}
<button type="submit">Generate in signer</button>
</form>
{note}"#,
        label = html::text_field("Label", "label", ""),
        network = html::choice_field("Network", "network", &networks, "testnet"),
        magic = html::TextField {
            label: "Private-network magic",
            name: "network_magic",
            value: "",
            help: Some("Leave blank for mainnet/testnet canonical magic."),
            ..html::TextField::default()
        }
        .render(),
        note = html::note(
            "Generation happens inside custody. NeoNexus never receives the resulting private key.",
        ),
    )
}

fn key_table(keys: Vec<SignerKey>) -> String {
    if keys.is_empty() {
        return html::note("No custody keys are registered.");
    }
    let rows = keys
        .iter()
        .map(|key| {
            let state_label = if key.signing_enabled {
                "Disable"
            } else {
                "Enable"
            };
            let state = html::control_form(
                &format!("/signer/keys/{}/state", html::urlencoding_lite(&key.key_id)),
                &[(
                    "disabled",
                    if key.signing_enabled { "true" } else { "false" },
                )],
                state_label,
            );
            let detail = format!(
                r#"<a href="/signer/keys/{}">Boundary</a>"#,
                html::urlencoding_lite(&key.key_id)
            );
            html::row(&[
                html::cell(&key.label),
                html::cell(&key.network),
                html::cell(&key.network_magic.to_string()),
                html::cell(&key.address),
                html::cell(&short(&key.public_key, 18)),
                html::cell(if key.signing_enabled {
                    "enabled"
                } else {
                    "disabled"
                }),
                html::raw_cell(&detail),
                html::raw_cell(&state),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Label",
            "Network",
            "Magic",
            "Address",
            "Public key",
            "State",
            "Policy",
            "Control",
        ],
        &rows,
    )
}

fn caller_forms() -> String {
    let capabilities = vec![
        "admin".to_string(),
        "sign".to_string(),
        "raw_sign".to_string(),
    ];
    let grants = vec!["only".to_string(), "any".to_string()];
    let common = |action: &str, workload: bool| {
        format!(
            r#"<form class="filters" method="post" action="{action}">
{label}
{capability}
{grant}
{keys}
{origins}
{workload_fields}
<button type="submit">{button}</button>
</form>"#,
            action = action,
            label = html::text_field("Label", "label", ""),
            capability = html::choice_field("Capability", "capability", &capabilities, "sign"),
            grant = html::choice_field("Key grant", "grant_mode", &grants, "only"),
            keys = html::TextField {
                label: "Granted key ids",
                name: "key_ids",
                value: "",
                help: Some("Comma or newline separated; required for an Only grant."),
                ..html::TextField::default()
            }
            .render(),
            origins = html::TextField {
                label: "Browser origins",
                name: "allowed_origins",
                value: "",
                help: Some("Leave blank for a server-to-server caller."),
                ..html::TextField::default()
            }
            .render(),
            workload_fields = if workload {
                format!(
                    "{}{}",
                    html::text_field("Ed25519 public key", "workload_public_key", ""),
                    html::text_field("Pinned subject", "workload_subject", "")
                )
            } else {
                String::new()
            },
            button = if workload {
                "Create workload caller"
            } else {
                "Create bearer caller"
            },
        )
    };
    format!(
        "{}<h3>Proof-of-possession caller</h3>{}",
        common("/signer/callers", false),
        common("/signer/callers/workload", true)
    )
}

fn caller_table(callers: Vec<SignerCaller>) -> String {
    if callers.is_empty() {
        return html::note("No signer callers are registered.");
    }
    let rows = callers
        .iter()
        .map(|caller| {
            let grant = if caller.key_grant.mode == "any" {
                "any".to_string()
            } else {
                caller.key_grant.key_ids.join(", ")
            };
            let action = html::control_form(
                &format!(
                    "/signer/callers/{}/state",
                    html::urlencoding_lite(&caller.id)
                ),
                &[("disabled", if caller.disabled { "false" } else { "true" })],
                if caller.disabled { "Enable" } else { "Disable" },
            );
            html::row(&[
                html::cell(&caller.label),
                html::cell(&caller.auth_mode),
                html::cell(&caller.capabilities.join(", ")),
                html::cell(&grant),
                html::cell(caller.workload_subject.as_deref().unwrap_or("not pinned")),
                html::cell(if caller.disabled {
                    "disabled"
                } else {
                    "enabled"
                }),
                html::raw_cell(&action),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Label",
            "Auth",
            "Capability",
            "Key grant",
            "Subject",
            "State",
            "Control",
        ],
        &rows,
    )
}

fn audit_table(entries: Vec<AuditEntry>) -> String {
    if entries.is_empty() {
        return html::note("The signer audit journal is empty.");
    }
    let rows = entries
        .iter()
        .map(|entry| {
            html::row(&[
                html::raw_cell(&time::time_cell(Some(entry.recorded_at_unix))),
                html::cell(&entry.action),
                html::cell(&entry.outcome),
                html::cell(entry.caller_id.as_deref().unwrap_or("unattributed")),
                html::cell(entry.key_id.as_deref().unwrap_or("whole vault")),
                html::cell(entry.reason.as_deref().unwrap_or("—")),
                html::cell(entry.detail.as_deref().unwrap_or("—")),
            ])
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Time", "Action", "Outcome", "Caller", "Key", "Reason", "Detail",
        ],
        &rows,
    )
}

fn policy_page(detail: &KeyPolicy) -> String {
    let key = &detail.key;
    let policy = &detail.policy;
    let network = format!("{} ({})", key.network, key.network_magic);
    let advice = if detail.problems.is_empty() {
        html::notice("ok", "The signer reports no boundary-shape warnings.")
    } else {
        detail
            .problems
            .iter()
            .map(|problem| html::notice("warn", &format!("{}: {}", problem.code, problem.message)))
            .collect()
    };
    let delete_link = format!(
        r#"<a href="/signer/keys/{}/delete">Review deletion</a>"#,
        html::urlencoding_lite(&key.key_id)
    );
    format!(
        r#"<p><a href="/signer">← Signer</a></p>
<h1>{label}</h1>
{facts}
{advice}
<h2>Functional boundary</h2>
{form}
<h2>Retire key</h2>
{delete_note}
{delete_link}"#,
        label = html::escape(&key.label),
        facts = html::table(
            &["Field", "Value"],
            &[
                html::row(&[html::cell("Key id"), html::cell(&key.key_id)]),
                html::row(&[html::cell("Address"), html::cell(&key.address)]),
                html::row(&[html::cell("Network"), html::cell(&network)]),
                html::row(&[html::cell("Public key"), html::cell(&key.public_key)]),
            ],
        ),
        advice = advice,
        form = policy_form(&key.key_id, policy),
        delete_note = html::note(
            "Deletion removes sealed custody material but preserves signer audit history.",
        ),
        delete_link = delete_link,
    )
}

fn policy_form(key_id: &str, policy: &SignerPolicy) -> String {
    format!(
        r#"<form class="filters" method="post" action="/signer/keys/{key_id}/policy">
{switches}
{contract_allow}
{contract_deny}
{method_allow}
{method_deny}
{asset_allow}
{asset_deny}
{asset_limits}
{recipient_allow}
{recipient_deny}
{single}
{window_seconds}
{window_amount}
{max_signers}
{system_fee}
{network_fee}
{signature_seconds}
{signature_count}
<button type="submit">Save boundary</button>
</form>"#,
        key_id = html::urlencoding_lite(key_id),
        switches = [
            checkbox(
                "allow_consensus",
                "Consensus signatures",
                policy.allow_consensus
            ),
            checkbox("allow_transfer", "NEP-17 transfers", policy.allow_transfer),
            checkbox(
                "allow_contract_call",
                "Other contract calls",
                policy.allow_contract_call,
            ),
            checkbox(
                "allow_global_scope",
                "Global witness scope",
                policy.allow_global_scope,
            ),
            checkbox("allow_raw", "Raw signing", policy.allow_raw),
        ]
        .join(""),
        contract_allow = list_field(
            "Contract allowlist",
            "contract_whitelist",
            &policy.contract_whitelist,
        ),
        contract_deny = list_field(
            "Contract denylist",
            "contract_blacklist",
            &policy.contract_blacklist,
        ),
        method_allow = method_field(
            "Contract-method allowlist",
            "contract_method_whitelist",
            &policy.contract_method_whitelist,
        ),
        method_deny = method_field(
            "Contract-method denylist",
            "contract_method_blacklist",
            &policy.contract_method_blacklist,
        ),
        asset_allow = list_field(
            "Asset allowlist",
            "asset_whitelist",
            &policy.asset_whitelist,
        ),
        asset_deny = list_field("Asset denylist", "asset_blacklist", &policy.asset_blacklist,),
        asset_limits = asset_limit_field(&policy.asset_limits),
        recipient_allow = list_field(
            "Recipient allowlist",
            "transfer_to_whitelist",
            &policy.transfer_to_whitelist,
        ),
        recipient_deny = list_field(
            "Recipient denylist",
            "transfer_to_blacklist",
            &policy.transfer_to_blacklist,
        ),
        single = optional_field(
            "Maximum single transfer (raw units)",
            "max_single_amount",
            policy.max_single_amount.as_deref(),
        ),
        window_seconds = optional_field(
            "Rolling window seconds",
            "window_seconds",
            policy
                .window_limit
                .as_ref()
                .map(|window| window.seconds.to_string())
                .as_deref(),
        ),
        window_amount = optional_field(
            "Rolling maximum (raw units)",
            "window_max_amount",
            policy
                .window_limit
                .as_ref()
                .map(|window| window.max_amount.as_str()),
        ),
        max_signers = optional_field(
            "Maximum transaction signers",
            "max_signers",
            policy.max_signers.map(|value| value.to_string()).as_deref(),
        ),
        system_fee = optional_field(
            "Maximum system fee (GAS fixed-8)",
            "max_system_fee",
            policy.max_system_fee.as_deref(),
        ),
        network_fee = optional_field(
            "Maximum network fee (GAS fixed-8)",
            "max_network_fee",
            policy.max_network_fee.as_deref(),
        ),
        signature_seconds = optional_field(
            "Signature-rate window seconds",
            "signature_window_seconds",
            policy
                .max_signatures
                .as_ref()
                .map(|limit| limit.seconds.to_string())
                .as_deref(),
        ),
        signature_count = optional_field(
            "Maximum signatures per window",
            "signature_window_count",
            policy
                .max_signatures
                .as_ref()
                .map(|limit| limit.count.to_string())
                .as_deref(),
        ),
    )
}

pub fn delete_form(key_id: &str) -> String {
    format!(
        r#"<p><a href="/signer/keys/{id}">← Cancel</a></p>
<h1>Delete custody key?</h1>
{warning}
<form method="post" action="/signer/keys/{id}/delete">
<button type="submit">Delete sealed key</button>
</form>"#,
        id = html::urlencoding_lite(key_id),
        warning = html::notice(
            "danger",
            "This removes the signer's sealed private key and cannot be undone. Audit history remains.",
        ),
    )
}

fn result_section<T>(result: Result<T, String>, render: impl FnOnce(T) -> String) -> String {
    match result {
        Ok(value) => render(value),
        Err(error) => html::notice("danger", &error),
    }
}

fn refusal_notice(refusal: &crate::signer_client::SignerRefusal) -> String {
    html::notice(
        "danger",
        &format!(
            "Signer refused the request: {} ({}; HTTP {})",
            refusal.message, refusal.code, refusal.status
        ),
    )
}

fn checkbox(name: &str, label: &str, checked: bool) -> String {
    format!(
        r#"<label class="field"><span>{label}</span><input type="checkbox" name="{name}" value="true"{checked}></label>"#,
        label = html::escape(label),
        name = html::escape(name),
        checked = if checked { " checked" } else { "" },
    )
}

fn list_field(label: &str, name: &str, values: &[String]) -> String {
    textarea(label, name, &values.join("\n"))
}

fn method_field(
    label: &str,
    name: &str,
    values: &[crate::signer_client::ContractMethod],
) -> String {
    let text = values
        .iter()
        .map(|entry| format!("{}:{}", entry.contract, entry.method))
        .collect::<Vec<_>>()
        .join("\n");
    textarea(label, name, &text)
}

fn asset_limit_field(values: &[crate::signer_client::AssetLimit]) -> String {
    let text = values
        .iter()
        .map(|entry| {
            format!(
                "{}|{}|{}|{}",
                entry.asset,
                entry.max_single_amount.as_deref().unwrap_or_default(),
                entry
                    .window_limit
                    .as_ref()
                    .map(|window| window.seconds.to_string())
                    .unwrap_or_default(),
                entry
                    .window_limit
                    .as_ref()
                    .map(|window| window.max_amount.as_str())
                    .unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    textarea(
        "Asset ceilings (asset|single|window seconds|window maximum)",
        "asset_limits",
        &text,
    )
}

fn textarea(label: &str, name: &str, value: &str) -> String {
    format!(
        r#"<label class="field span-all"><span>{}</span><textarea name="{}" rows="3">{}</textarea></label>"#,
        html::escape(label),
        html::escape(name),
        html::escape(value),
    )
}

fn optional_field(label: &str, name: &str, value: Option<&str>) -> String {
    html::text_field(label, name, value.unwrap_or_default())
}

fn short(value: &str, length: usize) -> String {
    if value.chars().count() <= length {
        value.to_string()
    } else {
        format!("{}…", value.chars().take(length).collect::<String>())
    }
}
