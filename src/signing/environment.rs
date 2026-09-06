//! One process-level signer-registry loader shared by Web and CLI.

use std::{env, str::FromStr};

use anyhow::{bail, Context, Result};

use crate::signer_client::{SignerClient, SignerConfig};

use super::{
    ConfiguredSignerBackend, LocalSignerConfig, LocalWalletConfig, LocalWalletSigner,
    ServiceSignerBackend, SignerBackendKind, SignerBackendProfile, SignerRegistry, BACKEND_ENV,
    PROFILES_FILE_ENV,
};

impl SignerRegistry {
    /// Load the named registry document, or the legacy single-backend
    /// environment, into the same registry type. Mixing the two is rejected.
    pub fn from_process_environment() -> Result<Self> {
        if let Some(registry) = Self::from_env_file()? {
            let conflicts = env::vars_os()
                .filter_map(|(name, _)| name.into_string().ok())
                .filter(|name| {
                    !name.eq_ignore_ascii_case(PROFILES_FILE_ENV)
                        && name.to_ascii_uppercase().starts_with("NEONEXUS_SIGNER_")
                })
                .collect::<Vec<_>>();
            if !conflicts.is_empty() {
                bail!(
                    "{PROFILES_FILE_ENV} cannot be combined with legacy signer settings: {}",
                    conflicts.join(", ")
                );
            }
            return Ok(registry);
        }
        let selected = selected_backend_from_env()?;
        let local_wallet = LocalWalletConfig::from_env()?;
        let local_signer = LocalSignerConfig::from_env()?;
        let service = SignerConfig::from_env()?;
        Self::from_legacy_components(selected, local_wallet, local_signer, service)
    }

    pub(crate) fn from_legacy_components(
        selected: Option<SignerBackendKind>,
        local_wallet: Option<LocalWalletConfig>,
        local_signer: Option<LocalSignerConfig>,
        service: Option<SignerConfig>,
    ) -> Result<Self> {
        match selected {
            None => match (local_wallet, local_signer, service) {
                (None, None, None) => Ok(Self::empty()),
                (None, None, Some(config)) => {
                    if config.uses_cleartext() {
                        bail!(
                            "legacy untyped signer configuration requires HTTPS; use an authenticated HTTPS endpoint or a signer profile registry"
                        );
                    }
                    single_neo_os_service(config)
                }
                (Some(_), _, _) => bail!("set {BACKEND_ENV}=local-wallet before local wallet settings can be used"),
                (_, Some(_), _) => bail!("set {BACKEND_ENV}=local-signer before native local signer settings can be used"),
            },
            Some(SignerBackendKind::LocalWallet) => match (local_wallet, local_signer, service) {
                (Some(config), None, None) => single_local_wallet(LocalWalletSigner::open(config)?),
                (None, _, _) => bail!(
                    "{BACKEND_ENV}=local-wallet requires a complete local wallet configuration"
                ),
                (Some(_), _, _) => bail!(
                    "local wallet and other signer settings cannot be active together; select one explicit signer backend"
                ),
            },
            Some(SignerBackendKind::LocalSigner) => match (local_wallet, local_signer, service) {
                (None, Some(config), None) => single_local_signer(config),
                (_, None, _) => bail!(
                    "{BACKEND_ENV}=local-signer requires endpoint, public key, and network magic"
                ),
                _ => bail!("NeoOS HTTP or local-wallet settings are not valid for {BACKEND_ENV}=local-signer"),
            },
            Some(SignerBackendKind::NeoOsService) => {
                if local_wallet.is_some() || local_signer.is_some() {
                    bail!("local signer settings are not valid for {BACKEND_ENV}=neo-os-service");
                }
                let config = service.context(
                    "neo-os-service requires the signer URL and exactly one admin identity",
                )?;
                if config.uses_cleartext() {
                    bail!("neo-os-service requires HTTPS");
                }
                single_neo_os_service(config)
            }
        }
    }
}

fn selected_backend_from_env() -> Result<Option<SignerBackendKind>> {
    match env::var(BACKEND_ENV) {
        Ok(value) if value.trim().is_empty() => bail!("{BACKEND_ENV} must not be blank when set"),
        Ok(value) => Ok(Some(SignerBackendKind::from_str(&value)?)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => bail!("{BACKEND_ENV} must contain valid Unicode"),
    }
}

fn single_local_wallet(signer: LocalWalletSigner) -> Result<SignerRegistry> {
    let id = SignerBackendKind::LocalWallet.slug().to_string();
    let profile = SignerBackendProfile::new(
        &id,
        SignerBackendKind::LocalWallet.label(),
        SignerBackendKind::LocalWallet,
    )?;
    let backend = ConfiguredSignerBackend::local_wallet(profile, signer)?;
    SignerRegistry::new([backend], Some(id), None)
}

fn single_local_signer(config: LocalSignerConfig) -> Result<SignerRegistry> {
    let kind = SignerBackendKind::LocalSigner;
    let id = kind.slug().to_string();
    let profile = SignerBackendProfile::new(&id, kind.label(), kind)?;
    let backend = ConfiguredSignerBackend::local_signer(profile, config)?;
    SignerRegistry::new([backend], None, None)
}

fn single_neo_os_service(config: SignerConfig) -> Result<SignerRegistry> {
    let kind = SignerBackendKind::NeoOsService;
    let id = kind.slug().to_string();
    let profile = SignerBackendProfile::new(&id, kind.label(), kind)?;
    let service = ServiceSignerBackend::new(SignerClient::new(config), None);
    let backend = ConfiguredSignerBackend::neo_os_service(profile, service)?;
    SignerRegistry::new([backend], Some(id.clone()), Some(id))
}
