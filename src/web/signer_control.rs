//! Authenticated workbench controls for non-secret signer management.

use axum::{
    extract::{Form, Path, State},
    http::{header, HeaderValue},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::signer_client::{
    AssetLimit, ContractMethod, CreateCallerRequest, CreateWorkloadCallerRequest,
    GenerateKeyRequest, KeyGrant, SignatureRateLimit, SignerClient, SignerOutcome, SignerPolicy,
    WindowLimit,
};

use super::{html, WebState};

#[derive(Deserialize)]
pub struct GenerateKeyForm {
    #[serde(default)]
    label: String,
    #[serde(default)]
    network: String,
    #[serde(default)]
    network_magic: String,
}

pub async fn generate(
    State(state): State<WebState>,
    Form(input): Form<GenerateKeyForm>,
) -> Response {
    let request = (|| -> anyhow::Result<GenerateKeyRequest> {
        let label = required(&input.label, "label")?;
        let network = input.network.trim().to_ascii_lowercase();
        if !matches!(network.as_str(), "mainnet" | "testnet" | "private") {
            anyhow::bail!("network must be mainnet, testnet or private");
        }
        let network_magic = optional_number::<u32>(&input.network_magic, "network magic")?;
        Ok(GenerateKeyRequest {
            label,
            network,
            network_magic,
        })
    })();
    let outcome = match request {
        Ok(request) => call(&state, move |client| client.generate_key(&request)).await,
        Err(error) => Err(error.to_string()),
    };
    respond(
        "/signer",
        outcome.map(|key| format!("generated key {}", key.key_id)),
    )
}

pub async fn set_key_state(
    State(state): State<WebState>,
    Path(key_id): Path<String>,
    Form(input): Form<StateForm>,
) -> Response {
    let disabled = truthy(&input.disabled);
    let redirect = format!("/signer/keys/{key_id}");
    let outcome = call(&state, move |client| {
        client.set_key_disabled(&key_id, disabled)
    })
    .await;
    respond(
        &redirect,
        outcome.map(|key| {
            format!(
                "key {} {}",
                key.key_id,
                if key.signing_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            )
        }),
    )
}

pub async fn delete_key(State(state): State<WebState>, Path(key_id): Path<String>) -> Response {
    let outcome = call(&state, move |client| client.delete_key(&key_id)).await;
    respond(
        "/signer",
        outcome.map(|removed| format!("deleted key {}", removed.key_id)),
    )
}

#[derive(Deserialize)]
pub struct StateForm {
    #[serde(default)]
    disabled: String,
}

#[derive(Deserialize)]
pub struct CallerForm {
    #[serde(default)]
    label: String,
    #[serde(default)]
    capability: String,
    #[serde(default)]
    grant_mode: String,
    #[serde(default)]
    key_ids: String,
    #[serde(default)]
    allowed_origins: String,
}

#[derive(Deserialize)]
pub struct WorkloadCallerForm {
    #[serde(flatten)]
    caller: CallerForm,
    #[serde(default)]
    workload_public_key: String,
    #[serde(default)]
    workload_subject: String,
}

pub async fn create_caller(
    State(state): State<WebState>,
    Form(input): Form<CallerForm>,
) -> Response {
    let request = caller_request(input);
    let outcome = match request {
        Ok(request) => call(&state, move |client| client.create_caller(&request)).await,
        Err(error) => Err(error.to_string()),
    };
    match outcome {
        Ok(created) => token_response(
            "Signer caller created",
            &created.token,
            &format!(
                "Caller {} was created. Copy this bearer token now; it is not stored or shown again.",
                created.caller.label
            ),
        ),
        Err(error) => respond("/signer", Err(error)),
    }
}

pub async fn create_workload_caller(
    State(state): State<WebState>,
    Form(input): Form<WorkloadCallerForm>,
) -> Response {
    let request = (|| -> anyhow::Result<CreateWorkloadCallerRequest> {
        let caller = caller_request(input.caller)?;
        let public_key = required(&input.workload_public_key, "Ed25519 public key")?;
        if public_key.len() != 64
            || public_key
                .bytes()
                .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
        {
            anyhow::bail!("Ed25519 public key must be 64 lowercase hexadecimal characters");
        }
        Ok(CreateWorkloadCallerRequest {
            caller,
            workload_public_key: public_key,
            workload_subject: optional_text(&input.workload_subject),
        })
    })();
    let outcome = match request {
        Ok(request) => {
            call(&state, move |client| {
                client.create_workload_caller(&request)
            })
            .await
        }
        Err(error) => Err(error.to_string()),
    };
    respond(
        "/signer",
        outcome.map(|created| format!("created workload caller {}", created.caller.label)),
    )
}

pub async fn rotate_caller(
    State(state): State<WebState>,
    Path(caller_id): Path<String>,
) -> Response {
    let outcome = call(&state, move |client| client.rotate_caller(&caller_id)).await;
    match outcome {
        Ok(rotated) => token_response(
            "Signer caller rotated",
            &rotated.token,
            &format!(
                "Caller {} was rotated. Copy this replacement token now.",
                rotated.caller_id
            ),
        ),
        Err(error) => respond("/signer", Err(error)),
    }
}

pub async fn set_caller_state(
    State(state): State<WebState>,
    Path(caller_id): Path<String>,
    Form(input): Form<StateForm>,
) -> Response {
    let disabled = truthy(&input.disabled);
    let outcome = call(&state, move |client| {
        client.set_caller_disabled(&caller_id, disabled)
    })
    .await;
    respond(
        "/signer",
        outcome.map(|caller| {
            format!(
                "caller {} {}",
                caller.label,
                if caller.disabled {
                    "disabled"
                } else {
                    "enabled"
                }
            )
        }),
    )
}

pub async fn delete_caller(
    State(state): State<WebState>,
    Path(caller_id): Path<String>,
) -> Response {
    let outcome = call(&state, move |client| client.delete_caller(&caller_id)).await;
    respond(
        "/signer",
        outcome.map(|removed| format!("deleted caller {}", removed.caller_id)),
    )
}

#[derive(Default, Deserialize)]
pub struct PolicyForm {
    #[serde(default)]
    allow_consensus: String,
    #[serde(default)]
    allow_transfer: String,
    #[serde(default)]
    allow_contract_call: String,
    #[serde(default)]
    allow_global_scope: String,
    #[serde(default)]
    allow_raw: String,
    #[serde(default)]
    contract_whitelist: String,
    #[serde(default)]
    contract_blacklist: String,
    #[serde(default)]
    contract_method_whitelist: String,
    #[serde(default)]
    contract_method_blacklist: String,
    #[serde(default)]
    asset_whitelist: String,
    #[serde(default)]
    asset_blacklist: String,
    #[serde(default)]
    asset_limits: String,
    #[serde(default)]
    transfer_to_whitelist: String,
    #[serde(default)]
    transfer_to_blacklist: String,
    #[serde(default)]
    max_single_amount: String,
    #[serde(default)]
    window_seconds: String,
    #[serde(default)]
    window_max_amount: String,
    #[serde(default)]
    max_signers: String,
    #[serde(default)]
    max_system_fee: String,
    #[serde(default)]
    max_network_fee: String,
    #[serde(default)]
    signature_window_seconds: String,
    #[serde(default)]
    signature_window_count: String,
}

pub async fn save_policy(
    State(state): State<WebState>,
    Path(key_id): Path<String>,
    Form(input): Form<PolicyForm>,
) -> Response {
    let redirect = format!("/signer/keys/{key_id}");
    let policy = parse_policy(input);
    let outcome = match policy {
        Ok(policy) => call(&state, move |client| client.save_policy(&key_id, &policy)).await,
        Err(error) => Err(error.to_string()),
    };
    respond(
        &redirect,
        outcome.map(|saved| {
            if saved.problems.is_empty() {
                "signer boundary saved".to_string()
            } else {
                format!(
                    "signer boundary saved with {} warning(s)",
                    saved.problems.len()
                )
            }
        }),
    )
}

fn parse_policy(input: PolicyForm) -> anyhow::Result<SignerPolicy> {
    let allow_consensus = truthy(&input.allow_consensus);
    let allow_transfer = truthy(&input.allow_transfer);
    let allow_contract_call = truthy(&input.allow_contract_call);
    let allow_global_scope = truthy(&input.allow_global_scope);
    let allow_raw = truthy(&input.allow_raw);
    if allow_raw
        && (allow_consensus || allow_transfer || allow_contract_call || allow_global_scope)
    {
        anyhow::bail!(
            "raw signing requires a dedicated key with no transaction or consensus authority"
        );
    }
    let window_seconds = optional_number::<u64>(&input.window_seconds, "window seconds")?;
    let window_amount = optional_decimal(&input.window_max_amount, "window maximum", 128)?;
    let window_limit = match (window_seconds, window_amount) {
        (None, None) => None,
        (Some(seconds), Some(max_amount)) => Some(WindowLimit {
            seconds,
            max_amount,
        }),
        _ => anyhow::bail!("rolling window seconds and maximum must be set together"),
    };
    let signature_seconds =
        optional_number::<u64>(&input.signature_window_seconds, "signature window seconds")?;
    let signature_count =
        optional_number::<u64>(&input.signature_window_count, "signature window count")?;
    let max_signatures = match (signature_seconds, signature_count) {
        (None, None) => None,
        (Some(seconds), Some(count)) => Some(SignatureRateLimit { seconds, count }),
        _ => anyhow::bail!("signature window seconds and count must be set together"),
    };
    Ok(SignerPolicy {
        allow_consensus,
        allow_transfer,
        allow_contract_call,
        allow_global_scope,
        allow_raw,
        contract_whitelist: list(&input.contract_whitelist),
        contract_blacklist: list(&input.contract_blacklist),
        contract_method_whitelist: methods(&input.contract_method_whitelist)?,
        contract_method_blacklist: methods(&input.contract_method_blacklist)?,
        asset_whitelist: list(&input.asset_whitelist),
        asset_blacklist: list(&input.asset_blacklist),
        asset_limits: parse_asset_limits(&input.asset_limits)?,
        transfer_to_whitelist: list(&input.transfer_to_whitelist),
        transfer_to_blacklist: list(&input.transfer_to_blacklist),
        max_single_amount: optional_decimal(&input.max_single_amount, "single amount", 128)?,
        window_limit,
        max_signers: optional_number::<u16>(&input.max_signers, "maximum signers")?,
        max_system_fee: optional_decimal(&input.max_system_fee, "system fee", 64)?,
        max_network_fee: optional_decimal(&input.max_network_fee, "network fee", 64)?,
        max_signatures,
    })
}

async fn call<T, F>(state: &WebState, operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(SignerClient) -> Result<SignerOutcome<T>, crate::signer_client::SignerClientError>
        + Send
        + 'static,
{
    let client = state
        .signer_client()?
        .ok_or_else(|| "signer integration is not configured".to_string())?;
    match tokio::task::spawn_blocking(move || operation(client)).await {
        Ok(Ok(SignerOutcome::Allowed(value))) => Ok(value),
        Ok(Ok(SignerOutcome::Refused(refusal))) => Err(format!(
            "{} ({}; HTTP {})",
            refusal.message, refusal.code, refusal.status
        )),
        Ok(Err(error)) => Err(error.to_string()),
        Err(_) => Err("signer request task did not finish".to_string()),
    }
}

fn caller_request(input: CallerForm) -> anyhow::Result<CreateCallerRequest> {
    let mode = input.grant_mode.trim().to_ascii_lowercase();
    let key_ids = list(&input.key_ids);
    let key_grant = match mode.as_str() {
        "any" if key_ids.is_empty() => KeyGrant::any(),
        "any" => anyhow::bail!("an Any grant cannot name key ids"),
        "only" => KeyGrant::only(key_ids),
        _ => anyhow::bail!("key grant must be Any or Only"),
    };
    let capability = input.capability.trim().to_ascii_lowercase();
    if !matches!(capability.as_str(), "admin" | "sign" | "raw_sign") {
        anyhow::bail!("capability must be admin, sign, or raw_sign");
    }
    Ok(CreateCallerRequest {
        label: required(&input.label, "label")?,
        key_grant,
        capabilities: vec![capability],
        allowed_origins: list(&input.allowed_origins),
    })
}

fn methods(raw: &str) -> anyhow::Result<Vec<ContractMethod>> {
    list(raw)
        .into_iter()
        .map(|line| {
            let (contract, method) = line
                .split_once(':')
                .ok_or_else(|| anyhow::anyhow!("contract methods use <script-hash>:<method>"))?;
            if contract.trim().is_empty() || method.trim().is_empty() {
                anyhow::bail!("contract methods use <script-hash>:<method>");
            }
            Ok(ContractMethod {
                contract: contract.trim().to_string(),
                method: method.trim().to_string(),
            })
        })
        .collect()
}

fn parse_asset_limits(raw: &str) -> anyhow::Result<Vec<AssetLimit>> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
            if columns.len() != 4 {
                anyhow::bail!(
                    "asset limits use <asset>|<single>|<window-seconds>|<window-maximum>"
                );
            }
            let seconds = optional_number::<u64>(columns[2], "asset window seconds")?;
            let maximum = optional_decimal(columns[3], "asset window maximum", 128)?;
            let window_limit = match (seconds, maximum) {
                (None, None) => None,
                (Some(seconds), Some(max_amount)) => Some(WindowLimit {
                    seconds,
                    max_amount,
                }),
                _ => anyhow::bail!(
                    "asset window seconds and maximum must be set together on each line"
                ),
            };
            Ok(AssetLimit {
                asset: required(columns[0], "asset limit hash")?,
                max_single_amount: optional_decimal(columns[1], "asset single maximum", 128)?,
                window_limit,
            })
        })
        .collect()
}

