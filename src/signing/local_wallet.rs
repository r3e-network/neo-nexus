//! Local encrypted NEP-6 wallet backend.
//!
//! The backend is intentionally process-local and is never exposed through the
//! caller-facing HTTP relay.  It reloads and verifies the pinned wallet file and
//! verifies the pinned wallet again for each operation. The passphrase is read
//! only during startup; one zeroizing private-key allocation is then shared by
//! signer clones so signing does not repeat the memory-hard NEP-2 KDF.

mod config;
mod crypto;
mod document;
mod files;
mod transaction;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Context, Result};
use serde_json::Value;
use sha2::Digest as _;
use zeroize::Zeroizing;

use crate::{
    config::ServiceWallet,
    signer_client::{KeyPublic, RawSignRequest, RawSignature, SignRequest, Signature},
    signing::SignerCapabilities,
};

pub use config::{
    LocalWalletConfig, LocalWalletEnvironment, LOCAL_WALLET_ACCOUNT_ENV,
    LOCAL_WALLET_ALLOW_CONSENSUS_ENV, LOCAL_WALLET_ALLOW_RAW_ENV,
    LOCAL_WALLET_ALLOW_TRANSACTION_ENV, LOCAL_WALLET_NETWORK_ENV, LOCAL_WALLET_NETWORK_MAGIC_ENV,
    LOCAL_WALLET_PASSWORD_FILE_ENV, LOCAL_WALLET_PATH_ENV,
};

use crypto::{decrypt_nep2, identity_from_private, sign_message, sign_transaction};
use document::{selected_account, WalletAccount};
use files::{read_password, read_wallet, read_wallet_pinned};

/// A pinned local wallet account. Its custom `Debug` implementation omits both
/// the passphrase path and the process-resident zeroizing private key.
#[derive(Clone)]
pub struct LocalWalletSigner {
    config: LocalWalletConfig,
    wallet_sha256: String,
    identity: KeyPublic,
    secret: Arc<LocalWalletSecret>,
    request_ledger: Arc<Mutex<RequestLedger>>,
}

/// One process-local key allocation shared by cheap signer clones. The bytes
/// are zeroized when the last clone is dropped and are never exposed through
/// Debug, serialization, or the public API.
struct LocalWalletSecret {
    private_key: Zeroizing<[u8; 32]>,
}

#[derive(Default)]
struct RequestLedger {
    claims: BTreeMap<(RequestOperation, String), [u8; 32]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RequestOperation {
    Transaction,
    Raw,
}

const MAX_REQUEST_CLAIMS: usize = 4_096;

impl std::fmt::Debug for LocalWalletSigner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LocalWalletSigner")
            .field("wallet_path", &self.config.wallet_path)
            .field("password_file", &"<protected-file>")
            .field("account", &self.identity.address)
            .field("network", &self.config.network)
            .field("network_magic", &self.config.network_magic)
            .field("capabilities", &self.capabilities())
            .finish()
    }
}

impl LocalWalletSigner {
    /// Validate, pin, and unlock the selected account once at startup.
    pub fn open(config: LocalWalletConfig) -> Result<Self> {
        config.validate()?;
        let wallet = read_wallet(&config.wallet_path)?;
        let account = selected_account(&wallet.document, config.account.as_deref())?;
        let password = read_password(&config.password_file)?;
        let private_key = decrypt_nep2(&account.encrypted_key, password.as_str(), account.scrypt)
            .context("failed to unlock the selected local wallet account")?;
        let mut derived = identity_from_private(
            &private_key,
            &config.network,
            config.network_magic,
            &wallet.sha256,
        )?;
        derived.signing_enabled =
            config.allow_transaction || config.allow_consensus || config.allow_raw;
        validate_account_identity(&account, &derived)?;
        Ok(Self {
            config,
            wallet_sha256: wallet.sha256,
            identity: derived,
            secret: Arc::new(LocalWalletSecret { private_key }),
            request_ledger: Arc::new(Mutex::new(RequestLedger::default())),
        })
    }

    pub fn key_info(&self) -> KeyPublic {
        self.identity.clone()
    }

    pub fn capabilities(&self) -> SignerCapabilities {
        SignerCapabilities::local_wallet(
            self.config.allow_transaction,
            self.config.allow_consensus,
            self.config.allow_raw,
        )
    }

    pub fn wallet_sha256(&self) -> &str {
        &self.wallet_sha256
    }

    /// Material needed by a native Neo node to unlock this exact NEP-6
    /// wallet. The wallet and selected account are hash/identity checked again
    /// before the password is read. The returned password is intentionally
    /// short lived and its [`Debug`] representation is redacted by
    /// [`ServiceWallet`].
    pub fn native_service_wallet(&self) -> Result<ServiceWallet> {
        self.checked_private_key()?;
        let password = read_password(&self.config.password_file)?;
        Ok(
            ServiceWallet::at(self.config.wallet_path.to_string_lossy().into_owned())
                .unlocked_with(password.as_str()),
        )
    }

    pub fn sign_transaction(&self, request: &SignRequest) -> Result<Signature> {
        if !self.config.allow_transaction {
            bail!("local wallet transaction signing is disabled by configuration");
        }
        self.sign_framed(request)
    }

    pub fn sign_consensus(&self, request: &SignRequest) -> Result<Signature> {
        let _ = request;
        bail!(
            "local-wallet consensus signing is unavailable; use a service backend with durable anti-equivocation"
        )
    }

