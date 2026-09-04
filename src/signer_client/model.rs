//! Wire types for the NeoOS signer API.
//!
//! These types describe transport shape only. Policy meaning remains owned by
//! the signer service.

use std::{fmt, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize};
use zeroize::Zeroizing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignerOutcome<T> {
    Allowed(T),
    Refused(SignerRefusal),
}

impl<T> SignerOutcome<T> {
    pub fn allowed(self) -> Option<T> {
        match self {
            Self::Allowed(value) => Some(value),
            Self::Refused(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerRefusal {
    pub status: u16,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SignerHealth {
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignRequest {
    pub key_id: String,
    pub unsigned_hex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    /// Required for NeoX transaction signing; omitted for Neo N3.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RawSignRequest {
    pub key_id: String,
    pub data_hex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Local family guard only. The N3-only raw route has no chain-family field.
    #[serde(skip)]
    pub chain_family: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SignedWitness {
    pub key_id: String,
    pub script_hash: String,
    pub address: String,
    pub digest: String,
    pub invocation_script: String,
    pub verification_script: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    /// Complete serialized EVM transaction, ready for eth_sendRawTransaction.
    pub signed_transaction: Option<String>,
    /// Deprecated service alias for signed_transaction, not a bare signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RawSignature {
    pub key_id: String,
    pub script_hash: String,
    pub address: String,
    pub digest: String,
    pub signature: String,
    pub public_key: String,
    pub invocation_script: String,
    pub verification_script: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GenerateKeyRequest {
    pub label: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_magic: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    /// NeoX chain identity, including custom private-chain ids.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SignerKey {
    pub key_id: String,
    pub label: String,
    pub network: String,
    /// Absent on older Neo N3 signer versions.
    pub chain_family: Option<String>,
    pub chain_id: Option<u64>,
    pub network_magic: u32,
    pub public_key: String,
    pub script_hash: String,
    pub address: String,
    pub verification_script: String,
    pub signing_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StateRequest {
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SignerPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    pub allow_consensus: bool,
    pub allow_transfer: bool,
    pub allow_contract_call: bool,
    pub allow_global_scope: bool,
    pub allow_raw: bool,
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
    // EVM-specific policy fields
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractMethod {
    pub contract: String,
    pub method: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowLimit {
    pub seconds: u64,
    pub max_amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignatureRateLimit {
    pub seconds: u64,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetLimit {
    pub asset: String,
    pub max_single_amount: Option<String>,
    pub window_limit: Option<WindowLimit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PolicyAdvice {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct KeyPolicy {
    #[serde(flatten)]
    pub key: SignerKey,
    pub problems: Vec<PolicyAdvice>,
    pub policy: SignerPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SavedPolicy {
    pub problems: Vec<PolicyAdvice>,
    pub policy: SignerPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct KeyGrant {
    pub mode: String,
    pub key_ids: Vec<String>,
}

impl KeyGrant {
    pub fn any() -> Self {
        Self {
            mode: "any".to_string(),
            key_ids: Vec::new(),
        }
    }

    pub fn only(key_ids: Vec<String>) -> Self {
        Self {
            mode: "only".to_string(),
            key_ids,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateCallerRequest {
    pub label: String,
    pub key_grant: KeyGrant,
    pub capabilities: Vec<String>,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateWorkloadCallerRequest {
    #[serde(flatten)]
    pub caller: CreateCallerRequest,
    pub workload_public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workload_subject: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SignerCaller {
    pub id: String,
    pub label: String,
    pub auth_mode: String,
    pub workload_public_key: Option<String>,
    pub workload_subject: Option<String>,
    pub key_grant: KeyGrant,
    pub capabilities: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub created_at_unix: u64,
    pub disabled: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub struct OneTimeToken(Zeroizing<String>);

impl OneTimeToken {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for OneTimeToken {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Debug for OneTimeToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OneTimeToken([REDACTED])")
    }
}

impl<'de> Deserialize<'de> for OneTimeToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self(Zeroizing::new(value)))
    }
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct CreatedCaller {
    pub caller: SignerCaller,
    pub token: OneTimeToken,
}

impl fmt::Debug for CreatedCaller {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreatedCaller")
            .field("caller", &self.caller)
            .field("token", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreatedWorkloadCaller {
    pub caller: SignerCaller,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OidcCallerRequest {
    pub label: String,
    pub key_grant: KeyGrant,
    pub capabilities: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub oidc_issuer: String,
    pub oidc_audience: String,
    pub oidc_subject_pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiKeyCallerRequest {
    pub label: String,
    pub key_grant: KeyGrant,
    pub capabilities: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub expires_at: Option<u64>,
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct CreatedApiKeyCaller {
    pub caller: SignerCaller,
    pub key_id: String,
    pub secret: String,
}

impl fmt::Debug for CreatedApiKeyCaller {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreatedApiKeyCaller")
            .field("caller", &self.caller)
            .field("key_id", &self.key_id)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct RotatedCaller {
    pub caller_id: String,
    pub token: OneTimeToken,
}

impl fmt::Debug for RotatedCaller {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RotatedCaller")
            .field("caller_id", &self.caller_id)
            .field("token", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RemovedKey {
    pub key_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RemovedCaller {
    pub caller_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AuditEntry {
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuditFilter {
    pub key_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct KeysBody {
    pub keys: Vec<SignerKey>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CallersBody {
    pub callers: Vec<SignerCaller>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AuditBody {
    pub entries: Vec<AuditEntry>,
}
