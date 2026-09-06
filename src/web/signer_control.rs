//! Custody controls from the browser: ask the service to generate a key, write
//! its boundary, switch it off, and register the callers that may ask it to
//! sign.
//!
//! Handlers answer the way the rest of the workbench does — a redirect carrying
//! a flash line — with one deliberate exception. `create_caller` and
//! `rotate_caller` render the list page directly, because their result is a
//! token and a token in `?flash=…` would be copied into browser history, the
//! access log, and the next `Referer` header.
//!
//! Every mutation is a POST to the custody service, made with this workbench's
//! own admin credential ([`crate::web::Custody::admin`]). Nothing here reaches
//! for the repository: the keys, the boundaries, the callers and the audit trail
//! are the service's rows, and a control that wrote them locally would produce a
//! page that disagrees with what the vault does — and a mutation with no audit
//! row, which §5.1 says is a mutation nobody can account for afterwards.
//!
//! Two consequences of that boundary shape this file:
//!
//! * The calls are blocking, so each one runs on [`tokio::task::spawn_blocking`]
//!   rather than on the request thread. An operator's stalled custody service
//!   costs that one page a slow response; it does not get to hold an axum worker.
//! * Validation is only about the *form*. Which networks exist, what a script
//!   hash looks like, what an amount may be — the service checks all of those and
//!   says so in its own words, which are then the words in the flash line. What
//!   stays here is the few rules about what a browser submitted at all: a switch
//!   that must say enabled or disabled rather than being guessed from a checkbox,
//!   a kill switch that must say true or false, and a window that cannot be half
//!   a window.

use std::collections::BTreeMap;

