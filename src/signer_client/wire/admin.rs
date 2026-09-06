//! §5.1 key, policy, caller, and audit transport models.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A key's public identity. Private key material has no representation here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyPublic {
    pub key_id: String,
    pub label: String,
    pub network: String,
    pub network_magic: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    pub public_key: String,
    pub script_hash: String,
    pub address: String,
    pub verification_script: String,
    pub signing_enabled: bool,
    /// Additive key metadata survives rendering and public relaying.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

/// Public key identity plus its normalized stored policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBoundary {
    #[serde(flatten)]
    pub key: KeyPublic,
    #[serde(default)]
    pub problems: Vec<PolicyAdvice>,
    pub policy: Policy,
}

/// Response to `POST /keys/{id}/policy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedBoundary {
    #[serde(default)]
    pub problems: Vec<PolicyAdvice>,
    pub policy: Policy,
}

/// Service-authored advice about a stored boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyAdvice {
    pub code: String,
    pub message: String,
}

/// A custody policy. Default/blank means every capability is closed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Policy {
    pub allow_consensus: bool,
    pub allow_raw: bool,
    pub allow_transfer: bool,
    pub allow_contract_call: bool,
    pub allow_global_scope: bool,
    pub contract_whitelist: Vec<String>,
    pub contract_blacklist: Vec<String>,
    pub contract_method_whitelist: Vec<ContractMethod>,
    pub contract_method_blacklist: Vec<ContractMethod>,
    pub asset_whitelist: Vec<String>,
    pub asset_blacklist: Vec<String>,
    pub asset_limits: Vec<AssetLimit>,
    pub transfer_to_whitelist: Vec<String>,
    pub transfer_to_blacklist: Vec<String>,
    pub max_single_amount: Option<String>,
    pub window_limit: Option<WindowLimit>,
    pub max_signers: Option<u16>,
    pub max_system_fee: Option<String>,
    pub max_network_fee: Option<String>,
    pub max_signatures: Option<SignatureRateLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_max_gas_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_max_gas_limit: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evm_method_whitelist: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evm_method_blacklist: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_chain_id: Option<u64>,
    /// Compatibility seam for additive policy fields from a newer signer.
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractMethod {
    pub contract: String,
    pub method: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowLimit {
    pub seconds: u64,
    pub max_amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetLimit {
    pub asset: String,
    pub max_single_amount: Option<String>,
    pub window_limit: Option<WindowLimit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureRateLimit {
    pub seconds: u64,
    pub count: u64,
}

/// Which keys a caller may name: `any` or an explicit canonicalized set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    pub mode: String,
    #[serde(default)]
    pub key_ids: Vec<String>,
}

impl Grant {
    pub const ANY: &'static str = "any";

    pub fn any() -> Self {
        Self {
            mode: Self::ANY.to_string(),
            key_ids: Vec::new(),
        }
    }

    pub fn only(key_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut key_ids: Vec<String> = key_ids.into_iter().map(Into::into).collect();
        key_ids.sort();
        key_ids.dedup();
        Self {
            mode: "only".to_string(),
            key_ids,
        }
    }

    pub fn is_any(&self) -> bool {
        self.mode == Self::ANY
    }
}

/// A registered caller and its signer-owned authorization scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Caller {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workload_public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workload_subject: Option<String>,
    pub key_grant: Grant,
    pub capabilities: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub created_at_unix: u64,
    pub disabled: bool,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

/// Bearer caller plus its one-time plaintext credential.
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CreatedCaller {
    pub caller: Caller,
    pub token: String,
}

impl std::fmt::Debug for CreatedCaller {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CreatedCaller")
            .field("caller", &self.caller)
            .field("token", &"<redacted>")
            .finish()
    }
}

/// Workload caller response; no recoverable bearer credential is minted.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CreatedWorkloadCaller {
    pub caller: Caller,
}

/// Public Ed25519 workload identity registration request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkloadCallerRequest {
    pub label: String,
    pub key_grant: Grant,
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_origins: Vec<String>,
    pub workload_public_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workload_subject: Option<String>,
}

/// Rotated one-time plaintext bearer credential.
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RotatedCaller {
    pub caller_id: String,
    pub token: String,
}

impl std::fmt::Debug for RotatedCaller {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RotatedCaller")
            .field("caller_id", &self.caller_id)
            .field("token", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovedKey {
    pub key_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovedCaller {
    pub caller_id: String,
}

/// Privileged signer audit record. This is the only response carrying `detail`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRow {
    pub id: i64,
    pub recorded_at_unix: u64,
    pub action: String,
    pub outcome: String,
    pub caller_id: Option<String>,
    pub key_id: Option<String>,
    pub tx_id: Option<String>,
    pub reason: Option<String>,
    pub detail: Option<String>,
    pub origin: Option<String>,
}
