//! Node-bound signer dispatch.
//!
//! A node stores one complete [`SignerKeyRef`]. Dispatch always uses that
//! backend-qualified key and never consults the registry's default route. This
//! Native-node integration is resolved here as well, before config is written
//! or an old process is stopped. The three signer kinds remain distinct and a
//! failed choice never falls through to another profile.

use std::path::Path;

use anyhow::{Context, Result};

use crate::{
    catalog::{PluginId, PluginState},
    config::{network_magic, GenerationContext, ServiceWallet},
    events::NewRuntimeEvent,
    preflight::resolve_command_path,
    repository::Repository,
    roles::NodeRole,
    signer_client::{
        Eip191FulfillmentRequest, Eip191FulfillmentSignature, KeyPublic, Outcome, RawSignRequest,
        RawSignature, SignRequest, Signature,
    },
    signing::{ConfiguredSignerBackend, SignerBackendKind, SignerKeyRef, SignerRegistry},
    types::{NodeConfig, NodeType},
};

/// Stable name registered by the official neo-cli SignClient plugin and used
/// by DBFT startup.
pub const NODE_SIGN_CLIENT_NAME: &str = "SignClient";
pub const SIGNER_BOOTSTRAP_PLUGIN: &str = "NeoNexus.SignerBootstrap";
pub const REMOTE_SIGNER_NEO_CLI_VERSION: &str = "3.9.2";

/// The one signer implementation selected for a node launch. There is no
/// fallback variant and no ordered list.
#[derive(Clone, PartialEq, Eq)]
pub enum NodeSignerRuntime {
    LocalWallet {
        wallet: ServiceWallet,
        public_key: String,
        network_magic: u32,
    },
    SignClient {
        backend_kind: SignerBackendKind,
        endpoint: String,
        public_key: String,
        network_magic: u32,
    },
}

impl std::fmt::Debug for NodeSignerRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LocalWallet {
                wallet,
                public_key,
                network_magic,
            } => formatter
                .debug_struct("LocalWallet")
                .field("wallet", wallet)
                .field("public_key", public_key)
                .field("network_magic", network_magic)
                .finish(),
            Self::SignClient {
                backend_kind,
                endpoint,
                public_key,
                network_magic,
            } => formatter
                .debug_struct("SignClient")
                .field("backend_kind", backend_kind)
                .field("endpoint", endpoint)
                .field("public_key", public_key)
                .field("network_magic", network_magic)
                .finish(),
        }
    }
}

/// Complete signer decision made before launch has side effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSignerLaunch {
    role: Option<NodeRole>,
    runtime: Option<NodeSignerRuntime>,
}

impl NodeSignerLaunch {
    pub fn inert(role: Option<NodeRole>) -> Self {
        Self {
            role,
            runtime: None,
        }
    }

    pub fn role(&self) -> Option<NodeRole> {
        self.role
    }

    pub fn runtime(&self) -> Option<&NodeSignerRuntime> {
        self.runtime.as_ref()
    }

    pub fn generation_context(&self) -> GenerationContext {
        let context = self
            .role
            .map_or_else(GenerationContext::default, GenerationContext::for_role);
        match &self.runtime {
            Some(NodeSignerRuntime::LocalWallet { wallet, .. }) => {
                context.with_wallet(wallet.clone())
            }
            Some(NodeSignerRuntime::SignClient {
                endpoint,
                public_key,
                network_magic,
                ..
            }) => context.with_sign_client(
                NODE_SIGN_CLIENT_NAME,
                endpoint,
                public_key,
                *network_magic,
            ),
            None => context,
        }
    }
}

/// Load the node's exact signer route and enforce it for duties that sign.
pub fn node_signer_key(repository: &Repository, node: &NodeConfig) -> Result<Option<SignerKeyRef>> {
    let key = repository.load_node_signer_key(&node.id)?;
    let role = repository.load_node_role(&node.id)?;
    if let Some(role) = role.filter(|role| role.requires_signer()) {
        if key.is_none() {
            anyhow::bail!(
                "{} has signing duty {} but no signer backend and key are bound",
                node.name,
                role.label()
            );
        }
    }
    Ok(key)
}