    pub fn sign_raw(&self, request: &RawSignRequest) -> Result<RawSignature> {
        if !self.config.allow_raw {
            bail!("local wallet raw signing is disabled by configuration");
        }
        self.check_key(&request.key_id)?;
        let bytes = crypto::decode_request_hex(&request.data_hex)?;
        let private_key = self.checked_private_key()?;
        self.claim_request(RequestOperation::Raw, request.request_id.as_deref(), &bytes)?;
        let signature = sign_message(private_key, &bytes)?;
        let invocation = crypto::invocation_script(&signature);
        let digest = crypto::sha256_display(&bytes, false, true);
        Ok(RawSignature {
            key_id: self.identity.key_id.clone(),
            script_hash: self.identity.script_hash.clone(),
            address: self.identity.address.clone(),
            digest,
            signature: crypto::hex_encode(&signature),
            public_key: self.identity.public_key.clone(),
            invocation_script: crypto::hex_encode(&invocation),
            verification_script: self.identity.verification_script.clone(),
            chain_family: Some("neo-n3".to_string()),
            additional_fields: local_metadata(&self.wallet_sha256),
        })
    }

    fn sign_framed(&self, request: &SignRequest) -> Result<Signature> {
        self.check_key(&request.key_id)?;
        if request
            .chain_family
            .as_deref()
            .is_some_and(|family| family != "neo-n3")
        {
            bail!("a local NEP-6 wallet supports only the neo-n3 chain family");
        }
        if request.chain_id.is_some() {
            bail!("Neo N3 local wallet requests use network_magic, not an EVM chain_id");
        }
        let unsigned = crypto::decode_request_hex(&request.unsigned_hex)?;
        transaction::validate_for_signer(&unsigned, &self.identity.script_hash)?;
        let private_key = self.checked_private_key()?;
        self.claim_request(
            RequestOperation::Transaction,
            request.request_id.as_deref(),
            &unsigned,
        )?;
        let signature = sign_transaction(private_key, &unsigned, self.config.network_magic)?;
        let invocation = crypto::invocation_script(&signature);
        Ok(Signature {
            key_id: self.identity.key_id.clone(),
            script_hash: self.identity.script_hash.clone(),
            address: self.identity.address.clone(),
            digest: crypto::transaction_hash_display(&unsigned),
            invocation_script: Some(crypto::hex_encode(&invocation)),
            verification_script: Some(self.identity.verification_script.clone()),
            chain_family: Some("neo-n3".to_string()),
            signed_transaction: None,
            signature: None,
            signature_hex: None,
            public_key: Some(self.identity.public_key.clone()),
            chain_id: None,
            additional_fields: local_metadata(&self.wallet_sha256),
        })
    }

    fn check_key(&self, key_id: &str) -> Result<()> {
        if key_id.trim() != self.identity.key_id {
            bail!(
                "local wallet key {} was requested from backend owning {}",
                key_id.trim(),
                self.identity.key_id
            );
        }
        Ok(())
    }

    fn checked_private_key(&self) -> Result<&[u8; 32]> {
        let wallet = read_wallet_pinned(&self.config.wallet_path, &self.wallet_sha256)?;
        let account = selected_account(&wallet.document, self.config.account.as_deref())?;
        let private_key = &*self.secret.private_key;
        let derived = identity_from_private(
            private_key,
            &self.config.network,
            self.config.network_magic,
            &wallet.sha256,
        )?;
        validate_account_identity(&account, &derived)?;
        if derived.key_id != self.identity.key_id || derived.public_key != self.identity.public_key
        {
            bail!("local wallet identity changed after startup");
        }
        Ok(private_key)
    }

    fn claim_request(
        &self,
        operation: RequestOperation,
        request_id: Option<&str>,
        bytes: &[u8],
    ) -> Result<()> {
        let Some(request_id) = request_id else {
            return Ok(());
        };
        let request_id = request_id.trim();
        if request_id.is_empty()
            || request_id.len() > 128
            || !request_id.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
            })
        {
            bail!(
                "local-wallet request_id must be 1..=128 ASCII alphanumeric, `-`, `_`, `.`, or `:` characters"
            );
        }
        let mut preimage = Vec::with_capacity(bytes.len() + 16);
        preimage.extend_from_slice(match operation {
            RequestOperation::Transaction => &b"transaction\0"[..],
            RequestOperation::Raw => &b"raw\0"[..],
        });
        preimage.extend_from_slice(bytes);
        let digest: [u8; 32] = sha2::Sha256::digest(preimage).into();
        let mut ledger = self
            .request_ledger
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let key = (operation, request_id.to_string());
        match ledger.claims.get(&key) {
            Some(previous) if previous == &digest => Ok(()),
            Some(_) => bail!(
                "local-wallet request_id {request_id:?} was already used for a different payload"
            ),
            None if ledger.claims.len() >= MAX_REQUEST_CLAIMS => {
                bail!("local-wallet request-id ledger is full; restart with a fresh trusted workload window")
            }
            None => {
                ledger.claims.insert(key, digest);
                Ok(())
            }
        }
    }
}

fn validate_account_identity(account: &WalletAccount, derived: &KeyPublic) -> Result<()> {
    if account.address != derived.address {
        bail!("the decrypted NEP-2 key does not match the selected wallet address");
    }
    if account.public_key != derived.public_key {
        bail!("the decrypted NEP-2 key does not match the selected wallet contract public key");
    }
    if account.verification_script != derived.verification_script {
        bail!("the decrypted NEP-2 key does not match the selected wallet verification script");
    }
    Ok(())
}

fn local_metadata(wallet_sha256: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "backend".to_string(),
            Value::String("local-wallet".to_string()),
        ),
        (
            "wallet_sha256".to_string(),
            Value::String(wallet_sha256.to_string()),
        ),
    ])
}

#[cfg(test)]
#[path = "../../tests/unit/signing/local_wallet/tests.rs"]
mod tests;
