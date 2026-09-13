//! Form data transfer objects and parser validation for signer controls.

use std::collections::BTreeMap;

use anyhow::{anyhow, bail, Context, Result};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::signer_client::{AssetLimit, ContractMethod, GenerateKeyRequest, Policy};
pub use crate::signer_client::{Grant, SignatureRateLimit, WindowLimit};

#[derive(Deserialize)]
pub struct NewKeyForm {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub network: String,
    #[serde(default)]
    pub network_magic: String,
    #[serde(default)]
    pub chain_family: String,
    #[serde(default)]
    pub chain_id: String,
}

/// A kill switch, sent as an explicit `true`/`false` rather than derived from a
/// checkbox being present or absent: an input dropped by a browser extension
/// must not quietly flip the direction of the request.
#[derive(Deserialize)]
pub struct SwitchForm {
    #[serde(default)]
    pub disabled: String,
}

#[derive(Deserialize)]
pub struct CallerForm {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub grant: String,
    #[serde(default)]
    pub keys: Vec<String>,
    /// `sign` or `admin`, never both and never neither — the rule is the
    /// service's, and it refuses a body that breaks it before it mints a token.
    #[serde(default)]
    pub capability: String,
    #[serde(default)]
    pub origins: String,
}

/// Register a workload by its public Ed25519 identity. This form intentionally
/// has no private-key, passphrase, or bearer-token field: proof generation stays
/// in the workload and this control plane only provisions what the signer needs
/// to verify it.
#[derive(Default)]
pub struct WorkloadCallerForm {
    pub label: String,
    pub grant: String,
    pub keys: Vec<String>,
    pub capability: String,
    pub origins: String,
    pub workload_public_key: String,
    pub workload_subject: String,
}

#[derive(Deserialize, Default)]
pub struct PolicyForm {
    #[serde(default)]
    pub allow_consensus: String,
    #[serde(default)]
    pub allow_raw: String,
    #[serde(default)]
    pub allow_transfer: String,
    #[serde(default)]
    pub allow_contract_call: String,
    #[serde(default)]
    pub allow_global_scope: String,
    #[serde(default)]
    pub contract_whitelist: String,
    #[serde(default)]
    pub contract_blacklist: String,
    #[serde(default)]
    pub contract_method_whitelist: String,
    #[serde(default)]
    pub contract_method_blacklist: String,
    #[serde(default)]
    pub asset_whitelist: String,
    #[serde(default)]
    pub asset_blacklist: String,
    #[serde(default)]
    pub asset_limits: String,
    #[serde(default)]
    pub transfer_to_whitelist: String,
    #[serde(default)]
    pub transfer_to_blacklist: String,
    #[serde(default)]
    pub max_single_amount: String,
    #[serde(default)]
    pub window_seconds: String,
    #[serde(default)]
    pub window_max_amount: String,
    #[serde(default)]
    pub max_signers: String,
    #[serde(default)]
    pub max_system_fee: String,
    #[serde(default)]
    pub max_network_fee: String,
    #[serde(default)]
    pub signature_window_seconds: String,
    #[serde(default)]
    pub signature_window_count: String,
    #[serde(default)]
    pub chain_family: String,
    #[serde(default)]
    pub evm_max_gas_price: String,
    #[serde(default)]
    pub evm_max_gas_limit: String,
    #[serde(default)]
    pub evm_method_whitelist: String,
    #[serde(default)]
    pub evm_method_blacklist: String,
    #[serde(default)]
    pub evm_chain_id: String,
    /// Additive policy fields returned by a newer signer. The key page posts
    /// them back as JSON so saving a known field cannot erase an unknown one.
    #[serde(default)]
    pub additional_fields: String,
}

/// HTML form decoders based on `serde_urlencoded` cannot represent repeated
/// fields as a `Vec`, while a multi-select emits one `keys=` pair per selected
/// key. Read this one form explicitly so the operator's complete grant reaches
/// custody, and reject ambiguous duplicate scalar fields rather than silently
/// choosing the first or last value.
pub fn workload_caller_form(body: &[u8]) -> Result<WorkloadCallerForm> {
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

/// The five boundary switches, read as words rather than checkbox presence. A
/// missing field is an error, not a `false`: an unposted switch is more likely a
/// broken form than a decision, and guessing either way could open a boundary.
pub fn switch(raw: &str, field: &str) -> Result<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "enabled" | "true" => Ok(true),
        "disabled" | "false" => Ok(false),
        other => bail!("{field} must say enabled or disabled, found {other:?}"),
    }
}

pub fn flag(raw: &str, field: &str) -> Result<bool> {
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
pub fn network_magic(raw: &str) -> Result<Option<u32>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed.parse().with_context(|| {
        format!("network magic must be a whole number, found {trimmed:?}")
    })?))
}

pub fn generate_key_request(form: &NewKeyForm) -> Result<GenerateKeyRequest> {
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
pub fn grant(raw: &str, keys: &[String]) -> Result<Grant> {
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
pub fn entries(raw: &str) -> Vec<String> {
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
pub fn amount(raw: &str) -> Option<&str> {
    let raw = raw.trim();
    (!raw.is_empty()).then_some(raw)
}

pub fn optional_number<T>(raw: &str, field: &str) -> Result<Option<T>>
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

pub fn json_list<T>(raw: &str, field: &str) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(raw).with_context(|| format!("{field} must be a JSON array"))
}

pub fn additional_policy_fields(raw: &str) -> Result<BTreeMap<String, Value>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(BTreeMap::new());
    }
    serde_json::from_str(raw).context("additional signer policy fields must be a JSON object")
}

pub fn build_policy(form: &PolicyForm) -> Result<Policy> {
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
