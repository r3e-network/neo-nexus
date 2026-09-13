//! Signer backend profiles and transport validation.

use anyhow::{bail, Result};

use crate::signer_client::SignerClient;

use super::super::{
    LocalSignerConfig, LocalWalletSigner, SignerBackendKind, SignerBackendProfile,
    SignerCapabilities,
};

/// A service-backed profile keeps its admin and signing identities separate.
/// The first is used only by the operator console; the second is optional and
/// must be a least-privilege `sign` caller for internal dispatch.
#[derive(Clone)]
pub struct ServiceSignerBackend {
    admin: SignerClient,
    signing: Option<SignerClient>,
}

impl ServiceSignerBackend {
    pub fn new(admin: SignerClient, signing: Option<SignerClient>) -> Self {
        Self { admin, signing }
    }

    pub fn admin_client(&self) -> &SignerClient {
        &self.admin
    }

    pub fn signing_client(&self) -> Option<&SignerClient> {
        self.signing.as_ref()
    }
}

impl std::fmt::Debug for ServiceSignerBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServiceSignerBackend")
            .field("admin", &self.admin)
            .field("signing_identity_configured", &self.signing.is_some())
            .finish()
    }
}

/// One configured backend and its stable, non-secret identity.
#[derive(Clone)]
pub enum ConfiguredSignerBackend {
    LocalWallet {
        profile: SignerBackendProfile,
        signer: Box<LocalWalletSigner>,
    },
    LocalSigner {
        profile: SignerBackendProfile,
        config: LocalSignerConfig,
    },
    NeoOsService {
        profile: SignerBackendProfile,
        service: ServiceSignerBackend,
        /// Optional node-facing SecureSign-compatible bridge. The HTTP
        /// service remains the custody/control plane; native neo-cli consumes
        /// this separate gRPC endpoint.
        node_signer: Option<LocalSignerConfig>,
    },
}

impl std::fmt::Debug for ConfiguredSignerBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ConfiguredSignerBackend")
            .field("profile", self.profile())
            .field("capabilities", &self.capabilities())
            .finish()
    }
}

impl ConfiguredSignerBackend {
    pub fn local_wallet(profile: SignerBackendProfile, signer: LocalWalletSigner) -> Result<Self> {
        require_kind(&profile, SignerBackendKind::LocalWallet)?;
        Ok(Self::LocalWallet {
            profile,
            signer: Box::new(signer),
        })
    }

    pub fn local_signer(profile: SignerBackendProfile, config: LocalSignerConfig) -> Result<Self> {
        require_kind(&profile, SignerBackendKind::LocalSigner)?;
        Ok(Self::LocalSigner { profile, config })
    }

    pub fn neo_os_service(
        profile: SignerBackendProfile,
        service: ServiceSignerBackend,
    ) -> Result<Self> {
        Self::neo_os_service_with_node_signer(profile, service, None)
    }

    pub fn neo_os_service_with_node_signer(
        profile: SignerBackendProfile,
        service: ServiceSignerBackend,
        node_signer: Option<LocalSignerConfig>,
    ) -> Result<Self> {
        require_kind(&profile, SignerBackendKind::NeoOsService)?;
        validate_service_transports(&profile, &service, Some(false), true)?;
        Ok(Self::NeoOsService {
            profile,
            service,
            node_signer,
        })
    }

    /// Compatibility seam for an in-process NeoOS HTTP test stub. Production
    /// registry documents and environment resolution always require the remote
    /// authenticated HTTPS topology.
    pub(crate) fn neo_os_service_compatible(
        profile: SignerBackendProfile,
        service: ServiceSignerBackend,
    ) -> Result<Self> {
        require_kind(&profile, SignerBackendKind::NeoOsService)?;
        validate_service_transports(&profile, &service, None, false)?;
        Ok(Self::NeoOsService {
            profile,
            service,
            node_signer: None,
        })
    }

