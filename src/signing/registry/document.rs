//! Signer registry file format parsing and deserialization.

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::signer_client::{SignerClient, SignerConfig, SignerProfileInput};

use super::{
    super::{
        LocalSignerConfig, LocalWalletConfig, LocalWalletSigner, SignerBackendKind,
        SignerBackendProfile,
    },
    backend::{ConfiguredSignerBackend, ServiceSignerBackend},
};

pub const REGISTRY_VERSION: u32 = 1;
pub const MAX_REGISTRY_BYTES: u64 = 256 * 1024;

pub fn normalized_id(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryDocument {
    pub version: u32,
    #[serde(default)]
    pub console_backend: Option<String>,
    #[serde(default)]
    pub relay_backend: Option<String>,
    pub backends: Vec<BackendDocument>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendDocument {
    pub id: String,
    pub label: String,
    pub kind: String,

    #[serde(default)]
    pub wallet_path: Option<String>,
    #[serde(default)]
    pub password_file: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub network_magic: Option<u32>,
    #[serde(default)]
    pub allow_transaction: Option<bool>,
    #[serde(default)]
    pub allow_consensus: Option<bool>,
    #[serde(default)]
    pub allow_raw: Option<bool>,

    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub public_key: Option<String>,

    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    #[serde(default)]
    pub admin_token_file: Option<String>,
    #[serde(default)]
    pub admin_caller_id: Option<String>,
    #[serde(default)]
    pub admin_workload_key_file: Option<String>,
    #[serde(default)]
    pub admin_workload_subject: Option<String>,
    #[serde(default)]
    pub admin_origin: Option<String>,
    #[serde(default)]
    pub signing_token_file: Option<String>,
    #[serde(default)]
    pub signing_caller_id: Option<String>,
    #[serde(default)]
    pub signing_workload_key_file: Option<String>,
    #[serde(default)]
    pub signing_workload_subject: Option<String>,
}

impl BackendDocument {
    pub fn open(self, base: &Path) -> Result<ConfiguredSignerBackend> {
        let profile_id = self.id.clone();
        let kind = self.kind.parse::<SignerBackendKind>()?;
        let profile = SignerBackendProfile::new(&self.id, &self.label, kind)?;
        match kind {
            SignerBackendKind::LocalWallet => self.open_wallet(profile, base),
            SignerBackendKind::LocalSigner => self.open_local_signer(profile),
            SignerBackendKind::NeoOsService => self.open_service(profile, base),
        }
        .with_context(|| format!("failed to open signer profile {profile_id}"))
    }

    fn open_wallet(
        self,
        profile: SignerBackendProfile,
        base: &Path,
    ) -> Result<ConfiguredSignerBackend> {
        if self.has_service_fields() || self.has_local_signer_fields() {
            bail!("local-wallet profile contains fields for another signer protocol");
        }
        let config = LocalWalletConfig {
            wallet_path: profile_path(base, &required(self.wallet_path, "wallet_path")?),
            password_file: profile_path(base, &required(self.password_file, "password_file")?),
            account: normalized_id(self.account),
            network: required(self.network, "network")?.trim().to_string(),
            network_magic: self
                .network_magic
                .context("local-wallet profile requires network_magic")?,
            allow_transaction: self.allow_transaction.unwrap_or(true),
            allow_consensus: self.allow_consensus.unwrap_or(false),
            allow_raw: self.allow_raw.unwrap_or(false),
        };
        if config.network_magic == 0 {
            bail!("local-wallet profile network_magic must be greater than zero");
        }
        if config.network.is_empty()
            || config.network.len() > 64
            || config.network.chars().any(char::is_control)
        {
            bail!("local-wallet profile network must be 1..=64 printable characters");
        }
        ConfiguredSignerBackend::local_wallet(profile, LocalWalletSigner::open(config)?)
    }

    fn open_local_signer(self, profile: SignerBackendProfile) -> Result<ConfiguredSignerBackend> {
        if self.has_wallet_fields() || self.has_service_fields() {
            bail!("local-signer profile contains local-wallet or NeoOS HTTP fields");
        }
        let endpoint = required(self.endpoint, "endpoint")?;
        let public_key = required(self.public_key, "public_key")?;
        let network_magic = self
            .network_magic
            .context("local-signer profile requires network_magic")?;
        ConfiguredSignerBackend::local_signer(
            profile,
            LocalSignerConfig::new(endpoint, public_key, network_magic)?,
        )
    }

    fn open_service(
        mut self,
        profile: SignerBackendProfile,
        base: &Path,
    ) -> Result<ConfiguredSignerBackend> {
        if self.has_wallet_fields() {
            bail!("neo-os-service profile contains local-wallet fields");
        }
        let node_signer = match (
            self.endpoint.take(),
            self.public_key.take(),
            self.network_magic.take(),
        ) {
            (None, None, None) => None,
            (Some(endpoint), Some(public_key), Some(network_magic)) => {
                Some(LocalSignerConfig::new(endpoint, public_key, network_magic)?)
            }
            _ => bail!(
                "neo-os-service node bridge requires endpoint, public_key, and network_magic together"
            ),
        };
        let url = required(self.url.clone(), "url")?.trim().to_string();
        let timeout = Duration::from_secs(self.timeout_seconds.unwrap_or(10));
        if timeout.is_zero() {
            bail!("service profile timeout_seconds must be greater than zero");
        }
        let has_signing_identity = self.signing_token_file.is_some()
            || self.signing_caller_id.is_some()
            || self.signing_workload_key_file.is_some()
            || self.signing_workload_subject.is_some();
        ensure_distinct_profile_files(
            base,
            self.admin_token_file
                .as_deref()
                .or(self.admin_workload_key_file.as_deref()),
            self.signing_token_file
                .as_deref()
                .or(self.signing_workload_key_file.as_deref()),
        )?;
        let admin_caller_id = normalized_id(self.admin_caller_id.clone());
        let signing_caller_id = normalized_id(self.signing_caller_id.clone());
        if admin_caller_id.is_some() && admin_caller_id == signing_caller_id {
            bail!("admin_caller_id and signing_caller_id must name different callers");
        }
        let admin = SignerClient::new(SignerConfig::from_profile(SignerProfileInput {
            service_url: url.clone(),
            token_file: self
                .admin_token_file
                .as_deref()
                .map(|path| profile_path(base, path)),
            caller_id: admin_caller_id,
            workload_key_file: self
                .admin_workload_key_file
                .as_deref()
                .map(|path| profile_path(base, path)),
            workload_subject: normalized_id(self.admin_workload_subject.clone()),
            consumer_origin: normalized_id(self.admin_origin.clone()),
            timeout,
        })?);

        let signing = has_signing_identity
            .then(|| {
                SignerConfig::from_profile(SignerProfileInput {
                    service_url: url,
                    token_file: self
                        .signing_token_file
                        .as_deref()
                        .map(|path| profile_path(base, path)),
                    caller_id: signing_caller_id,
                    workload_key_file: self
                        .signing_workload_key_file
                        .as_deref()
                        .map(|path| profile_path(base, path)),
                    workload_subject: normalized_id(self.signing_workload_subject),
                    consumer_origin: None,
                    timeout,
                })
                .map(SignerClient::new)
            })
            .transpose()?;
        let service = ServiceSignerBackend::new(admin, signing);
        ConfiguredSignerBackend::neo_os_service_with_node_signer(profile, service, node_signer)
    }

    fn has_wallet_fields(&self) -> bool {
        self.wallet_path.is_some()
            || self.password_file.is_some()
            || self.account.is_some()
            || self.network.is_some()
            || self.allow_transaction.is_some()
            || self.allow_consensus.is_some()
            || self.allow_raw.is_some()
    }

    fn has_local_signer_fields(&self) -> bool {
        self.endpoint.is_some() || self.public_key.is_some()
    }

    fn has_service_fields(&self) -> bool {
        self.url.is_some()
            || self.timeout_seconds.is_some()
            || self.admin_token_file.is_some()
            || self.admin_caller_id.is_some()
            || self.admin_workload_key_file.is_some()
            || self.admin_workload_subject.is_some()
            || self.admin_origin.is_some()
            || self.signing_token_file.is_some()
            || self.signing_caller_id.is_some()
            || self.signing_workload_key_file.is_some()
            || self.signing_workload_subject.is_some()
    }
}

pub fn read_registry_document(path: &Path) -> Result<RegistryDocument> {
    let file = File::open(path)
        .with_context(|| format!("failed to open signer registry {}", path.display()))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("failed to inspect signer registry {}", path.display()))?;
    if !metadata.is_file() {
        bail!("signer registry {} is not a regular file", path.display());
    }
    if metadata.len() > MAX_REGISTRY_BYTES {
        bail!(
            "signer registry {} exceeds the {MAX_REGISTRY_BYTES}-byte limit",
            path.display()
        );
    }
    let mut text = String::new();
    file.take(MAX_REGISTRY_BYTES + 1)
        .read_to_string(&mut text)
        .with_context(|| format!("failed to read signer registry {}", path.display()))?;
    if text.len() as u64 > MAX_REGISTRY_BYTES {
        bail!(
            "signer registry {} exceeds the {MAX_REGISTRY_BYTES}-byte limit",
            path.display()
        );
    }
    toml::from_str(&text)
        .with_context(|| format!("signer registry {} is not valid TOML", path.display()))
}

pub fn profile_path(base: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw.trim());
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

pub fn ensure_distinct_profile_files(
    base: &Path,
    admin: Option<&str>,
    signing: Option<&str>,
) -> Result<()> {
    let (Some(admin), Some(signing)) = (admin, signing) else {
        return Ok(());
    };
    let admin = std::fs::canonicalize(profile_path(base, admin))
        .context("failed to resolve the admin credential file")?;
    let signing = std::fs::canonicalize(profile_path(base, signing))
        .context("failed to resolve the signing credential file")?;
    #[cfg(windows)]
    let same = admin
        .to_string_lossy()
        .eq_ignore_ascii_case(&signing.to_string_lossy());
    #[cfg(not(windows))]
    let same = admin == signing;
    const MAX_CREDENTIAL_BYTES: u64 = 4 * 1024;
    let admin_secret =
        crate::secret_file::read_secret(&admin, MAX_CREDENTIAL_BYTES, "signer admin identity")?;
    let signing_secret =
        crate::secret_file::read_secret(&signing, MAX_CREDENTIAL_BYTES, "signer signing identity")?;
    if same || credential_line(&admin_secret) == credential_line(&signing_secret) {
        bail!("admin and signing identities must use different credential files and values");
    }
    Ok(())
}

fn credential_line(bytes: &[u8]) -> &[u8] {
    bytes
        .strip_suffix(b"\r\n")
        .or_else(|| bytes.strip_suffix(b"\n"))
        .unwrap_or(bytes)
}

pub fn required(value: Option<String>, field: &str) -> Result<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .with_context(|| format!("signer profile requires {field}"))
}
