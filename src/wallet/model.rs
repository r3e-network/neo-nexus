use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NeoWalletValidationReport {
    pub schema_version: u32,
    pub status: String,
    pub source_path: String,
    pub wallet_version: Option<String>,
    pub account_count: usize,
    pub encrypted_account_count: usize,
    pub default_account_count: usize,
    pub watch_only_account_count: usize,
    pub contract_public_keys: Vec<String>,
    pub passed_count: usize,
    pub warning_count: usize,
    pub failed_count: usize,
    pub checks: Vec<NeoWalletValidationCheck>,
}

impl NeoWalletValidationReport {
    pub fn is_success(&self) -> bool {
        self.failed_count == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeoWalletProfile {
    pub id: String,
    pub label: String,
    pub source_path: String,
    pub wallet_version: Option<String>,
    pub primary_address: String,
    pub contract_public_keys: Vec<String>,
    pub wallet_sha256: String,
    pub account_count: usize,
    pub encrypted_account_count: usize,
    pub default_account_count: usize,
    pub watch_only_account_count: usize,
    pub validated_at_unix: u64,
    pub last_used_at_unix: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NeoWalletValidationCheck {
    pub category: String,
    pub label: String,
    pub status: NeoWalletValidationStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NeoWalletValidationStatus {
    Pass,
    Warn,
    Fail,
}