/// Resolve and validate the exact node signer before launch. Only signing
/// duties activate a bound profile; read-only duties never unlock a wallet or
/// contact custody merely because a dormant binding exists.
pub fn prepare_node_signer_launch(
    repository: &Repository,
    registry: Option<&SignerRegistry>,
    node: &NodeConfig,
    plugins: &[PluginState],
    working_dir: &Path,
) -> Result<NodeSignerLaunch> {
    let role = repository.load_node_role(&node.id)?;
    let key = node_signer_key(repository, node)?;
    let Some(role) = role.filter(|role| role.requires_signer()) else {
        return Ok(NodeSignerLaunch::inert(role));
    };
    let key = key.context("the signing duty has no signer backend and key binding")?;

    // Slashing & Double-Signing Prevention: Ensure no other running node holds this same signer lease.
    if let Ok(nodes) = repository.list_nodes() {
        for other in nodes {
            if other.id != node.id && (other.status.is_running() || other.pid.is_some()) {
                if let Ok(Some(other_key)) = repository.load_node_signer_key(&other.id) {
                    if other_key == key {
                        anyhow::bail!(
                            "double-signing hazard: signer {}/{} is currently leased to active running node {} ({})",
                            key.backend_id,
                            key.key_id,
                            other.name,
                            other.id
                        );
                    }
                }
            }
        }
    }

    let registry = registry.context(
        "this signing node requires its configured signer registry; no fallback is available",
    )?;
    let backend = registry.backend(&key.backend_id).with_context(|| {
        format!(
            "node {} is bound to unavailable signer backend {}",
            node.name, key.backend_id
        )
    })?;

    ensure_role_plugin(node, role, plugins)?;
    let runtime = match backend {
        ConfiguredSignerBackend::LocalWallet { signer, .. } => {
            ensure_local_wallet_runtime(node, role)?;
            let identity = signer.key_info();
            ensure_bound_key(&key, &identity.key_id, "local wallet")?;
            ensure_neo_n3_identity(
                node,
                identity.chain_family.as_deref(),
                identity.network_magic,
                &identity.network,
            )?;
            let network_magic = identity
                .network_magic
                .context("the validated local wallet has no Neo N3 network magic")?;
            if node.node_type == NodeType::NeoCli {
                ensure_neo_cli_runtime_root(node, working_dir, &[role_plugin_name(role)?])?;
            }
            NodeSignerRuntime::LocalWallet {
                wallet: signer.native_service_wallet()?,
                public_key: identity.public_key,
                network_magic,
            }
        }
        ConfiguredSignerBackend::LocalSigner { config, .. } => {
            ensure_sign_client_runtime(node, role)?;
            ensure_bound_key(&key, config.public_key(), "local signer")?;
            ensure_network_magic(node, config.network_magic())?;
            ensure_neo_cli_runtime_root(
                node,
                working_dir,
                &["DBFTPlugin", "SignClient", SIGNER_BOOTSTRAP_PLUGIN],
            )?;
            NodeSignerRuntime::SignClient {
                backend_kind: SignerBackendKind::LocalSigner,
                endpoint: config.endpoint().to_string(),
                public_key: config.public_key().to_string(),
                network_magic: config.network_magic(),
            }
        }
        ConfiguredSignerBackend::NeoOsService { .. } => {
            ensure_sign_client_runtime(node, role)?;
            let bridge = backend.neo_os_node_signer_config().with_context(|| {
                format!(
                    "NeoOS signer profile {} has no node-facing gRPC bridge; configure endpoint, public_key, and network_magic",
                    backend.profile().id
                )
            })?;
            ensure_network_magic(node, bridge.network_magic())?;
            let identity = registry
                .key_info(&key)?
                .into_parts()
                .map_err(|refusal| anyhow::anyhow!(refusal.summary()))?;
            ensure_neo_n3_identity(
                node,
                identity.chain_family.as_deref(),
                identity.network_magic,
                &identity.network,
            )?;
            if !identity.signing_enabled {
                anyhow::bail!("NeoOS signer key {} is disabled", key.key_id);
            }
            if !identity
                .public_key
                .eq_ignore_ascii_case(bridge.public_key())
            {
                anyhow::bail!(
                    "NeoOS bridge public key {} does not match bound key {} public key {}",
                    bridge.public_key(),
                    key.key_id,
                    identity.public_key
                );
            }
            ensure_neo_cli_runtime_root(
                node,
                working_dir,
                &["DBFTPlugin", "SignClient", SIGNER_BOOTSTRAP_PLUGIN],
            )?;
            NodeSignerRuntime::SignClient {
                backend_kind: SignerBackendKind::NeoOsService,
                endpoint: bridge.endpoint().to_string(),
                public_key: bridge.public_key().to_string(),
                network_magic: bridge.network_magic(),
            }
        }
    };
    Ok(NodeSignerLaunch {
        role: Some(role),
        runtime: Some(runtime),
    })
}

