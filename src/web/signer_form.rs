//! The signer policy form: the text the browser posts and the text the page
//! draws.
//!
//! The boundary an operator edits is twenty-nine fields wide, and it has two
//! shapes. The browser's copy is text everywhere; the service's copy is
//! booleans, numbers and lists. Both live here so that a refused save can
//! come back with the operator's own words still in it — an amount this
//! console deliberately does not parse and a JSON list it could not read both
//! have to survive the round trip as the text they were typed as.
//!
//! The field names are constants because three places have to agree on them:
//! the page's `name` attributes, the parser's attribution of a failure to a
//! field, and the contract that says which fields exist at all. A name that
//! drifts would put the error beside the wrong control, which is worse than
//! no error at all.

use serde::Deserialize;

use crate::signer_client::Policy;
use crate::web::form_state::FormDraft;

pub const CHAIN_FAMILY: &str = "chain_family";
pub const ALLOW_CONSENSUS: &str = "allow_consensus";
pub const ALLOW_RAW: &str = "allow_raw";
pub const ALLOW_TRANSFER: &str = "allow_transfer";
pub const ALLOW_CONTRACT_CALL: &str = "allow_contract_call";
pub const ALLOW_GLOBAL_SCOPE: &str = "allow_global_scope";
pub const CONTRACT_WHITELIST: &str = "contract_whitelist";
pub const CONTRACT_BLACKLIST: &str = "contract_blacklist";
pub const CONTRACT_METHOD_WHITELIST: &str = "contract_method_whitelist";
pub const CONTRACT_METHOD_BLACKLIST: &str = "contract_method_blacklist";
pub const ASSET_WHITELIST: &str = "asset_whitelist";
pub const ASSET_BLACKLIST: &str = "asset_blacklist";
pub const ASSET_LIMITS: &str = "asset_limits";
pub const TRANSFER_TO_WHITELIST: &str = "transfer_to_whitelist";
pub const TRANSFER_TO_BLACKLIST: &str = "transfer_to_blacklist";
pub const MAX_SINGLE_AMOUNT: &str = "max_single_amount";
pub const WINDOW_SECONDS: &str = "window_seconds";
pub const WINDOW_MAX_AMOUNT: &str = "window_max_amount";
pub const MAX_SIGNERS: &str = "max_signers";
pub const MAX_SYSTEM_FEE: &str = "max_system_fee";
pub const MAX_NETWORK_FEE: &str = "max_network_fee";
pub const SIGNATURE_WINDOW_SECONDS: &str = "signature_window_seconds";
pub const SIGNATURE_WINDOW_COUNT: &str = "signature_window_count";
pub const EVM_MAX_GAS_PRICE: &str = "evm_max_gas_price";
pub const EVM_MAX_GAS_LIMIT: &str = "evm_max_gas_limit";
pub const EVM_METHOD_WHITELIST: &str = "evm_method_whitelist";
pub const EVM_METHOD_BLACKLIST: &str = "evm_method_blacklist";
pub const EVM_CHAIN_ID: &str = "evm_chain_id";
pub const ADDITIONAL_FIELDS: &str = "additional_fields";

/// Every field the policy form has, once each.
///
/// This is the list a test walks to prove that each name the contract carries
/// reaches its own slot — a field drawn from another field's text would show a
/// boundary the operator is not saving.
pub const POLICY_FIELDS: [&str; 29] = [
    CHAIN_FAMILY,
    ALLOW_CONSENSUS,
    ALLOW_RAW,
    ALLOW_TRANSFER,
    ALLOW_CONTRACT_CALL,
    ALLOW_GLOBAL_SCOPE,
    CONTRACT_WHITELIST,
    CONTRACT_BLACKLIST,
    CONTRACT_METHOD_WHITELIST,
    CONTRACT_METHOD_BLACKLIST,
    ASSET_WHITELIST,
    ASSET_BLACKLIST,
    ASSET_LIMITS,
    TRANSFER_TO_WHITELIST,
    TRANSFER_TO_BLACKLIST,
    MAX_SINGLE_AMOUNT,
    WINDOW_SECONDS,
    WINDOW_MAX_AMOUNT,
    MAX_SIGNERS,
    MAX_SYSTEM_FEE,
    MAX_NETWORK_FEE,
    SIGNATURE_WINDOW_SECONDS,
    SIGNATURE_WINDOW_COUNT,
    EVM_MAX_GAS_PRICE,
    EVM_MAX_GAS_LIMIT,
    EVM_METHOD_WHITELIST,
    EVM_METHOD_BLACKLIST,
    EVM_CHAIN_ID,
    ADDITIONAL_FIELDS,
];

/// The policy boundary as the browser posts it.
///
/// Every field is text, and that is the point: the values are what the
/// operator wrote, not what this console made of them. A form that came back
/// with parsed values would show them something they never typed.
#[derive(Deserialize, Default, Clone, Debug)]
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