use anyhow::{anyhow, bail, Context, Result};
use axum::{
    extract::{Form, Path, RawForm, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::signer_client::{
    AssetLimit, ContractMethod, GenerateKeyRequest, Grant, Policy, SignatureRateLimit, WindowLimit,
    WorkloadCallerRequest,
};

use super::{html, pages::signer, Admin, WebState};

#[derive(Deserialize)]
pub struct NewKeyForm {
    #[serde(default)]
    label: String,
    #[serde(default)]
    network: String,
    #[serde(default)]
    network_magic: String,
    #[serde(default)]
    chain_family: String,
    #[serde(default)]
    chain_id: String,
}

/// A kill switch, sent as an explicit `true`/`false` rather than derived from a
/// checkbox being present or absent: an input dropped by a browser extension
/// must not quietly flip the direction of the request.
#[derive(Deserialize)]
pub struct SwitchForm {
    #[serde(default)]
    disabled: String,
}

#[derive(Deserialize)]
pub struct CallerForm {
    #[serde(default)]
    label: String,
    #[serde(default)]
    grant: String,
    #[serde(default)]
    keys: Vec<String>,
    /// `sign` or `admin`, never both and never neither — the rule is the
    /// service's, and it refuses a body that breaks it before it mints a token.
    #[serde(default)]
    capability: String,
    #[serde(default)]
    origins: String,
}

/// Register a workload by its public Ed25519 identity. This form intentionally
/// has no private-key, passphrase, or bearer-token field: proof generation stays
/// in the workload and this control plane only provisions what the signer needs
/// to verify it.
#[derive(Default)]
pub struct WorkloadCallerForm {
    label: String,
    grant: String,
    keys: Vec<String>,
    capability: String,
    origins: String,
    workload_public_key: String,
    workload_subject: String,
}

#[derive(Deserialize, Default)]
pub struct PolicyForm {
    #[serde(default)]
    allow_consensus: String,
    #[serde(default)]
    allow_raw: String,
    #[serde(default)]
    allow_transfer: String,
    #[serde(default)]
    allow_contract_call: String,
    #[serde(default)]
    allow_global_scope: String,
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
    #[serde(default)]
    chain_family: String,
    #[serde(default)]
    evm_max_gas_price: String,
    #[serde(default)]
    evm_max_gas_limit: String,
    #[serde(default)]
    evm_method_whitelist: String,
    #[serde(default)]
    evm_method_blacklist: String,
    #[serde(default)]
    evm_chain_id: String,
    /// Additive policy fields returned by a newer signer. The key page posts
    /// them back as JSON so saving a known field cannot erase an unknown one.
    #[serde(default)]
    additional_fields: String,
}

// -- keys -------------------------------------------------------------------

pub async fn generate(State(state): State<WebState>, Form(form): Form<NewKeyForm>) -> Response {
    let path = "/signer".to_string();
    manage(&state, path, move |admin| {
        let request = generate_key_request(&form)?;
        let key = admin
            .client()
            .generate_key_request(&admin.credentials()?, &request)?
            .into_parts()?;
        Ok(format!(
            "custody key {} generated — {} (no boundary yet, so it signs nothing)",
            key.label, key.address
        ))
    })
    .await
}

pub async fn set_key_state(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<SwitchForm>,
) -> Response {
    let path = key_path(&id);
    manage(&state, path, move |admin| {
        let disabled = flag(&form.disabled, "signing state")?;
        let key = admin
            .client()
            .set_key_disabled(&admin.credentials()?, &id, disabled)?
            .into_parts()?;
        Ok(format!(
            "{} is now {}",
            key.label,
            if disabled { "disabled" } else { "enabled" }
        ))
    })
    .await
}

pub async fn delete_key(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let path = "/signer".to_string();
    manage(&state, path, move |admin| {
        admin
            .client()
            .delete_key(&admin.credentials()?, &id)?
            .into_parts()?;
        Ok(format!(
            "custody key {id} deleted; its audit history was kept"
        ))
    })
    .await
}

pub async fn save_policy(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<PolicyForm>,
) -> Response {
    let path = key_path(&id);
    manage(&state, path, move |admin| {
        let policy = build_policy(&form)?;
        let stored = admin
            .client()
            .save_policy(&admin.credentials()?, &id, &policy)?
            .into_parts()?;
        // The service's own advice about the shape it just stored, counted rather
        // than spelled out: the page this redirect lands on lists them in full.
        let warning = if stored.problems.is_empty() {
            String::new()
        } else {
            format!(
                " — stored, and {} of its shapes look weaker than intended",
                stored.problems.len()
            )
        };
        let policy = stored.policy;
        Ok(format!(
            "boundary saved for {id}{warning}: consensus {}, raw {}, transfers {}, calls {}, \
             global scope {}",
            word(policy.allow_consensus),
            word(policy.allow_raw),
            word(policy.allow_transfer),
            word(policy.allow_contract_call),
            word(policy.allow_global_scope),
        ))
    })
    .await
}

// -- callers ----------------------------------------------------------------

pub async fn create_caller(
    State(state): State<WebState>,
    Form(form): Form<CallerForm>,
) -> Response {
    let outcome = state
        .custody()
        .ask(move |admin| {
            let grant = grant(&form.grant, &form.keys)?;
            let created = admin
                .client()
                .create_caller(
                    &admin.credentials()?,
                    &form.label,
                    &grant,
                    std::slice::from_ref(&form.capability),
                    &entries(&form.origins),
                )?
                .into_parts()?;
            Ok(created)
        })
        .await;
    match outcome {
        Ok(created) => {
            signer::render(
                &state,
                &format!("caller {} created", created.caller.label),
                Some(&created.token),
            )
            .await
        }
        Err(error) => signer::render(&state, &format!("not saved: {error}"), None).await,
    }
}

pub async fn create_workload_caller(
    State(state): State<WebState>,
    RawForm(body): RawForm,
) -> Response {
    let path = "/signer".to_string();
    let form = match workload_caller_form(&body) {
        Ok(form) => form,
        Err(error) => return respond(&path, Err(error)),
    };
    manage(&state, path, move |admin| {
        let request = WorkloadCallerRequest {
            label: form.label.trim().to_string(),
            key_grant: grant(&form.grant, &form.keys)?,
            capabilities: vec![form.capability.trim().to_string()],
            allowed_origins: entries(&form.origins),
            workload_public_key: form.workload_public_key.trim().to_string(),
            workload_subject: amount(&form.workload_subject).map(str::to_string),
        };
        let created = admin
            .client()
            .create_workload_caller(&admin.credentials()?, &request)?
            .into_parts()?;
        Ok(format!(
            "workload caller {} registered — its Ed25519 private key remains in the workload",
            created.caller.label
        ))
    })
    .await
}

/// HTML form decoders based on `serde_urlencoded` cannot represent repeated
/// fields as a `Vec`, while a multi-select emits one `keys=` pair per selected
/// key. Read this one form explicitly so the operator's complete grant reaches
/// custody, and reject ambiguous duplicate scalar fields rather than silently
/// choosing the first or last value.
fn workload_caller_form(body: &[u8]) -> Result<WorkloadCallerForm> {
    let mut form = WorkloadCallerForm::default();
    let mut seen = std::collections::BTreeSet::new();
    for (name, value) in url::form_urlencoded::parse(body) {
        if name == "keys" {
            form.keys.push(value.into_owned());
            continue;
        }
        let target = match name.as_ref() {
            "label" => Some(&mut form.label),
            "grant" => Some(&mut form.grant),
            "capability" => Some(&mut form.capability),
            "origins" => Some(&mut form.origins),
            "workload_public_key" => Some(&mut form.workload_public_key),
            "workload_subject" => Some(&mut form.workload_subject),
            _ => None,
        };
        let Some(target) = target else {
            continue;
        };
        if !seen.insert(name.into_owned()) {
            bail!("workload caller form contains a duplicate scalar field");
        }
        *target = value.into_owned();
    }
    Ok(form)
}

pub async fn rotate_caller(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let outcome = state
        .custody()
        .ask(move |admin| {
            let rotated = admin
                .client()
                .rotate_caller_token(&admin.credentials()?, &id)?
                .into_parts()?;
            Ok(rotated)
        })
        .await;
    match outcome {
        // The old token stops working at this moment, which is the point the
        // operator has to be told plainly: a rotation that left two live tokens
        // would not be a revocation.
        Ok(rotated) => {
            signer::render(
                &state,
                &format!(
                    "caller {} rotated — its previous token no longer works",
                    rotated.caller_id
                ),
                Some(&rotated.token),
            )
            .await
        }
        Err(error) => signer::render(&state, &format!("not rotated: {error}"), None).await,
    }
}

pub async fn set_caller_state(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<SwitchForm>,
) -> Response {
    let path = "/signer".to_string();
    manage(&state, path, move |admin| {
        let disabled = flag(&form.disabled, "caller state")?;
        let caller = admin
            .client()
            .set_caller_disabled(&admin.credentials()?, &id, disabled)?
            .into_parts()?;
        Ok(format!(
            "{} is now {}",
            caller.label,
            if disabled { "disabled" } else { "enabled" }
        ))
    })
    .await
}

pub async fn delete_caller(State(state): State<WebState>, Path(id): Path<String>) -> Response {
    let path = "/signer".to_string();
    manage(&state, path, move |admin| {
        admin
            .client()
            .delete_caller(&admin.credentials()?, &id)?
            .into_parts()?;
        Ok(format!("caller {id} deleted"))
    })
    .await
}

// -- running a call ---------------------------------------------------------

/// Ask the service, off the request thread, and redirect with its answer.
async fn manage<Call>(state: &WebState, path: String, call: Call) -> Response
where
    Call: FnOnce(&Admin) -> Result<String> + Send + 'static,
{
    respond(&path, state.custody().ask(call).await)
}

// -- parsing ----------------------------------------------------------------

/// The five boundary switches, read as words rather than checkbox presence. A
/// missing field is an error, not a `false`: an unposted switch is more likely a
/// broken form than a decision, and guessing either way could open a boundary.
fn switch(raw: &str, field: &str) -> Result<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "enabled" | "true" => Ok(true),
        "disabled" | "false" => Ok(false),
        other => bail!("{field} must say enabled or disabled, found {other:?}"),
    }
}