    pub fn profile(&self) -> &SignerBackendProfile {
        match self {
            Self::LocalWallet { profile, .. }
            | Self::LocalSigner { profile, .. }
            | Self::NeoOsService { profile, .. } => profile,
        }
    }

    pub fn capabilities(&self) -> SignerCapabilities {
        match self {
            Self::LocalWallet { signer, .. } => signer.capabilities(),
            Self::LocalSigner { .. } => SignerCapabilities::local_signer(),
            Self::NeoOsService { .. } => SignerCapabilities::service(),
        }
    }

    pub fn local_wallet_signer(&self) -> Option<&LocalWalletSigner> {
        match self {
            Self::LocalWallet { signer, .. } => Some(signer.as_ref()),
            Self::LocalSigner { .. } | Self::NeoOsService { .. } => None,
        }
    }

    pub fn service(&self) -> Option<&ServiceSignerBackend> {
        match self {
            Self::NeoOsService { service, .. } => Some(service),
            Self::LocalWallet { .. } | Self::LocalSigner { .. } => None,
        }
    }

    pub fn local_signer_config(&self) -> Option<&LocalSignerConfig> {
        match self {
            Self::LocalSigner { config, .. } => Some(config),
            Self::LocalWallet { .. } | Self::NeoOsService { .. } => None,
        }
    }

    /// The one key this backend owns, when that is knowable here.
    ///
    /// A process-local wallet holds exactly one key, and a local signer is
    /// configured with exactly one public key — in both cases the workbench
    /// already knows the only value a node could legally be bound to, so asking
    /// an operator to type it is asking them to guess at something we could
    /// have filled in. A custody service holds many keys and is authoritative
    /// about them, so it answers `None` and the operator names the key.
    pub fn sole_key_id(&self) -> Option<String> {
        match self {
            Self::LocalWallet { signer, .. } => Some(signer.key_info().key_id),
            Self::LocalSigner { config, .. } => Some(config.public_key().to_string()),
            Self::NeoOsService { .. } => None,
        }
    }

    /// The gRPC bridge a native node may use for this NeoOS custody profile.
    /// Absence is explicit: application HTTP signing may still work, but a
    /// consensus node launch must fail instead of inventing a transport.
    pub fn neo_os_node_signer_config(&self) -> Option<&LocalSignerConfig> {
        match self {
            Self::NeoOsService { node_signer, .. } => node_signer.as_ref(),
            Self::LocalWallet { .. } | Self::LocalSigner { .. } => None,
        }
    }
}

pub(super) fn require_kind(
    profile: &SignerBackendProfile,
    expected: SignerBackendKind,
) -> Result<()> {
    if profile.kind != expected {
        bail!(
            "signer profile {} declares {}, but its backend is {}",
            profile.id,
            profile.kind,
            expected
        );
    }
    Ok(())
}

pub(super) fn validate_service_transports(
    profile: &SignerBackendProfile,
    service: &ServiceSignerBackend,
    expected_loopback: Option<bool>,
    require_https: bool,
) -> Result<()> {
    let admin = service.admin_client().config();
    for (role, client) in [
        ("admin", Some(service.admin_client())),
        ("signing", service.signing_client()),
    ] {
        let Some(client) = client else {
            continue;
        };
        let config = client.config();
        if require_https && config.uses_cleartext() {
            bail!(
                "{} profile {} requires HTTPS for its {role} transport",
                profile.kind,
                profile.id
            );
        }
        if expected_loopback.is_some_and(|expected| config.is_loopback() != expected) {
            let location = if expected_loopback == Some(true) {
                "a loopback"
            } else {
                "a non-loopback"
            };
            bail!(
                "{} profile {} requires {location} {role} origin",
                profile.kind,
                profile.id
            );
        }
        if config.base_url() != admin.base_url() {
            bail!(
                "signer profile {} must use one endpoint for its admin and signing identities",
                profile.id
            );
        }
    }
    Ok(())
}
