//! What the generator needs to know about a node beyond its definition.
//!
//! A node's *duty* decides which service sections its configuration carries,
//! and its wallet decides whether those services can be switched on. Neither
//! lives on `NodeConfig`: the duty and the wallet assignment are recorded
//! against the node in the repository, and the wallet password is an operator
//! secret supplied per export. They travel together here so a generator
//! signature does not grow a parameter every time a duty needs one more fact.

use crate::roles::NodeRole;

/// The wallet a signing service unlocks at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

impl GenerationContext {
    pub fn for_role(role: NodeRole) -> Self {
        Self {
            role: Some(role),
            wallet: None,
        }
    }

    pub fn with_wallet(mut self, wallet: ServiceWallet) -> Self {
        self.wallet = Some(wallet);
        self
    }
}