fn flag(raw: &str, field: &str) -> Result<bool> {
    match raw.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => bail!("{field} must be sent as true or false, found {other:?}"),
    }
}

/// The optional chain-identity override for a private network: blank means the
/// service's canonical value, anything else has to be the deployment's real
/// magic — a typo here would bind the key to a chain it cannot sign for, so it
/// is refused here rather than discovered after custody.
fn network_magic(raw: &str) -> Result<Option<u32>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed.parse().with_context(|| {
        format!("network magic must be a whole number, found {trimmed:?}")
    })?))
}

fn generate_key_request(form: &NewKeyForm) -> Result<GenerateKeyRequest> {
    Ok(GenerateKeyRequest {
        label: form.label.trim().to_string(),
        network: form.network.trim().to_string(),
        chain_family: amount(&form.chain_family).map(str::to_string),
        chain_id: optional_number(&form.chain_id, "chain id")?,
        network_magic: network_magic(&form.network_magic)?,
    })
}

/// Which keys a caller may name. Anything the select did not send is an error
/// rather than `any`: guessing whole-vault authority from a dropped field is the
/// one mistake this function could make that no later check would catch.
fn grant(raw: &str, keys: &[String]) -> Result<Grant> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "any" => Ok(Grant::any()),
        "only" | "keys" => Ok(Grant::only(
            keys.iter()
                .map(|key| key.trim())
                .filter(|key| !key.is_empty()),
        )),
        other => bail!("grant must be any or only, found {other:?}"),
    }
}

/// Split a pasted list on the separators an operator actually uses: a comma
/// between entries, a newline when the list is long, and the whitespace either
/// side of both.
fn entries(raw: &str) -> Vec<String> {
    raw.split([',', '\n', '\r'])
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}

/// An amount as text, with blank read as "no ceiling".
///
/// Not parsed. §5.1 carries amounts as decimal text precisely because a JSON
/// number cannot hold an `i128`, and the service is the process that says what a
/// ceiling must look like — so `"1e6"` reaches its refusal, in its words, rather
/// than a guess made here.
fn amount(raw: &str) -> Option<&str> {
    let raw = raw.trim();
    (!raw.is_empty()).then_some(raw)
}