fn list(raw: &str) -> Vec<String> {
    raw.split([',', '\r', '\n'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn optional_number<T>(raw: &str, label: &str) -> anyhow::Result<Option<T>>
where
    T: std::str::FromStr,
{
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    raw.parse::<T>()
        .map(Some)
        .map_err(|_| anyhow::anyhow!("{label} is not a valid whole number"))
}

fn optional_decimal(raw: &str, label: &str, bits: u16) -> anyhow::Result<Option<String>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    let valid = match bits {
        64 => raw.parse::<i64>().is_ok(),
        _ => raw.parse::<i128>().is_ok(),
    };
    if !valid {
        anyhow::bail!("{label} is not a valid decimal integer");
    }
    Ok(Some(raw.to_string()))
}

fn required(raw: &str, label: &str) -> anyhow::Result<String> {
    let value = raw.trim();
    if value.is_empty() {
        anyhow::bail!("{label} is required");
    }
    Ok(value.to_string())
}

fn optional_text(raw: &str) -> Option<String> {
    let value = raw.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on" | "enabled"
    )
}

fn respond(path: &str, outcome: Result<String, String>) -> Response {
    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("signer operation failed: {error}"),
    };
    Redirect::to(&format!(
        "{}?flash={}",
        path,
        html::urlencoding_lite(&message)
    ))
    .into_response()
}

fn token_response(title: &str, token: &str, message: &str) -> Response {
    let body = format!(
        r#"<h1>{title}</h1>
{notice}
<p><code>{token}</code></p>
<p><a href="/signer">Return to signer</a></p>"#,
        title = html::escape(title),
        notice = html::notice("warn", message),
        token = html::escape(token),
    );
    let mut response = Html(html::layout(title, "signer", "", &body)).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, max-age=0"),
    );
    response
}
