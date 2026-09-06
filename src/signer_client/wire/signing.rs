//! §5 signing and key-generation transport models.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One semantic signing request. Optional fields remain absent for legacy Neo
/// N3 callers while newer callers can bind idempotency and chain identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignRequest {
    pub key_id: String,
    pub unsigned_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
}

impl SignRequest {
    pub fn neo_n3(key_id: impl Into<String>, unsigned_hex: impl Into<String>) -> Self {
        Self {
            key_id: key_id.into(),
            unsigned_hex: unsigned_hex.into(),
            request_id: None,
            chain_family: None,
            chain_id: None,
        }
    }
}

/// Raw signing has the same idempotency key but deliberately no chain identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSignRequest {
    pub key_id: String,
    pub data_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl RawSignRequest {
    pub fn new(key_id: impl Into<String>, data_hex: impl Into<String>) -> Self {
        Self {
            key_id: key_id.into(),
            data_hex: data_hex.into(),
            request_id: None,
        }
    }
}

/// Semantic NeoX oracle fields committed to by the signer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Eip191Fulfillment {
    pub request_id: String,
    pub app_id: String,
    pub module_id: String,
    pub operation: String,
    pub success: bool,
    #[serde(default)]
    pub error: String,
}

/// `POST /sign/eip191-fulfillment`; no caller-supplied digest or prehash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Eip191FulfillmentRequest {
    pub key_id: String,
    pub request_id: String,
    pub chain_id: u64,
    pub oracle_contract: String,
    pub fulfillment: Eip191Fulfillment,
    pub result_bytes_hex: String,
}

/// A key generation request with optional immutable chain identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateKeyRequest {
    pub label: String,
    pub network: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_magic: Option<u32>,
}

impl GenerateKeyRequest {
    pub fn neo_n3(
        label: impl Into<String>,
        network: impl Into<String>,
        network_magic: Option<u32>,
    ) -> Self {
        Self {
            label: label.into(),
            network: network.into(),
            chain_family: None,
            chain_id: None,
            network_magic,
        }
    }
}

/// Successful Neo N3 or NeoX transaction/consensus signature response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub key_id: String,
    pub script_hash: String,
    pub address: String,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocation_script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_transaction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    /// Additive service fields survive a NeoNexus relay.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

/// Successful raw signature response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawSignature {
    pub key_id: String,
    pub script_hash: String,
    pub address: String,
    pub digest: String,
    pub signature: String,
    pub public_key: String,
    pub invocation_script: String,
    pub verification_script: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_family: Option<String>,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

/// Contract-bound EIP-191 verifier signature returned by custody.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eip191FulfillmentSignature {
    pub key_id: String,
    pub address: String,
    pub public_key: String,
    pub digest: String,
    pub message_hash: String,
    pub signature: String,
    pub chain_family: String,
    pub chain_id: u64,
    pub oracle_contract: String,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}