fn ensure_local_wallet_runtime(node: &NodeConfig, role: NodeRole) -> Result<()> {
    match node.node_type {
        NodeType::NeoCli
            if matches!(
                role,
                NodeRole::Consensus | NodeRole::Oracle | NodeRole::StateValidator
            ) =>
        {
            Ok(())
        }
        NodeType::NeoGo => Ok(()),
        NodeType::NeoCli => anyhow::bail!(
            "neo-cli cannot perform {} with a local wallet in NeoNexus",
            role.label()
        ),
        NodeType::NeoRs => anyhow::bail!(
            "neo-rs has no safe NEP-6 wallet integration; its plaintext private_key_hex path is not supported"
        ),
        NodeType::NeoXGeth | NodeType::NeoXReth => anyhow::bail!(
            "{} uses NeoX key material and cannot consume a Neo N3 NEP-6 signer profile",
            node.node_type
        ),
    }
}

fn ensure_sign_client_runtime(node: &NodeConfig, role: NodeRole) -> Result<()> {
    if node.node_type != NodeType::NeoCli || role != NodeRole::Consensus {
        anyhow::bail!(
            "{} {} cannot consume the Neo SecureSign gRPC protocol; only neo-cli consensus is supported",
            node.node_type,
            role.label()
        );
    }
    if node.runtime_version.trim().trim_start_matches('v') != REMOTE_SIGNER_NEO_CLI_VERSION {
        anyhow::bail!(
            "remote signer consensus is ABI-locked to neo-cli {}; node {} declares {}",
            REMOTE_SIGNER_NEO_CLI_VERSION,
            node.name,
            node.runtime_version
        );
    }
    Ok(())
}

fn ensure_bound_key(key: &SignerKeyRef, actual: &str, label: &str) -> Result<()> {
    if key.key_id != actual.trim() {
        anyhow::bail!(
            "node binding names key {}, but {label} owns {}",
            key.key_id,
            actual.trim()
        );
    }
    Ok(())
}

fn ensure_neo_n3_identity(
    node: &NodeConfig,
    chain_family: Option<&str>,
    magic: Option<u32>,
    network: &str,
) -> Result<()> {
    if chain_family.is_some_and(|family| family != "neo-n3") {
        anyhow::bail!("the selected signer key belongs to {chain_family:?}, not Neo N3");
    }
    let magic = magic.context("the selected signer key has no network magic")?;
    ensure_network_magic(node, magic)?;
    if network.trim() != node.network.to_string() {
        anyhow::bail!(
            "signer key network {} does not match node network {}",
            network.trim(),
            node.network
        );
    }
    Ok(())
}

fn ensure_network_magic(node: &NodeConfig, actual: u32) -> Result<()> {
    let expected = network_magic(node.network);
    if actual != expected {
        anyhow::bail!(
            "signer network magic {actual} does not match {} node magic {expected}",
            node.network
        );
    }
    Ok(())
}

fn ensure_role_plugin(node: &NodeConfig, role: NodeRole, plugins: &[PluginState]) -> Result<()> {
    if node.node_type != NodeType::NeoCli {
        return Ok(());
    }
    let id = match role {
        NodeRole::Consensus => PluginId::DBFTPlugin,
        NodeRole::Oracle => PluginId::OracleService,
        NodeRole::StateValidator => PluginId::StateService,
        _ => return Ok(()),
    };
    if plugins
        .iter()
        .any(|plugin| plugin.plugin_id == id && plugin.enabled)
    {
        Ok(())
    } else {
        anyhow::bail!("{} duty requires enabled {} plugin state", role.label(), id)
    }
}

fn role_plugin_name(role: NodeRole) -> Result<&'static str> {
    match role {
        NodeRole::Consensus => Ok("DBFTPlugin"),
        NodeRole::Oracle => Ok("OracleService"),
        NodeRole::StateValidator => Ok("StateService"),
        _ => anyhow::bail!("{} has no neo-cli signer plugin", role.label()),
    }
}

