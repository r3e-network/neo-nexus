//! What the generator needs to know about a node beyond its definition.
//!
//! A node's *duty* decides which service sections its configuration carries,
//! and its wallet decides whether those services can be switched on. Neither
//! lives on `NodeConfig`: the duty and the wallet assignment are recorded
//! against the node in the repository, and the wallet password is an operator
//! secret supplied per export. They travel together here so a generator
//! signature does not grow a parameter every time a duty needs one more fact.

use crate::roles::NodeRole;

/// How a neo-cli consensus process obtains signatures. This is runtime
/// configuration, not a fallback chain: exactly one variant is selected for a
/// signing node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusSigner {
    /// The node unlocks its bound NEP-6 wallet and DBFT follows that wallet.
    LocalWallet,
    /// DBFT resolves this exact `ISigner` name, backed by the official
    /// SignClient gRPC plugin.
    SignClient {
        name: String,
        endpoint: String,
        public_key: String,
        network_magic: u32,
    },
}

/// The wallet a signing service unlocks at startup.
#[derive(Clone, PartialEq, Eq)]
pub struct ServiceWallet {
    /// Path to the NEP-6 wallet file, from the profile assigned to the node.
    pub path: String,
    /// The password, supplied for this export only.
    ///
    /// **Never persisted.** Both clients read it in plaintext from the config
    /// file, and NeoNexus's own wallet validation fails any wallet carrying a
    /// plaintext secret — keeping a password in the workspace database would
    /// contradict the boundary the app enforces on everyone else. Without it
    /// the service is written with its path and left disabled, so the operator
    /// fills in one field rather than learning the schema.
    pub password: Option<String>,
}

impl std::fmt::Debug for ServiceWallet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServiceWallet")
            .field("path", &self.path)
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

impl ServiceWallet {
    /// A wallet known by path alone: enough to configure a service, not enough
    /// to start it.
    pub fn at(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            password: None,
        }
    }

    pub fn unlocked_with(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Whether the service can actually start. A path without a password
    /// configures the service but cannot unlock it.
    pub fn can_unlock(&self) -> bool {
        self.password
            .as_ref()
            .is_some_and(|value| !value.is_empty())
    }
}

/// Extra inputs for a config render. The default — no duty, no wallet — renders
/// exactly what NeoNexus rendered before duties existed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GenerationContext {
    pub role: Option<NodeRole>,
    pub wallet: Option<ServiceWallet>,
    pub consensus_signer: Option<ConsensusSigner>,
}

impl GenerationContext {
    pub fn for_role(role: NodeRole) -> Self {
        Self {
            role: Some(role),
            wallet: None,
            consensus_signer: None,
        }
    }

    pub fn with_wallet(mut self, wallet: ServiceWallet) -> Self {
        self.wallet = Some(wallet);
        self.consensus_signer = Some(ConsensusSigner::LocalWallet);
        self
    }

    pub fn with_sign_client(
        mut self,
        name: impl Into<String>,
        endpoint: impl Into<String>,
        public_key: impl Into<String>,
        network_magic: u32,
    ) -> Self {
        self.consensus_signer = Some(ConsensusSigner::SignClient {
            name: name.into(),
            endpoint: endpoint.into(),
            public_key: public_key.into(),
            network_magic,
        });
        self
    }
}