fn optional_number<T>(raw: &str, field: &str) -> Result<Option<T>>
where
    T: std::str::FromStr,
{
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    raw.parse::<T>()
        .map(Some)
        .map_err(|_| anyhow!("{field} must be a whole number"))
}

fn json_list<T>(raw: &str, field: &str) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(raw).with_context(|| format!("{field} must be a JSON array"))
}

fn additional_policy_fields(raw: &str) -> Result<BTreeMap<String, Value>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(BTreeMap::new());
    }
    serde_json::from_str(raw).context("additional signer policy fields must be a JSON object")
}

fn build_policy(form: &PolicyForm) -> Result<Policy> {
    let window_limit = match (form.window_seconds.trim(), amount(&form.window_max_amount)) {
        ("", None) => None,
        ("", Some(_)) => bail!("the window needs a length in seconds as well as a ceiling"),
        (_, None) => bail!("the window needs a ceiling as well as a length in seconds"),
        (seconds, Some(max_amount)) => {
            let seconds = seconds
                .parse::<u64>()
                .map_err(|_| anyhow!("window seconds must be a whole number"))?;
            Some(WindowLimit {
                seconds,
                max_amount: max_amount.to_string(),
            })
        }
    };
    let max_signatures = match (
        optional_number::<u64>(&form.signature_window_seconds, "signature window seconds")?,
        optional_number::<u64>(&form.signature_window_count, "signature window count")?,
    ) {
        (None, None) => None,
        (Some(_), None) => bail!("the signature window needs a count as well as a length"),
        (None, Some(_)) => bail!("the signature window needs a length as well as a count"),
        (Some(seconds), Some(count)) => Some(SignatureRateLimit { seconds, count }),
    };
    Ok(Policy {
        allow_consensus: switch(&form.allow_consensus, "consensus payloads")?,
        allow_raw: switch(&form.allow_raw, "raw signing")?,
        allow_transfer: switch(&form.allow_transfer, "transfers")?,
        allow_contract_call: switch(&form.allow_contract_call, "contract calls")?,
        allow_global_scope: switch(&form.allow_global_scope, "global scope")?,
        contract_whitelist: entries(&form.contract_whitelist),
        contract_blacklist: entries(&form.contract_blacklist),
        contract_method_whitelist: json_list::<ContractMethod>(
            &form.contract_method_whitelist,
            "contract method whitelist",
        )?,
        contract_method_blacklist: json_list::<ContractMethod>(
            &form.contract_method_blacklist,
            "contract method blacklist",
        )?,
        asset_whitelist: entries(&form.asset_whitelist),
        asset_blacklist: entries(&form.asset_blacklist),
        asset_limits: json_list::<AssetLimit>(&form.asset_limits, "asset limits")?,
        transfer_to_whitelist: entries(&form.transfer_to_whitelist),
        transfer_to_blacklist: entries(&form.transfer_to_blacklist),
        max_single_amount: amount(&form.max_single_amount).map(str::to_string),
        window_limit,
        max_signers: optional_number(&form.max_signers, "maximum signers")?,
        max_system_fee: amount(&form.max_system_fee).map(str::to_string),
        max_network_fee: amount(&form.max_network_fee).map(str::to_string),
        max_signatures,
        chain_family: amount(&form.chain_family).map(str::to_string),
        evm_max_gas_price: amount(&form.evm_max_gas_price).map(str::to_string),
        evm_max_gas_limit: optional_number(&form.evm_max_gas_limit, "EVM maximum gas limit")?,
        evm_method_whitelist: entries(&form.evm_method_whitelist),
        evm_method_blacklist: entries(&form.evm_method_blacklist),
        evm_chain_id: optional_number(&form.evm_chain_id, "EVM chain id")?,
        additional_fields: additional_policy_fields(&form.additional_fields)?,
    })
}

// -- responses --------------------------------------------------------------

fn key_path(id: &str) -> String {
    format!("/signer/keys/{}", html::urlencoding_lite(id))
}

fn word(flag: bool) -> &'static str {
    if flag {
        "allowed"
    } else {
        "closed"
    }
}

fn respond(path: &str, outcome: Result<String>) -> Response {
    let message = match outcome {
        Ok(message) => message,
        Err(error) => format!("not saved: {error}"),
    };
    Redirect::to(&format!(
        "{path}?flash={}",
        html::urlencoding_lite(&message),
        path = path
    ))
    .into_response()
}

#[cfg(test)]
#[path = "../../tests/unit/web/signer_control/tests.rs"]
mod tests;
