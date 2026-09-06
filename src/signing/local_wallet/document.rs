use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::wallet::crypto::{extract_single_sig_contract_public_key, valid_scrypt_parameters};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WalletDocument {
    pub accounts: Vec<WalletAccountDocument>,
    pub scrypt: WalletScrypt,
    #[serde(default)]
    #[serde(rename = "name")]
    pub _name: Option<String>,
    #[serde(default)]
    #[serde(rename = "version")]
    pub _version: Option<String>,
    #[serde(default)]
    #[serde(rename = "extra")]
    pub _extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WalletScrypt {
    pub n: u64,
    pub r: u32,
    pub p: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WalletAccountDocument {
    pub address: String,
    pub key: String,
    #[serde(default)]
    #[serde(rename = "label")]
    pub _label: Option<String>,
    #[serde(rename = "isDefault", default)]
    pub is_default: bool,
    #[serde(default)]
    pub lock: bool,
    pub contract: Option<WalletContract>,
    #[serde(default)]
    #[serde(rename = "extra")]
    pub _extra: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WalletContract {
    pub script: String,
    #[serde(default)]
    #[serde(rename = "parameters")]
    pub _parameters: Vec<serde_json::Value>,
    #[serde(default)]
    #[serde(rename = "deployed")]
    pub _deployed: bool,
}

pub(super) struct WalletAccount {
    pub address: String,
    pub encrypted_key: String,
    pub public_key: String,
    pub verification_script: String,
    pub scrypt: WalletScrypt,
}

pub(super) fn selected_account(
    document: &WalletDocument,
    selector: Option<&str>,
) -> Result<WalletAccount> {
    validate_scrypt(&document.scrypt)?;
    let matches = document
        .accounts
        .iter()
        .filter(|account| match selector {
            Some(selector) => account.address.trim() == selector.trim(),
            None => account.is_default,
        })
        .collect::<Vec<_>>();
    let account = match (selector, matches.as_slice()) {
        (_, [account]) => *account,
        (Some(_), []) => bail!("the selected account is not present in the local wallet"),
        (Some(_), _) => bail!("the local wallet contains duplicate selected account addresses"),
        (None, []) if document.accounts.len() == 1 => &document.accounts[0],
        (None, []) => {
            bail!("the local wallet has no unique default account; configure one explicitly")
        }
        (None, _) => bail!("the local wallet has more than one default account"),
    };
    if account.lock {
        bail!("the selected local wallet account is locked");
    }
    let encrypted_key = account.key.trim();
    if encrypted_key.is_empty() {
        bail!("the selected local wallet account is watch-only");
    }
    let contract = account
        .contract
        .as_ref()
        .context("the selected local wallet account has no contract")?;
    let public_key = extract_single_sig_contract_public_key(&contract.script)
        .context("the selected local wallet account is not an N3 single-signature contract")?;
    Ok(WalletAccount {
        address: account.address.trim().to_string(),
        encrypted_key: encrypted_key.to_string(),
        public_key,
        verification_script: contract.script.trim().to_ascii_lowercase(),
        scrypt: document.scrypt,
    })
}

fn validate_scrypt(scrypt: &WalletScrypt) -> Result<()> {
    if !valid_scrypt_parameters(scrypt.n, u64::from(scrypt.r), u64::from(scrypt.p)) {
        bail!("local signing requires bounded power-of-two NEP-6 scrypt parameters");
    }
    Ok(())
}