impl PolicyForm {
    /// The stored boundary as the form shows it.
    ///
    /// The page draws the form from this, so what an operator sees is what
    /// would be posted back: an amount stays the text the service stored, a
    /// list stays the list the service stored, and a switch stays the word
    /// the select offers.
    pub fn from_policy(policy: &Policy) -> Self {
        let (window_seconds, window_max_amount) = policy.window_limit.as_ref().map_or_else(
            || (String::new(), String::new()),
            |limit| (limit.seconds.to_string(), limit.max_amount.clone()),
        );
        let (signature_seconds, signature_count) = policy.max_signatures.as_ref().map_or_else(
            || (String::new(), String::new()),
            |limit| (limit.seconds.to_string(), limit.count.to_string()),
        );
        Self {
            allow_consensus: word(policy.allow_consensus),
            allow_raw: word(policy.allow_raw),
            allow_transfer: word(policy.allow_transfer),
            allow_contract_call: word(policy.allow_contract_call),
            allow_global_scope: word(policy.allow_global_scope),
            contract_whitelist: policy.contract_whitelist.join(", "),
            contract_blacklist: policy.contract_blacklist.join(", "),
            contract_method_whitelist: json(&policy.contract_method_whitelist),
            contract_method_blacklist: json(&policy.contract_method_blacklist),
            asset_whitelist: policy.asset_whitelist.join(", "),
            asset_blacklist: policy.asset_blacklist.join(", "),
            asset_limits: json(&policy.asset_limits),
            transfer_to_whitelist: policy.transfer_to_whitelist.join(", "),
            transfer_to_blacklist: policy.transfer_to_blacklist.join(", "),
            max_single_amount: policy.max_single_amount.clone().unwrap_or_default(),
            window_seconds,
            window_max_amount,
            max_signers: number(policy.max_signers),
            max_system_fee: policy.max_system_fee.clone().unwrap_or_default(),
            max_network_fee: policy.max_network_fee.clone().unwrap_or_default(),
            signature_window_seconds: signature_seconds,
            signature_window_count: signature_count,
            chain_family: policy.chain_family.clone().unwrap_or_default(),
            evm_max_gas_price: policy.evm_max_gas_price.clone().unwrap_or_default(),
            evm_max_gas_limit: number(policy.evm_max_gas_limit),
            evm_method_whitelist: policy.evm_method_whitelist.join(", "),
            evm_method_blacklist: policy.evm_method_blacklist.join(", "),
            evm_chain_id: number(policy.evm_chain_id),
            additional_fields: json(&policy.additional_fields),
        }
    }
}

impl FormDraft for PolicyForm {
    /// The text a field shows, by the `name` its input carries.
    ///
    /// The page draws every control from a field constant and asks for its
    /// value by that same constant, so the value beside a label cannot be
    /// another field's. A name this form does not have reads as blank rather
    /// than failing: the alternative is a panic in a page renderer, and a
    /// blank control an operator can fill in is the smaller harm.
    fn value(&self, field: &str) -> &str {
        match field {
            CHAIN_FAMILY => &self.chain_family,
            ALLOW_CONSENSUS => &self.allow_consensus,
            ALLOW_RAW => &self.allow_raw,
            ALLOW_TRANSFER => &self.allow_transfer,
            ALLOW_CONTRACT_CALL => &self.allow_contract_call,
            ALLOW_GLOBAL_SCOPE => &self.allow_global_scope,
            CONTRACT_WHITELIST => &self.contract_whitelist,
            CONTRACT_BLACKLIST => &self.contract_blacklist,
            CONTRACT_METHOD_WHITELIST => &self.contract_method_whitelist,
            CONTRACT_METHOD_BLACKLIST => &self.contract_method_blacklist,
            ASSET_WHITELIST => &self.asset_whitelist,
            ASSET_BLACKLIST => &self.asset_blacklist,
            ASSET_LIMITS => &self.asset_limits,
            TRANSFER_TO_WHITELIST => &self.transfer_to_whitelist,
            TRANSFER_TO_BLACKLIST => &self.transfer_to_blacklist,
            MAX_SINGLE_AMOUNT => &self.max_single_amount,
            WINDOW_SECONDS => &self.window_seconds,
            WINDOW_MAX_AMOUNT => &self.window_max_amount,
            MAX_SIGNERS => &self.max_signers,
            MAX_SYSTEM_FEE => &self.max_system_fee,
            MAX_NETWORK_FEE => &self.max_network_fee,
            SIGNATURE_WINDOW_SECONDS => &self.signature_window_seconds,
            SIGNATURE_WINDOW_COUNT => &self.signature_window_count,
            EVM_MAX_GAS_PRICE => &self.evm_max_gas_price,
            EVM_MAX_GAS_LIMIT => &self.evm_max_gas_limit,
            EVM_METHOD_WHITELIST => &self.evm_method_whitelist,
            EVM_METHOD_BLACKLIST => &self.evm_method_blacklist,
            EVM_CHAIN_ID => &self.evm_chain_id,
            ADDITIONAL_FIELDS => &self.additional_fields,
            _ => "",
        }
    }

    /// This form has no checkboxes. Every switch is a select that always posts
    /// a word, so there is no absent-means-false field to report.
    fn checked(&self, _field: &str) -> bool {
        false
    }
}

/// A switch as the select offers it.
///
/// The words are the select's option values, not a translation of the boolean:
/// the form posts back exactly what it drew, so a value the page invented would
/// be one the parser then had to learn.
fn word(flag: bool) -> String {
    if flag { "enabled" } else { "disabled" }.to_string()
}

/// A number as the operator would have typed it.
fn number(value: Option<impl ToString>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

/// A structured field as the JSON the parser reads back.
fn json(value: &impl serde::Serialize) -> String {
    match serde_json::to_string(value) {
        Ok(serialized) => serialized,
        Err(error) => {
            log::error!("NeoNexus could not render a signer policy field: {error}");
            String::new()
        }
    }
}