fn ensure_neo_cli_runtime_root(
    node: &NodeConfig,
    working_dir: &Path,
    plugins: &[&str],
) -> Result<()> {
    let binary = resolve_command_path(&node.binary_path).with_context(|| {
        format!(
            "neo-cli runtime {} could not be resolved",
            node.binary_path.display()
        )
    })?;
    let binary = std::fs::canonicalize(&binary)
        .with_context(|| format!("failed to resolve neo-cli runtime {}", binary.display()))?;
    let runtime_root = binary
        .parent()
        .context("neo-cli runtime has no parent directory")?;
    let working_dir = std::fs::canonicalize(working_dir).with_context(|| {
        format!(
            "failed to resolve node working directory {}",
            working_dir.display()
        )
    })?;
    if !same_path(runtime_root, &working_dir) {
        anyhow::bail!(
            "neo-cli loads plugins beside its binary, but this node writes isolated plugin config under {}; place a complete per-node neo-cli runtime in that directory (current binary root: {})",
            working_dir.display(),
            runtime_root.display()
        );
    }
    for plugin in plugins {
        let assembly = runtime_root
            .join("Plugins")
            .join(plugin)
            .join(format!("{plugin}.dll"));
        let metadata = std::fs::symlink_metadata(&assembly).with_context(|| {
            format!(
                "required neo-cli plugin assembly {} is not installed",
                assembly.display()
            )
        })?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            anyhow::bail!(
                "required neo-cli plugin assembly {} must be a regular non-symlink file",
                assembly.display()
            );
        }
    }
    Ok(())
}

#[cfg(windows)]
fn same_path(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

#[cfg(not(windows))]
fn same_path(left: &Path, right: &Path) -> bool {
    left == right
}

/// Resolve a node binding against the currently configured registry.
///
/// A missing profile is an error, never a reason to use a process-wide default
/// or another healthy backend.
pub fn resolve_node_signer<'a>(
    repository: &'a Repository,
    registry: &'a SignerRegistry,
    node: &NodeConfig,
) -> Result<Option<NodeSignerRoute<'a>>> {
    let Some(key) = node_signer_key(repository, node)? else {
        return Ok(None);
    };
    registry.backend(&key.backend_id).with_context(|| {
        format!(
            "node {} is bound to unavailable signer backend {}",
            node.name, key.backend_id
        )
    })?;
    Ok(Some(NodeSignerRoute {
        registry,
        key,
        repository,
    }))
}

/// The only application-level signer dispatch handle for a node.
#[derive(Debug)]
pub struct NodeSignerRoute<'a> {
    registry: &'a SignerRegistry,
    key: SignerKeyRef,
    repository: &'a Repository,
}

impl<'a> NodeSignerRoute<'a> {
    pub fn key(&self) -> &SignerKeyRef {
        &self.key
    }

    pub fn backend(&self) -> Result<&'a ConfiguredSignerBackend> {
        self.registry.backend(&self.key.backend_id)
    }

    fn record_signing_usage(&self, operation: &str) {
        let _ = self.repository.record_event(NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: crate::events::EventKind::NeoWalletProfileUsed,
            severity: crate::events::EventSeverity::Info,
            message: format!("wallet profile '{}' used for {operation}", self.key.key_id),
        });
    }

    pub fn sign_transaction(&self, request: &SignRequest) -> Result<Outcome<Signature>> {
        let result = self.registry.sign_transaction(&self.key, request)?;
        self.record_signing_usage("transaction signing");
        Ok(result)
    }

    pub fn sign_consensus(&self, request: &SignRequest) -> Result<Outcome<Signature>> {
        let result = self.registry.sign_consensus(&self.key, request)?;
        self.record_signing_usage("consensus signing");
        Ok(result)
    }

    pub fn sign_raw(&self, request: &RawSignRequest) -> Result<Outcome<RawSignature>> {
        let result = self.registry.sign_raw(&self.key, request)?;
        self.record_signing_usage("raw signing");
        Ok(result)
    }

    pub fn sign_eip191_fulfillment(
        &self,
        request: &Eip191FulfillmentRequest,
    ) -> Result<Outcome<Eip191FulfillmentSignature>> {
        let result = self.registry.sign_eip191_fulfillment(&self.key, request)?;
        self.record_signing_usage("EIP-191 fulfillment");
        Ok(result)
    }

    pub fn key_info(&self) -> Result<Outcome<KeyPublic>> {
        self.registry.key_info(&self.key)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/core/node_signer/tests.rs"]
mod tests;
